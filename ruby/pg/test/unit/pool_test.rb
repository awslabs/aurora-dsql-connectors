# frozen_string_literal: true

# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require "rspec"
require_relative "../../lib/aurora_dsql_pg"
require_relative "../../lib/aurora_dsql/pg/pool"

RSpec.describe AuroraDsql::Pg::Pool do
  let(:mock_pg_conn) { double("pg_conn", close: nil) }
  let(:mock_token_cache) { double("token_cache") }

  before do
    allow(AuroraDsql::Pg::TokenCache).to receive(:new).and_return(mock_token_cache)
    allow(mock_token_cache).to receive(:get_token).and_return("test-token")
    allow(mock_token_cache).to receive(:clear)

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
      expect(mock_token_cache).to receive(:get_token).and_return("fresh-token")

      pool.with { |conn| expect(conn).not_to be_nil }
    end
  end

  describe "#clear_token_cache" do
    it "delegates to token cache" do
      pool = described_class.create(host: "cluster.dsql.us-east-1.on.aws")

      expect(mock_token_cache).to receive(:clear)
      pool.clear_token_cache
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
