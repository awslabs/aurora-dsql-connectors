# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require "rspec"
require_relative "../../lib/aurora_dsql_pg"

TABLE_NAME = "occ_retry_test_#{Process.pid}"

RSpec.describe "OCC retry integration", order: :defined do
  before(:all) do
    skip "CLUSTER_ENDPOINT required for integration test" unless ENV["CLUSTER_ENDPOINT"]

    @pool = AuroraDsql::Pg.create_pool(
      host: ENV.fetch("CLUSTER_ENDPOINT"),
      user: ENV.fetch("CLUSTER_USER", "admin"),
      region: ENV.fetch("REGION", nil),
      pool_size: 5,
      logger: Logger.new($stdout)
    )

    # Retry initial connection to handle DNS propagation delay
    # on freshly created clusters in CI.
    retries = 0
    begin
      AuroraDsql::Pg::OCCRetry.exec_with_retry(
        @pool,
        "CREATE TABLE IF NOT EXISTS #{TABLE_NAME} (id UUID DEFAULT gen_random_uuid() PRIMARY KEY, value INT NOT NULL DEFAULT 0)"
      )
    rescue PG::ConnectionBad => e
      retries += 1
      raise if retries > 5

      sleep(retries * 5)
      retry
    end
  end

  after(:all) do
    if @pool
      @pool.with { |conn| conn.exec("DROP TABLE IF EXISTS #{TABLE_NAME}") } rescue nil
      @pool.shutdown
    end
  end

  it "performs a basic transactional write with automatic OCC retry" do
    @pool.with do |conn|
      conn.transaction do
        conn.exec_params("INSERT INTO #{TABLE_NAME} (value) VALUES ($1)", [42])
      end
    end

    result = @pool.with { |conn| conn.exec("SELECT count(*) FROM #{TABLE_NAME} WHERE value = 42") }
    expect(result[0]["count"].to_i).to be >= 1
  end

  it "retries on OCC conflict from concurrent transactions" do
    # Insert a row to contend on
    row_id = nil
    @pool.with do |conn|
      conn.transaction do
        res = conn.exec_params(
          "INSERT INTO #{TABLE_NAME} (value) VALUES ($1) RETURNING id", [0]
        )
        row_id = res[0]["id"]
      end
    end

    # Concurrent read-modify-write on the same row to trigger OCC conflict.
    barrier = Queue.new
    results = Array.new(2)
    errors = Array.new(2)

    threads = 2.times.map do |i|
      Thread.new do
        begin
          @pool.with do |conn|
            conn.transaction do
              current = conn.exec_params(
                "SELECT value FROM #{TABLE_NAME} WHERE id = $1", [row_id]
              )
              current_value = current[0]["value"].to_i

              # Synchronize so both threads read before either commits
              barrier << i
              sleep(0.2)

              conn.exec_params(
                "UPDATE #{TABLE_NAME} SET value = $1 WHERE id = $2",
                [current_value + 1, row_id]
              )
            end
          end
          results[i] = :success
        rescue => e
          errors[i] = e
        end
      end
    end

    threads.each(&:join)

    # Both threads should succeed (one via OCC retry)
    results.each_with_index do |r, i|
      expect(r).to eq(:success), "Thread #{i} failed: #{errors[i]&.message}"
    end

    # Both increments applied
    final = @pool.with { |conn| conn.exec_params("SELECT value FROM #{TABLE_NAME} WHERE id = $1", [row_id]) }
    expect(final[0]["value"].to_i).to eq(2)
  end

  it "retries OCC via OCCRetry.with_retry" do
    AuroraDsql::Pg::OCCRetry.with_retry(@pool) do |conn|
      conn.exec_params("INSERT INTO #{TABLE_NAME} (value) VALUES ($1)", [99])
    end

    result = @pool.with { |conn| conn.exec("SELECT count(*) FROM #{TABLE_NAME} WHERE value = 99") }
    expect(result[0]["count"].to_i).to be >= 1
  end

  it "retries OCC via OCCRetry.exec_with_retry" do
    AuroraDsql::Pg::OCCRetry.exec_with_retry(
      @pool,
      "INSERT INTO #{TABLE_NAME} (value) VALUES (77)"
    )

    result = @pool.with { |conn| conn.exec("SELECT count(*) FROM #{TABLE_NAME} WHERE value = 77") }
    expect(result[0]["count"].to_i).to be >= 1
  end
end
