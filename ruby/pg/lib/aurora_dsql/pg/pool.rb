# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require "pg"
require "connection_pool"

module AuroraDsql
  module Pg
    # Connection pool for Aurora DSQL with token caching and max_lifetime enforcement.
    class Pool
      # Wrapper to track connection creation time for max_lifetime enforcement.
      PooledConnection = Struct.new(:conn, :created_at, keyword_init: true)

      # Create a new connection pool.
      def self.create(config = nil, **options)
        new(Config.from(config, **options).resolve)
      end

      def initialize(resolved_config)
        @config = resolved_config
        @token_cache = TokenCache.new(
          credentials_provider: resolved_config.credentials_provider,
          profile: resolved_config.profile
        )

        @pool = ConnectionPool.new(
          size: resolved_config.pool_size,
          timeout: resolved_config.checkout_timeout
        ) { create_connection }
      end

      # Maximum stale connection discards before giving up.
      MAX_STALE_RETRIES = 10

      # Check out a connection and yield it to the block.
      # Enforces max_lifetime. Retries on OCC errors only when occ_max_retries
      # is set in the pool config. Pass retry_occ: false to skip retry on
      # individual calls.
      def with(retry_occ: @config.occ_max_retries, &block)
        return checkout_and_execute(&block) unless retry_occ

        unless retry_occ.is_a?(Integer) && retry_occ > 0
          raise ArgumentError,
                "retry_occ must be false/nil or a positive integer, got #{retry_occ.inspect}. " \
                "Configure occ_max_retries on the pool instead of passing retry_occ: true"
        end

        occ_config = OCCRetry::DEFAULT_CONFIG.merge(max_retries: retry_occ)
        OCCRetry.retry_on_occ(occ_config, logger: @config.logger) { checkout_and_execute(&block) }
      end

      # Clear all cached authentication tokens.
      def clear_token_cache
        @token_cache.clear
      end

      # Shutdown the pool, closing all connections.
      def shutdown
        @pool.shutdown { |wrapped| wrapped.conn.close rescue nil }
      end

      private

      # Check out a connection, handling stale connection replacement.
      # Loops because each @pool.with checkout may return a stale connection
      # that must be discarded before retrying with a fresh one.
      def checkout_and_execute(&block)
        stale_retries = 0

        loop do
          @pool.with do |wrapped|
            if stale?(wrapped)
              stale_retries += 1
              if stale_retries > MAX_STALE_RETRIES
                raise AuroraDsql::Pg::Error,
                      "unable to acquire a non-stale connection after #{MAX_STALE_RETRIES} attempts"
              end
              @config.logger&.warn(
                "[AuroraDsql::Pg] Discarding stale connection " \
                "(age #{(Time.now - wrapped.created_at).round}s, max_lifetime #{@config.max_lifetime}s)"
              )
              @pool.discard_current_connection
              wrapped.conn.close rescue nil
            else
              return block.call(wrapped.conn)
            end
          end
        end
      end

      def stale?(wrapped)
        Time.now - wrapped.created_at > @config.max_lifetime
      end

      def create_connection
        token = @token_cache.get_token(
          host: @config.host,
          region: @config.region,
          user: @config.user,
          duration: @config.token_duration
        )
        conn = ::PG.connect(@config.to_pg_params(password: token))
        PooledConnection.new(conn: conn, created_at: Time.now)
      end
    end
  end
end
