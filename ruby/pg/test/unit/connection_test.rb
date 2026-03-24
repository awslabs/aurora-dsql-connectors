# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require "rspec"
require_relative "../../lib/aurora_dsql_pg"

RSpec.describe "AuroraDsql::Pg.connect" do
  before do
    allow(AuroraDsql::Pg::Token).to receive(:generate).and_return("test-token")
    stub_const("PG", Class.new do
      @last_params = nil

      class << self
        attr_accessor :last_params

        def connect(params)
          self.last_params = params
          Object.new.tap do |conn|
            def conn.close; end
            def conn.finished?; false; end
          end
        end

        def library_version
          170000
        end
      end
    end)
  end

  it "returns a raw PG::Connection from keyword args" do
    conn = AuroraDsql::Pg.connect(host: "cluster.dsql.us-east-1.on.aws")
    expect(conn).not_to be_nil
    expect(conn).to respond_to(:close)
    expect(conn).to respond_to(:finished?)
  end

  it "returns a raw PG::Connection from connection string" do
    conn = AuroraDsql::Pg.connect("postgres://admin@cluster.dsql.us-east-1.on.aws/postgres")
    expect(conn).not_to be_nil
    expect(conn).to respond_to(:close)
  end

  it "returns a raw PG::Connection from Config object" do
    config = AuroraDsql::Pg::Config.new(host: "cluster.dsql.us-east-1.on.aws")
    conn = AuroraDsql::Pg.connect(config)
    expect(conn).not_to be_nil
    expect(conn).to respond_to(:close)
  end

  it "passes correct params to PG.connect" do
    AuroraDsql::Pg.connect(host: "cluster.dsql.us-east-1.on.aws")

    params = PG.last_params
    expect(params[:host]).to eq("cluster.dsql.us-east-1.on.aws")
    expect(params[:user]).to eq("admin")
    expect(params[:dbname]).to eq("postgres")
    expect(params[:password]).to eq("test-token")
    expect(params[:sslmode]).to eq("verify-full")
  end
end
