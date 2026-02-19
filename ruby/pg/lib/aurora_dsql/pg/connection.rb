# frozen_string_literal: true

# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require "pg"

module AuroraDsql
  module Pg
    # Single connection wrapper for Aurora DSQL.
    class Connection
      attr_reader :pg_conn, :config

      # Create a new connection to Aurora DSQL.
      #
      # @param config [String, Config, nil] connection string or Config object
      # @param options [Hash] configuration options if config is nil
      # @return [Connection]
      def self.connect(config = nil, **options)
        cfg = case config
              when String then Config.parse(config)
              when Config then config
              when nil then Config.new(**options)
              else Config.new(**options.merge(config.to_h))
              end

        resolved = cfg.resolve

        token = Token.generate(
          host: resolved.host,
          region: resolved.region,
          user: resolved.user,
          credentials: resolved.credentials_provider,
          profile: resolved.profile,
          expires_in: resolved.token_duration
        )

        pg_conn = ::PG.connect(resolved.to_pg_params(password: token))
        new(pg_conn, resolved)
      end

      def initialize(pg_conn, config)
        @pg_conn = pg_conn
        @config = config
      end

      # Delegate common pg methods
      def exec(...)
        @pg_conn.exec(...)
      end

      def exec_params(...)
        @pg_conn.exec_params(...)
      end

      def query(...)
        @pg_conn.query(...)
      end

      def transaction(...)
        @pg_conn.transaction(...)
      end

      def close
        @pg_conn.close
      end

      def finished?
        @pg_conn.finished?
      end
    end
  end
end
