# frozen_string_literal: true

# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require "rspec"
require_relative "../../lib/aurora_dsql_pg"
require_relative "../../lib/aurora_dsql/pg/util"
require_relative "../../lib/aurora_dsql/pg/config"

RSpec.describe AuroraDsql::Pg::Config do
  describe "DEFAULTS" do
    it "has correct default values" do
      expect(AuroraDsql::Pg::Config::DEFAULTS[:user]).to eq("admin")
      expect(AuroraDsql::Pg::Config::DEFAULTS[:database]).to eq("postgres")
      expect(AuroraDsql::Pg::Config::DEFAULTS[:port]).to eq(5432)
      expect(AuroraDsql::Pg::Config::DEFAULTS[:max_lifetime]).to eq(55 * 60)
      expect(AuroraDsql::Pg::Config::DEFAULTS[:token_duration]).to eq(15 * 60)
      expect(AuroraDsql::Pg::Config::DEFAULTS[:pool_size]).to eq(5)
    end
  end

  describe "#resolve" do
    it "applies defaults for minimal config" do
      config = described_class.new(host: "mycluster.dsql.us-east-1.on.aws")
      resolved = config.resolve

      expect(resolved.host).to eq("mycluster.dsql.us-east-1.on.aws")
      expect(resolved.region).to eq("us-east-1")
      expect(resolved.user).to eq("admin")
      expect(resolved.database).to eq("postgres")
      expect(resolved.port).to eq(5432)
      expect(resolved.token_duration).to eq(15 * 60)
    end

    it "expands cluster ID to full hostname" do
      config = described_class.new(
        host: "ijsamhssbh36dopuigphknejb4",
        region: "us-west-2"
      )
      resolved = config.resolve

      expect(resolved.host).to eq("ijsamhssbh36dopuigphknejb4.dsql.us-west-2.on.aws")
      expect(resolved.region).to eq("us-west-2")
    end

    it "raises error for cluster ID without region" do
      ENV.delete("AWS_REGION")
      ENV.delete("AWS_DEFAULT_REGION")

      config = described_class.new(host: "ijsamhssbh36dopuigphknejb4")

      expect { config.resolve }.to raise_error(AuroraDsql::Pg::Error, /region is required/)
    end

    it "uses region from env for cluster ID" do
      ENV["AWS_REGION"] = "ap-northeast-1"
      config = described_class.new(host: "ijsamhssbh36dopuigphknejb4")
      resolved = config.resolve

      expect(resolved.region).to eq("ap-northeast-1")
      expect(resolved.host).to eq("ijsamhssbh36dopuigphknejb4.dsql.ap-northeast-1.on.aws")
    ensure
      ENV.delete("AWS_REGION")
    end

    it "raises error for missing host" do
      config = described_class.new

      expect { config.resolve }.to raise_error(AuroraDsql::Pg::Error, /host is required/)
    end

    it "raises error for invalid port" do
      config = described_class.new(
        host: "mycluster.dsql.us-east-1.on.aws",
        port: 70000
      )

      expect { config.resolve }.to raise_error(AuroraDsql::Pg::Error, /port must be between/)
    end

    it "allows explicit region override" do
      config = described_class.new(
        host: "mycluster.dsql.us-east-1.on.aws",
        region: "eu-west-1"
      )
      resolved = config.resolve

      expect(resolved.region).to eq("eu-west-1")
    end

    it "preserves all user-provided values" do
      config = described_class.new(
        host: "mycluster.dsql.us-east-1.on.aws",
        user: "myuser",
        database: "mydb",
        port: 5433,
        profile: "myprofile",
        token_duration: 300,
        pool_size: 10,
        application_name: "rails"
      )
      resolved = config.resolve

      expect(resolved.user).to eq("myuser")
      expect(resolved.database).to eq("mydb")
      expect(resolved.port).to eq(5433)
      expect(resolved.profile).to eq("myprofile")
      expect(resolved.token_duration).to eq(300)
      expect(resolved.pool_size).to eq(10)
      expect(resolved.application_name).to eq("rails")
    end

    it "returns a frozen ResolvedConfig" do
      config = described_class.new(host: "mycluster.dsql.us-east-1.on.aws")
      resolved = config.resolve

      expect(resolved).to be_frozen
      expect(resolved).to be_a(AuroraDsql::Pg::ResolvedConfig)
    end

    it "does not mutate original config" do
      config = described_class.new(host: "ijsamhssbh36dopuigphknejb4", region: "us-east-1")
      config.resolve

      expect(config.host).to eq("ijsamhssbh36dopuigphknejb4")
    end
  end

  describe ".parse" do
    it "parses basic connection string" do
      config = described_class.parse("postgres://admin@mycluster.dsql.us-east-1.on.aws/postgres")

      expect(config.host).to eq("mycluster.dsql.us-east-1.on.aws")
      expect(config.user).to eq("admin")
      expect(config.database).to eq("postgres")
    end

    it "parses connection string with port" do
      config = described_class.parse("postgres://admin@mycluster.dsql.us-east-1.on.aws:5433/mydb")

      expect(config.port).to eq(5433)
      expect(config.database).to eq("mydb")
    end

    it "parses DSQL-specific query params" do
      config = described_class.parse(
        "postgres://admin@mycluster.dsql.us-east-1.on.aws/postgres?region=eu-west-1&profile=dev&tokenDurationSecs=300"
      )

      expect(config.region).to eq("eu-west-1")
      expect(config.profile).to eq("dev")
      expect(config.token_duration).to eq(300)
    end

    it "handles postgresql:// scheme" do
      config = described_class.parse("postgresql://admin@mycluster.dsql.us-east-1.on.aws/postgres")

      expect(config.host).to eq("mycluster.dsql.us-east-1.on.aws")
    end
  end
