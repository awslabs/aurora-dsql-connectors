# frozen_string_literal: true

# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require "rspec"
require_relative "../../lib/aurora_dsql_pg"
require_relative "../../lib/aurora_dsql/pg/token"
require_relative "../../lib/aurora_dsql/pg/token_cache"

RSpec.describe AuroraDsql::Pg::TokenCache do
  let(:mock_credentials) { double("credentials") }

  before do
    allow(AuroraDsql::Pg::Token).to receive(:resolve_credentials).and_return(mock_credentials)
  end

  describe "#initialize" do
    it "uses provided credentials_provider instead of resolving" do
      custom_credentials = double("custom_credentials")
      cache = described_class.new(credentials_provider: custom_credentials)

      expect(AuroraDsql::Pg::Token).not_to receive(:resolve_credentials)
      expect(AuroraDsql::Pg::Token).to receive(:generate)
        .with(hash_including(credentials: custom_credentials))
        .and_return("token")

      cache.get_token(host: "cluster.dsql.us-east-1.on.aws", region: "us-east-1", user: "admin", duration: 900)
    end

    it "resolves credentials with profile when provided" do
      profile_credentials = double("profile_credentials")

      expect(AuroraDsql::Pg::Token).to receive(:resolve_credentials)
        .with("myprofile")
        .and_return(profile_credentials)

      cache = described_class.new(profile: "myprofile")

      expect(AuroraDsql::Pg::Token).to receive(:generate)
        .with(hash_including(credentials: profile_credentials))
        .and_return("token")

      cache.get_token(host: "cluster.dsql.us-east-1.on.aws", region: "us-east-1", user: "admin", duration: 900)
    end
  end

  describe "#get_token" do
    it "generates and caches a new token" do
      cache = described_class.new

      expect(AuroraDsql::Pg::Token).to receive(:generate)
        .with(
          host: "cluster.dsql.us-east-1.on.aws",
          region: "us-east-1",
          user: "admin",
          credentials: mock_credentials,
          expires_in: 900
        )
        .once
        .and_return("cached-token")

      # First call generates
      token1 = cache.get_token(
        host: "cluster.dsql.us-east-1.on.aws",
        region: "us-east-1",
        user: "admin",
        duration: 900
      )

      # Second call uses cache
      token2 = cache.get_token(
        host: "cluster.dsql.us-east-1.on.aws",
        region: "us-east-1",
        user: "admin",
        duration: 900
      )

      expect(token1).to eq("cached-token")
      expect(token2).to eq("cached-token")
    end

    it "uses separate cache keys for different hosts" do
      cache = described_class.new

      expect(AuroraDsql::Pg::Token).to receive(:generate).twice.and_return("token1", "token2")

      token1 = cache.get_token(host: "cluster1.dsql.us-east-1.on.aws", region: "us-east-1", user: "admin", duration: 900)
      token2 = cache.get_token(host: "cluster2.dsql.us-east-1.on.aws", region: "us-east-1", user: "admin", duration: 900)

      expect(token1).to eq("token1")
      expect(token2).to eq("token2")
    end

    it "uses separate cache keys for different users" do
      cache = described_class.new

      expect(AuroraDsql::Pg::Token).to receive(:generate).twice.and_return("admin-token", "user-token")

      token1 = cache.get_token(host: "cluster.dsql.us-east-1.on.aws", region: "us-east-1", user: "admin", duration: 900)
      token2 = cache.get_token(host: "cluster.dsql.us-east-1.on.aws", region: "us-east-1", user: "myuser", duration: 900)

      expect(token1).to eq("admin-token")
      expect(token2).to eq("user-token")
    end

    it "refreshes token at 80% of lifetime" do
      cache = described_class.new
      call_count = 0

      allow(AuroraDsql::Pg::Token).to receive(:generate) do
        call_count += 1
        "token-#{call_count}"
      end

      # First call at t=0
      token1 = cache.get_token(host: "cluster.dsql.us-east-1.on.aws", region: "us-east-1", user: "admin", duration: 100)
      expect(token1).to eq("token-1")

      # Simulate time passing to 81% of lifetime (81 seconds)
      allow(Time).to receive(:now).and_return(Time.now + 81)

      # Should refresh
      token2 = cache.get_token(host: "cluster.dsql.us-east-1.on.aws", region: "us-east-1", user: "admin", duration: 100)
      expect(token2).to eq("token-2")
    end

    it "does not refresh before 80% of lifetime" do
      cache = described_class.new

      expect(AuroraDsql::Pg::Token).to receive(:generate).once.and_return("token")

      cache.get_token(host: "cluster.dsql.us-east-1.on.aws", region: "us-east-1", user: "admin", duration: 100)

      # At 50% of lifetime - should still use cache
      allow(Time).to receive(:now).and_return(Time.now + 50)

      cache.get_token(host: "cluster.dsql.us-east-1.on.aws", region: "us-east-1", user: "admin", duration: 100)
    end
  end

  describe "#clear" do
    it "clears all cached tokens" do
      cache = described_class.new

      expect(AuroraDsql::Pg::Token).to receive(:generate).twice.and_return("token1", "token2")

      cache.get_token(host: "cluster.dsql.us-east-1.on.aws", region: "us-east-1", user: "admin", duration: 900)
      cache.clear
      cache.get_token(host: "cluster.dsql.us-east-1.on.aws", region: "us-east-1", user: "admin", duration: 900)
    end
  end

  describe "REFRESH_BUFFER_PERCENTAGE" do
    it "is 0.2 (20%)" do
      expect(AuroraDsql::Pg::TokenCache::REFRESH_BUFFER_PERCENTAGE).to eq(0.2)
    end
  end
end
