# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require "pg"
require "connection_pool"

module AuroraDsql
  module Pg
    # Connection pool for Aurora DSQL with max_lifetime enforcement.
    class Pool
      # Wrapper to track connection creation time for max_lifetime enforcement.
      PooledConnection = Struct.new(:conn, :created_at, keyword_init: true)

      POOL_DEFAULTS = { size: 5, timeout: 5 }.freeze

      # Create a new connection pool.
      def self.create(config = nil, pool: {}, **options)
        new(Config.from(config, **options).resolve, pool)
      end

      def initialize(resolved_config, pool_options = {})
        @config = resolved_config

        effective_pool = POOL_DEFAULTS.merge(pool_options)
        @pool = ConnectionPool.new(**effective_pool) { create_connection }
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
        token = Token.generate(
          host: @config.host,
          region: @config.region,
          user: @config.user,
          credentials: @config.credentials_provider,
          profile: @config.profile,
          expires_in: @config.token_duration
        )
        conn = ::PG.connect(@config.to_pg_params(password: token))
        PooledConnection.new(conn: conn, created_at: Time.now)
      end
    end
  end
end
