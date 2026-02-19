#!/usr/bin/env ruby
# frozen_string_literal: true

# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require "aurora_dsql_pg"

NUM_CONCURRENT_QUERIES = 8

def example
  cluster_endpoint = ENV.fetch("CLUSTER_ENDPOINT") do
    raise "CLUSTER_ENDPOINT environment variable is not set"
  end

  pool = AuroraDsql::Pg.create_pool(
    host: cluster_endpoint,
    pool_size: 10
  )

  # Verify connection
  pool.with { |conn| conn.exec("SELECT 1") }
  puts "Connected to Aurora DSQL"

  # Run concurrent queries using the connection pool
  threads = NUM_CONCURRENT_QUERIES.times.map do |i|
    Thread.new do
      pool.with do |conn|
        result = conn.exec_params("SELECT $1::int AS worker_id", [i])
        puts "Worker #{i} result: #{result[0]['worker_id']}"
      end
    end
  end

  threads.each(&:join)
  puts "Connection pool with concurrent connections exercised successfully"
ensure
  pool&.shutdown
end

example if __FILE__ == $PROGRAM_NAME
