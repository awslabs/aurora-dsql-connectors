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
      #
      # @param config [String, Config, nil] connection string or Config object
      # @param options [Hash] configuration options
      # @return [Pool]
      def self.create(config = nil, **options)
        cfg = case config
              when String then Config.parse(config)
              when Config then config
              when nil then Config.new(**options)
              else Config.new(**options.merge(config.to_h))
              end

        resolved = cfg.resolve
        new(resolved)
      end

      def initialize(resolved_config)
        @config = resolved_config
        @token_cache = TokenCache.new(
          credentials_provider: resolved_config.credentials_provider,
          profile: resolved_config.profile
        )

        @pool = ConnectionPool.new(
          size: resolved_config.pool_size,
          timeout: 5
        ) { create_connection }
      end

      # Maximum number of stale connections to discard before giving up.
      # Prevents infinite loops if create_connection keeps failing.
      MAX_STALE_RETRIES = 10

      # Check out a connection and yield it to the block.
      # Enforces max_lifetime by replacing stale connections on checkout.
      # Automatically retries on OCC (Optimistic Concurrency Control) errors
      # with exponential backoff unless retry_occ: false is passed.
      #
      # @param retry_occ [Boolean] whether to retry on OCC errors (default: true)
      # @yield [PG::Connection] the database connection
      def with(retry_occ: true, &block)
        return checkout_and_execute(&block) unless retry_occ

        cfg = OCCRetry::DEFAULT_CONFIG
        wait = cfg[:initial_wait]
        last_error = nil

        (0..cfg[:max_retries]).each do |attempt|
          begin
            return checkout_and_execute(&block)
          rescue StandardError => e
            raise unless OCCRetry.occ_error?(e)

            last_error = e

            if attempt < cfg[:max_retries]
              jittered_wait = wait + rand * wait / 4
              @config.logger&.warn(
                "[AuroraDsql::Pg] OCC conflict detected, retrying " \
                "(attempt #{attempt + 1}/#{cfg[:max_retries]}, wait #{jittered_wait.round(2)}s)"
              )
              sleep(jittered_wait)
              wait = [wait * cfg[:multiplier], cfg[:max_wait]].min
            end
          end
        end

        raise AuroraDsql::Pg::Error,
              "Max retries (#{cfg[:max_retries]}) exceeded, last error: #{last_error&.message}"
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
              wrapped.conn.close rescue nil
              @pool.discard_current_connection
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
