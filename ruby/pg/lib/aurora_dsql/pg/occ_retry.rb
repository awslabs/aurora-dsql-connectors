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

      # Retry a block on OCC conflicts with exponential backoff and jitter.
      # Used by both Pool#with and OCCRetry.with_retry.
      def self.retry_on_occ(config = DEFAULT_CONFIG, logger: nil)
        wait = config[:initial_wait]
        last_error = nil

        (0..config[:max_retries]).each do |attempt|
          begin
            return yield
          rescue StandardError => e
            raise unless occ_error?(e)

            last_error = e

            if attempt < config[:max_retries]
              jittered_wait = wait + rand * wait / 4
              logger&.warn(
                "[AuroraDsql::Pg] OCC conflict detected, retrying " \
                "(attempt #{attempt + 1}/#{config[:max_retries]}, wait #{jittered_wait.round(2)}s)"
              )
              sleep(jittered_wait)
              wait = [wait * config[:multiplier], config[:max_wait]].min
            end
          end
        end

        # Re-raise inside rescue so Ruby sets .cause to the original OCC error.
        begin
          raise last_error
        rescue StandardError
          raise AuroraDsql::Pg::Error,
                "Max retries (#{config[:max_retries]}) exceeded, last error: #{last_error&.message}"
        end
      end

      # Execute a transactional block with automatic retry on OCC conflicts.
      def self.with_retry(pool, config = {}, &block)
        retry_on_occ(DEFAULT_CONFIG.merge(config)) do
          pool.with(retry_occ: false) do |conn|
            conn.transaction { block.call(conn) }
          end
        end
      end

      # Execute a single SQL statement with automatic retry on OCC conflicts.
      # Unlike with_retry, this does NOT wrap in an explicit transaction,
      # making it suitable for both DDL (CREATE TABLE, etc.) and single DML statements.
      def self.exec_with_retry(pool, sql, max_retries: 3)
        retry_on_occ(DEFAULT_CONFIG.merge(max_retries: max_retries)) do
          pool.with(retry_occ: false) { |conn| conn.exec(sql) }
        end
      end
    end
  end
end
