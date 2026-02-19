# frozen_string_literal: true

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

      # Check out a connection and yield it to the block.
      # Enforces max_lifetime by replacing stale connections on checkout.
      #
      # @yield [PG::Connection] the database connection
      def with(&block)
        result = nil
        retry_checkout = true

        while retry_checkout
          @pool.with do |wrapped|
            if stale?(wrapped)
              wrapped.conn.close rescue nil
              @pool.discard_current_connection
              # retry_checkout stays true, loop will continue
            else
              result = block.call(wrapped.conn)
              retry_checkout = false
            end
          end
        end

        result
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
