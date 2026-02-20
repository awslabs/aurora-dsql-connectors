# frozen_string_literal: true

# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require "rspec"
require_relative "../../lib/aurora_dsql_pg"
require_relative "../../lib/aurora_dsql/pg/util"

RSpec.describe AuroraDsql::Pg::Util do
  describe ".parse_region" do
    it "extracts region from standard DSQL hostname" do
      expect(described_class.parse_region("mycluster.dsql.us-east-1.on.aws")).to eq("us-east-1")
    end

    it "extracts region from hostname with suffix" do
      expect(described_class.parse_region("mycluster.dsql-gamma.eu-west-1.on.aws")).to eq("eu-west-1")
    end

    it "raises ArgumentError for non-DSQL hostname" do
      expect { described_class.parse_region("regular.postgres.host") }
        .to raise_error(ArgumentError, /Cannot parse region/)
    end

    it "raises ArgumentError for nil hostname" do
      expect { described_class.parse_region(nil) }
        .to raise_error(ArgumentError, /Cannot parse region/)
    end

    it "raises ArgumentError for empty hostname" do
      expect { described_class.parse_region("") }
        .to raise_error(ArgumentError, /Cannot parse region/)
    end
  end

  describe ".cluster_id?" do
    it "returns true for valid 26-char lowercase alphanumeric cluster ID" do
      expect(described_class.cluster_id?("ijsamhssbh36dopuigphknejb4")).to be true
    end

    it "returns false for hostname with dots" do
      expect(described_class.cluster_id?("cluster.dsql.us-east-1.on.aws")).to be false
    end

    it "returns false for nil" do
      expect(described_class.cluster_id?(nil)).to be false
    end

    it "returns false for empty string" do
      expect(described_class.cluster_id?("")).to be false
    end

    it "returns false for uppercase characters" do
      expect(described_class.cluster_id?("IJSAMHSSBH36DOPUIGPHKNEJB4")).to be false
    end

    it "returns false for wrong length" do
      expect(described_class.cluster_id?("tooshort")).to be false
    end
  end

  describe ".build_hostname" do
    it "constructs full DSQL hostname from cluster ID and region" do
      result = described_class.build_hostname("ijsamhssbh36dopuigphknejb4", "us-west-2")
      expect(result).to eq("ijsamhssbh36dopuigphknejb4.dsql.us-west-2.on.aws")
    end
  end

  describe ".region_from_env" do
    before do
      @original_region = ENV["AWS_REGION"]
      @original_default = ENV["AWS_DEFAULT_REGION"]
    end

    after do
      ENV["AWS_REGION"] = @original_region
      ENV["AWS_DEFAULT_REGION"] = @original_default
    end

    it "returns AWS_REGION if set" do
      ENV["AWS_REGION"] = "us-east-1"
      ENV["AWS_DEFAULT_REGION"] = "eu-west-1"
      expect(described_class.region_from_env).to eq("us-east-1")
    end

    it "falls back to AWS_DEFAULT_REGION" do
      ENV.delete("AWS_REGION")
      ENV["AWS_DEFAULT_REGION"] = "ap-northeast-1"
      expect(described_class.region_from_env).to eq("ap-northeast-1")
    end

    it "returns nil if neither set" do
      ENV.delete("AWS_REGION")
      ENV.delete("AWS_DEFAULT_REGION")
      expect(described_class.region_from_env).to be_nil
    end
  end
end
