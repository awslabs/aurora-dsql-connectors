# frozen_string_literal: true

# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require "rspec"
require_relative "../../lib/aurora_dsql_pg"
require_relative "../../lib/aurora_dsql/pg/occ_retry"

RSpec.describe AuroraDsql::Pg::OCCRetry do
  describe ".occ_error?" do
    it "returns true for OC000 mutation conflict" do
      error = StandardError.new("ERROR: OC000: transaction conflict")
      expect(described_class.occ_error?(error)).to be true
    end

    it "returns true for OC001 schema conflict" do
      error = StandardError.new("ERROR: OC001: schema conflict")
      expect(described_class.occ_error?(error)).to be true
    end

    it "returns true for SQLSTATE 40001" do
      # Simulate PG::Error with result
      mock_result = double("result")
      allow(mock_result).to receive(:error_field)
        .with(PG::Result::PG_DIAG_SQLSTATE)
        .and_return("40001")

      error = double("pg_error", message: "serialization failure", result: mock_result)
      allow(error).to receive(:respond_to?).with(:result).and_return(true)

      expect(described_class.occ_error?(error)).to be true
    end

    it "returns false for non-OCC error" do
      error = StandardError.new("connection refused")
      expect(described_class.occ_error?(error)).to be false
    end

    it "returns false for nil" do
      expect(described_class.occ_error?(nil)).to be false
    end
  end

  describe "DEFAULT_CONFIG" do
    it "has correct default values" do
      expect(described_class::DEFAULT_CONFIG[:max_retries]).to eq(3)
      expect(described_class::DEFAULT_CONFIG[:initial_wait]).to eq(0.1)
      expect(described_class::DEFAULT_CONFIG[:max_wait]).to eq(5.0)
      expect(described_class::DEFAULT_CONFIG[:multiplier]).to eq(2.0)
    end
  end

  describe ".with_retry" do
    let(:mock_pool) { double("pool") }
    let(:mock_conn) { double("conn") }

    before do
      allow(mock_conn).to receive(:transaction).and_yield
    end

    it "executes block successfully without retry" do
      allow(mock_pool).to receive(:with).and_yield(mock_conn)

      result = nil
      described_class.with_retry(mock_pool) do |conn|
        result = "success"
      end

      expect(result).to eq("success")
    end

    it "retries on OCC error" do
      call_count = 0
      allow(mock_pool).to receive(:with) do |&block|
        call_count += 1
        if call_count == 1
          raise StandardError.new("OC000: conflict")
        else
          block.call(mock_conn)
        end
      end

      # Stub sleep to avoid actual delays
      allow(described_class).to receive(:sleep)

      described_class.with_retry(mock_pool) { |_| }

      expect(call_count).to eq(2)
    end

    it "raises after max retries exceeded with last error included" do
      allow(mock_pool).to receive(:with) do
        raise StandardError.new("OC000: conflict")
      end

      allow(described_class).to receive(:sleep)

      expect {
        described_class.with_retry(mock_pool, max_retries: 2) { |_| }
      }.to raise_error(AuroraDsql::Pg::Error, /Max retries.*exceeded.*OC000: conflict/)
    end

    it "raises immediately for non-OCC error" do
      allow(mock_pool).to receive(:with) do
        raise StandardError.new("connection refused")
      end

      expect {
        described_class.with_retry(mock_pool) { |_| }
      }.to raise_error("connection refused")
    end
  end

  describe ".exec_with_retry" do
    let(:mock_pool) { double("pool") }
    let(:mock_conn) { double("conn") }

    it "executes SQL with retry" do
      allow(mock_pool).to receive(:with).and_yield(mock_conn)
      allow(mock_conn).to receive(:transaction).and_yield
      allow(mock_conn).to receive(:exec).with("CREATE TABLE test (id UUID)")

      described_class.exec_with_retry(mock_pool, "CREATE TABLE test (id UUID)")

      expect(mock_conn).to have_received(:exec).with("CREATE TABLE test (id UUID)")
    end
  end
end
