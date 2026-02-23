# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require "pg"

module AuroraDsql
  module Pg
    # OCC (Optimistic Concurrency Control) retry utilities for Aurora DSQL.
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

      # Check if an error is an OCC conflict.
      # Checks SQLSTATE first, then falls back to message matching.
      def self.occ_error?(error)
        return false if error.nil?

        # Prefer structured SQLSTATE check when available
        if error.respond_to?(:result) && error.result&.respond_to?(:error_field)
          sqlstate = error.result.error_field(PG::Result::PG_DIAG_SQLSTATE)
          return true if sqlstate == SQLSTATE_SERIALIZATION_FAILURE
        end

        # Fall back to message matching for OCC-specific codes
        msg = error.message.to_s
        msg.include?(ERROR_CODE_MUTATION) || msg.include?(ERROR_CODE_SCHEMA)
      end

      # Execute a transactional block with automatic retry on OCC conflicts.
      def self.with_retry(pool, config = {}, &block)
        cfg = DEFAULT_CONFIG.merge(config)
        wait = cfg[:initial_wait]
        last_error = nil

        (0..cfg[:max_retries]).each do |attempt|
          begin
            pool.with(retry_occ: false) do |conn|
              result = conn.transaction { block.call(conn) }
              return result
            end
          rescue StandardError => e
            raise unless occ_error?(e)

            last_error = e

            # Sleep before retry (unless this was the last attempt)
            if attempt < cfg[:max_retries]
              sleep(wait + rand * wait / 4)
              wait = [wait * cfg[:multiplier], cfg[:max_wait]].min
            end
          end
        end

        raise AuroraDsql::Pg::Error, "Max retries (#{cfg[:max_retries]}) exceeded, last error: #{last_error&.message}"
      end

      # Execute a SQL statement with automatic retry on OCC conflicts.
      def self.exec_with_retry(pool, sql, max_retries: 3)
        with_retry(pool, max_retries: max_retries) { |conn| conn.exec(sql) }
      end
    end
  end
end
