# frozen_string_literal: true

# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require "pg"

module AuroraDsql
  module Pg
    # OCC (Optimistic Concurrency Control) retry utilities for Aurora DSQL.
    #
    # Aurora DSQL uses OCC where conflicts are detected at commit time.
    # When two transactions modify the same data, the first to commit wins
    # and the second receives an OCC error.
    module OCCRetry
      # OCC error code for mutation conflicts.
      ERROR_CODE_MUTATION = "OC000"

      # OCC error code for schema conflicts.
      ERROR_CODE_SCHEMA = "OC001"

      # SQLSTATE for serialization failure.
      SQLSTATE_SERIALIZATION_FAILURE = "40001"

      # Default retry configuration.
      DEFAULT_CONFIG = {
        max_retries: 3,
        initial_wait: 0.1,    # 100ms
        max_wait: 5.0,        # 5 seconds
        multiplier: 2.0
      }.freeze

      # Check if an error is an OCC conflict error.
      #
      # @param error [Exception, nil] the error to check
      # @return [Boolean] true if it's an OCC error
      def self.occ_error?(error)
        return false if error.nil?

        msg = error.message.to_s
        return true if msg.include?(ERROR_CODE_MUTATION) || msg.include?(ERROR_CODE_SCHEMA)

        # Check SQLSTATE via PG error result
        if error.respond_to?(:result) && error.result&.respond_to?(:error_field)
          sqlstate = error.result.error_field(PG::Result::PG_DIAG_SQLSTATE)
          return sqlstate == SQLSTATE_SERIALIZATION_FAILURE
        end

        false
      end

      # Execute a transactional block with automatic retry on OCC conflicts.
      #
      # @param pool [Pool] the connection pool
      # @param config [Hash] retry configuration options
      # @yield [PG::Connection] yields connection within a transaction
      # @raise [StandardError] if max retries exceeded or non-OCC error
      def self.with_retry(pool, config = {}, &block)
        cfg = DEFAULT_CONFIG.merge(config)
        wait = cfg[:initial_wait]

        (0..cfg[:max_retries]).each do |attempt|
          begin
            pool.with do |conn|
              result = conn.transaction { block.call(conn) }
              return result
            end
          rescue StandardError => e
            raise unless occ_error?(e) && attempt < cfg[:max_retries]

            sleep(wait + rand * wait / 4)
            wait = [wait * cfg[:multiplier], cfg[:max_wait]].min
          end
        end

        raise "Max retries (#{cfg[:max_retries]}) exceeded"
      end

      # Execute a SQL statement with automatic retry on OCC conflicts.
      #
      # @param pool [Pool] the connection pool
      # @param sql [String] the SQL statement
      # @param max_retries [Integer] maximum retry attempts
      def self.exec_with_retry(pool, sql, max_retries: 3)
        with_retry(pool, max_retries: max_retries) { |conn| conn.exec(sql) }
      end
    end
  end
end
