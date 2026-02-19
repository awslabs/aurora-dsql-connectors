# frozen_string_literal: true

# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require "rspec"
require_relative "../../lib/aurora_dsql_pg"
require_relative "../../lib/aurora_dsql/pg/connection"

RSpec.describe AuroraDsql::Pg::Connection do
  let(:mock_pg_conn) do
    double("pg_conn").tap do |conn|
      allow(conn).to receive(:exec)
      allow(conn).to receive(:exec_params)
      allow(conn).to receive(:query)
      allow(conn).to receive(:transaction).and_yield
      allow(conn).to receive(:close)
      allow(conn).to receive(:finished?).and_return(false)
      allow(conn).to receive(:prepare)
    end
  end

  let(:mock_config) do
    AuroraDsql::Pg::ResolvedConfig.new(
      host: "cluster.dsql.us-east-1.on.aws",
      region: "us-east-1",
      user: "admin",
      database: "postgres",
      port: 5432,
      profile: nil,
      token_duration: 900,
      credentials_provider: nil,
      max_lifetime: 3300,
      pool_size: 5,
      application_name: nil
    )
  end

  subject { described_class.new(mock_pg_conn, mock_config) }

  describe "#pg_conn" do
    it "exposes the underlying PG::Connection" do
      expect(subject.pg_conn).to eq(mock_pg_conn)
    end
  end

  describe "#config" do
    it "exposes the resolved config" do
      expect(subject.config).to eq(mock_config)
    end
  end

  describe "explicit delegated methods" do
    it "delegates exec to pg_conn" do
      expect(mock_pg_conn).to receive(:exec).with("SELECT 1")
      subject.exec("SELECT 1")
    end

    it "delegates exec_params to pg_conn" do
      expect(mock_pg_conn).to receive(:exec_params).with("SELECT $1", [1])
      subject.exec_params("SELECT $1", [1])
    end

    it "delegates query to pg_conn" do
      expect(mock_pg_conn).to receive(:query).with("SELECT 1")
      subject.query("SELECT 1")
    end

    it "delegates transaction to pg_conn" do
      expect(mock_pg_conn).to receive(:transaction).and_yield
      subject.transaction { "in transaction" }
    end

    it "delegates close to pg_conn" do
      expect(mock_pg_conn).to receive(:close)
      subject.close
    end

    it "delegates finished? to pg_conn" do
      expect(mock_pg_conn).to receive(:finished?).and_return(true)
      expect(subject.finished?).to be true
    end
  end

  describe "method_missing delegation" do
    it "delegates unknown methods to pg_conn if it responds" do
      expect(mock_pg_conn).to receive(:prepare).with("stmt", "SELECT 1")
      subject.send(:prepare, "stmt", "SELECT 1")
    end

    it "raises NoMethodError for methods pg_conn doesn't respond to" do
      expect { subject.send(:nonexistent_method) }.to raise_error(NoMethodError)
    end
  end

  describe "#respond_to_missing?" do
    it "returns true for methods pg_conn responds to" do
      expect(subject.respond_to?(:prepare, true)).to be true
    end

    it "returns false for methods pg_conn doesn't respond to" do
      expect(subject.respond_to?(:nonexistent_method, true)).to be false
    end
  end

  describe ".connect" do
    before do
      allow(AuroraDsql::Pg::Token).to receive(:generate).and_return("test-token")
      stub_const("PG", Class.new do
        def self.connect(params)
          Object.new.tap do |conn|
            def conn.close; end
            def conn.finished?; false; end
            def conn.respond_to?(method, include_private = false)
              [:close, :finished?].include?(method)
            end
          end
        end

        def self.library_version
          170000
        end
      end)
    end

    it "creates a Connection from keyword args" do
      conn = described_class.connect(host: "cluster.dsql.us-east-1.on.aws")
      expect(conn).to be_a(described_class)
      expect(conn.config.host).to eq("cluster.dsql.us-east-1.on.aws")
    end

    it "creates a Connection from connection string" do
      conn = described_class.connect("postgres://admin@cluster.dsql.us-east-1.on.aws/postgres")
      expect(conn).to be_a(described_class)
      expect(conn.config.host).to eq("cluster.dsql.us-east-1.on.aws")
    end

    it "creates a Connection from Config object" do
      config = AuroraDsql::Pg::Config.new(host: "cluster.dsql.us-east-1.on.aws")
      conn = described_class.connect(config)
      expect(conn).to be_a(described_class)
    end
  end
end
