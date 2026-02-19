# frozen_string_literal: true

# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require "rspec"
require_relative "../../lib/aurora_dsql_pg"
require_relative "../../lib/aurora_dsql/pg/token"

RSpec.describe AuroraDsql::Pg::Token do
  let(:mock_credentials) { double("credentials") }
  let(:mock_token_generator) { double("token_generator") }

  before do
    allow(Aws::DSQL::AuthTokenGenerator).to receive(:new).and_return(mock_token_generator)
  end

  describe ".generate" do
    it "calls generate_db_connect_admin_auth_token for admin user" do
      expect(mock_token_generator).to receive(:generate_db_connect_admin_auth_token)
        .with(endpoint: "cluster.dsql.us-east-1.on.aws", region: "us-east-1", expires_in: 900)
        .and_return("admin-token-123")

      token = described_class.generate(
        host: "cluster.dsql.us-east-1.on.aws",
        region: "us-east-1",
        user: "admin",
        credentials: mock_credentials,
        expires_in: 900
      )

      expect(token).to eq("admin-token-123")
    end

    it "calls generate_db_connect_auth_token for non-admin user" do
      expect(mock_token_generator).to receive(:generate_db_connect_auth_token)
        .with(endpoint: "cluster.dsql.us-east-1.on.aws", region: "us-east-1", expires_in: 900)
        .and_return("user-token-456")

      token = described_class.generate(
        host: "cluster.dsql.us-east-1.on.aws",
        region: "us-east-1",
        user: "myuser",
        credentials: mock_credentials,
        expires_in: 900
      )

      expect(token).to eq("user-token-456")
    end

    it "uses default expires_in of 15 minutes" do
      expect(mock_token_generator).to receive(:generate_db_connect_admin_auth_token)
        .with(endpoint: "cluster.dsql.us-east-1.on.aws", region: "us-east-1", expires_in: 15 * 60)
        .and_return("token")

      described_class.generate(
        host: "cluster.dsql.us-east-1.on.aws",
        region: "us-east-1",
        user: "admin",
        credentials: mock_credentials
      )
    end

    it "wraps AWS service errors in AuroraDsql::Pg::Error" do
      aws_error = Aws::Errors::ServiceError.new(nil, "Access denied")
      allow(mock_token_generator).to receive(:generate_db_connect_admin_auth_token)
        .and_raise(aws_error)

      expect do
        described_class.generate(
          host: "cluster.dsql.us-east-1.on.aws",
          region: "us-east-1",
          user: "admin",
          credentials: mock_credentials
        )
      end.to raise_error(AuroraDsql::Pg::Error, /Failed to generate authentication token: Access denied/)
    end
  end

  describe ".resolve_credentials" do
    it "returns SharedCredentials for profile" do
      mock_shared_creds = double("shared_credentials")
      allow(Aws::SharedCredentials).to receive(:new)
        .with(profile_name: "myprofile")
        .and_return(mock_shared_creds)

      result = described_class.resolve_credentials("myprofile")

      expect(result).to eq(mock_shared_creds)
    end

    it "returns default credential chain for nil profile" do
      mock_chain = double("credential_chain")
      mock_resolved = double("resolved_credentials")
      allow(Aws::CredentialProviderChain).to receive(:new).and_return(mock_chain)
      allow(mock_chain).to receive(:resolve).and_return(mock_resolved)

      result = described_class.resolve_credentials(nil)

      expect(result).to eq(mock_resolved)
    end

    it "returns default credential chain for empty profile string" do
      mock_chain = double("credential_chain")
      mock_resolved = double("resolved_credentials")
      allow(Aws::CredentialProviderChain).to receive(:new).and_return(mock_chain)
      allow(mock_chain).to receive(:resolve).and_return(mock_resolved)

      result = described_class.resolve_credentials("")

      expect(result).to eq(mock_resolved)
    end
  end
end
