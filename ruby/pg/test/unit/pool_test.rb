# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require "rspec"
require_relative "../../lib/aurora_dsql_pg"
require_relative "../../lib/aurora_dsql/pg/pool"

RSpec.describe AuroraDsql::Pg::Pool do
  let(:mock_pg_conn) { double("pg_conn", close: nil) }

  before do
    allow(AuroraDsql::Pg::Token).to receive(:generate).and_return("test-token")

    # Mock PG.connect
    stub_const("PG", Class.new do
      def self.connect(params)
        @last_params = params
        Object.new.tap do |conn|
          def conn.close; end
        end
      end

      def self.last_params
        @last_params
      end

      def self.library_version
        170000
      end
    end)
  end

  describe ".create" do
    it "creates a pool with resolved config" do
      pool = described_class.create(host: "cluster.dsql.us-east-1.on.aws")

      expect(pool).to be_a(described_class)
    end

    it "accepts connection string" do
      pool = described_class.create("postgres://admin@cluster.dsql.us-east-1.on.aws/postgres")

      expect(pool).to be_a(described_class)
    end
  end

  describe "custom pool options" do
    it "accepts pool options hash" do
      pool = described_class.create(
        host: "cluster.dsql.us-east-1.on.aws",
        pool: { size: 10, timeout: 10 }
      )
      expect(pool).to be_a(described_class)
    end
  end

  describe "#with" do
    it "yields a connection from the pool" do
      pool = described_class.create(host: "cluster.dsql.us-east-1.on.aws")

      connection_yielded = false
      pool.with do |conn|
        connection_yielded = true
        expect(conn).not_to be_nil
      end

      expect(connection_yielded).to be true
    end
  end

  describe "max_lifetime enforcement" do
    it "replaces stale connections on checkout" do
      pool = described_class.create(
        host: "cluster.dsql.us-east-1.on.aws",
        max_lifetime: 10  # 10 seconds for test
      )

      # Get initial connection
      initial_conn = nil
      pool.with { |conn| initial_conn = conn }

      # Simulate time passing beyond max_lifetime
      allow(Time).to receive(:now).and_return(Time.now + 15)

      # Next checkout should get a fresh connection
      expect(AuroraDsql::Pg::Token).to receive(:generate).and_return("fresh-token")

      pool.with { |conn| expect(conn).not_to be_nil }
    end
  end

  describe "OCC retry" do
    it "does not retry by default when occ_max_retries is not set" do
      pool = described_class.create(host: "cluster.dsql.us-east-1.on.aws")

      call_count = 0
      expect {
        pool.with do |_conn|
          call_count += 1
          raise StandardError.new("OC000: transaction conflict")
        end
      }.to raise_error(StandardError, "OC000: transaction conflict")
      expect(call_count).to eq(1)
    end

    it "retries on OCC error and succeeds on next attempt" do
      pool = described_class.create(host: "cluster.dsql.us-east-1.on.aws", occ_max_retries: 3)
      allow(AuroraDsql::Pg::OCCRetry).to receive(:sleep)

      call_count = 0
      result = pool.with do |conn|
        call_count += 1
        raise StandardError.new("OC000: transaction conflict") if call_count == 1
        "success"
      end

      expect(result).to eq("success")
      expect(call_count).to eq(2)
    end

    it "raises after max retries exceeded" do
      pool = described_class.create(host: "cluster.dsql.us-east-1.on.aws", occ_max_retries: 3)
      allow(AuroraDsql::Pg::OCCRetry).to receive(:sleep)

      expect {
        pool.with do |_conn|
          raise StandardError.new("OC000: transaction conflict")
        end
      }.to raise_error(AuroraDsql::Pg::Error, /Max retries.*exceeded.*OC000: transaction conflict/)
    end

    it "does not retry non-OCC errors" do
      pool = described_class.create(host: "cluster.dsql.us-east-1.on.aws", occ_max_retries: 3)

      expect {
        pool.with do |_conn|
          raise StandardError.new("connection refused")
        end
      }.to raise_error(StandardError, "connection refused")
    end

    it "skips retry when retry_occ: false" do
      pool = described_class.create(host: "cluster.dsql.us-east-1.on.aws", occ_max_retries: 3)

      call_count = 0
      expect {
        pool.with(retry_occ: false) do |_conn|
          call_count += 1
          raise StandardError.new("OC000: transaction conflict")
        end
      }.to raise_error(StandardError, "OC000: transaction conflict")
      expect(call_count).to eq(1)
    end

    it "raises ArgumentError when retry_occ: true is passed explicitly" do
      pool = described_class.create(host: "cluster.dsql.us-east-1.on.aws", occ_max_retries: 3)

      expect {
        pool.with(retry_occ: true) { |_conn| }
      }.to raise_error(ArgumentError, /retry_occ must be false\/nil or a positive integer/)
    end

    it "skips retry when retry_occ: nil is passed explicitly" do
      pool = described_class.create(host: "cluster.dsql.us-east-1.on.aws", occ_max_retries: 3)

      call_count = 0
      expect {
        pool.with(retry_occ: nil) do |_conn|
          call_count += 1
          raise StandardError.new("OC000: transaction conflict")
        end
      }.to raise_error(StandardError, "OC000: transaction conflict")
      expect(call_count).to eq(1)
    end

    it "respects the configured occ_max_retries count" do
      pool = described_class.create(host: "cluster.dsql.us-east-1.on.aws", occ_max_retries: 1)
      allow(AuroraDsql::Pg::OCCRetry).to receive(:sleep)

      call_count = 0
      expect {
        pool.with do |_conn|
          call_count += 1
          raise StandardError.new("OC000: transaction conflict")
        end
      }.to raise_error(AuroraDsql::Pg::Error, /Max retries.*exceeded/)
      expect(call_count).to eq(2) # initial + 1 retry
    end

    it "logs OCC retries at warn level" do
      logger = double("logger")
      allow(logger).to receive(:warn)

      pool = described_class.create(
        host: "cluster.dsql.us-east-1.on.aws",
        occ_max_retries: 3,
        logger: logger
      )
      allow(AuroraDsql::Pg::OCCRetry).to receive(:sleep)

      call_count = 0
      pool.with do |_conn|
        call_count += 1
        raise StandardError.new("OC000: transaction conflict") if call_count == 1
        "success"
      end

      expect(logger).to have_received(:warn).with(/OCC conflict detected.*attempt 1/)
    end
  end

  describe "MAX_STALE_RETRIES limit" do
    it "raises after exceeding stale retry limit" do
      pool = described_class.create(
        host: "cluster.dsql.us-east-1.on.aws",
        max_lifetime: 1
      )

      # Stub stale? to always return true so every checkout discards
      allow(pool).to receive(:stale?).and_return(true)

      expect {
        pool.with { |_conn| }
      }.to raise_error(AuroraDsql::Pg::Error, /unable to acquire a non-stale connection/)
    end
  end

  describe "default configuration" do
    it "uses correct defaults" do
      pool = described_class.create(host: "cluster.dsql.us-east-1.on.aws")

      # Verify connection params via PG.last_params
      pool.with { |_| }  # Force a connection

      params = PG.last_params
      expect(params[:sslmode]).to eq("verify-full")
      expect(params[:application_name]).to eq("aurora-dsql-ruby-pg/1.0.0")
      expect(params[:sslnegotiation]).to eq("direct")
    end
  end
end