end

RSpec.describe AuroraDsql::Pg::ResolvedConfig do
  let(:resolved) do
    AuroraDsql::Pg::ResolvedConfig.new(
      host: "mycluster.dsql.us-east-1.on.aws",
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

  describe "#to_pg_params" do
    before do
      # Mock PG.library_version
      stub_const("PG", Class.new do
        def self.library_version
          170000
        end
      end)
    end

    it "returns hash with required pg connection params" do
      params = resolved.to_pg_params(password: "token123")

      expect(params[:host]).to eq("mycluster.dsql.us-east-1.on.aws")
      expect(params[:port]).to eq(5432)
      expect(params[:user]).to eq("admin")
      expect(params[:dbname]).to eq("postgres")
      expect(params[:password]).to eq("token123")
    end

    it "sets sslmode to verify-full" do
      params = resolved.to_pg_params(password: "token123")

      expect(params[:sslmode]).to eq("verify-full")
    end

    it "sets sslnegotiation to direct for libpq >= 17" do
      params = resolved.to_pg_params(password: "token123")

      expect(params[:sslnegotiation]).to eq("direct")
    end

    it "omits sslnegotiation for libpq < 17" do
      stub_const("PG", Class.new do
        def self.library_version
          160000
        end
      end)

      params = resolved.to_pg_params(password: "token123")

      expect(params).not_to have_key(:sslnegotiation)
    end

    it "sets application_name using build_application_name" do
      params = resolved.to_pg_params(password: "token123")

      expect(params[:application_name]).to eq("aurora-dsql-ruby-pg/1.0.0")
    end

    it "includes ORM prefix in application_name" do
      resolved_with_orm = AuroraDsql::Pg::ResolvedConfig.new(
        host: "mycluster.dsql.us-east-1.on.aws",
        region: "us-east-1",
        user: "admin",
        database: "postgres",
        port: 5432,
        profile: nil,
        token_duration: 900,
        credentials_provider: nil,
        max_lifetime: 3300,
        pool_size: 5,
        application_name: "rails"
      )

      params = resolved_with_orm.to_pg_params(password: "token123")

      expect(params[:application_name]).to eq("rails:aurora-dsql-ruby-pg/1.0.0")
    end
  end
end
