# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

module AuroraDsql
  module Pg
    # Utility functions for host/region parsing.
    module Util
      # Pattern to extract region from DSQL hostname.
      # Matches: cluster.dsql.us-east-1.on.aws or cluster.dsql-suffix.us-east-1.on.aws
      REGION_PATTERN = /\.dsql[^.]*\.([^.]+)\.on\.aws\z/

      # Pattern for valid DSQL cluster IDs: 26 lowercase alphanumeric characters.
      CLUSTER_ID_PATTERN = /\A[a-z0-9]{26}\z/

      # Extract AWS region from a DSQL hostname.
      def self.parse_region(host)
        match = host&.match(REGION_PATTERN)
        raise ArgumentError, "Cannot parse region from hostname: #{host.inspect}" unless match

        match[1]
      end

      # Check if the given string is a cluster ID (not a full hostname).
      def self.cluster_id?(host)
        return false if host.nil? || host.empty? || host.include?(".")

        CLUSTER_ID_PATTERN.match?(host)
      end

      # Build a full DSQL hostname from cluster ID and region.
      def self.build_hostname(cluster_id, region)
        "#{cluster_id}.dsql.#{region}.on.aws"
      end

      # Get AWS region from environment variables.
      def self.region_from_env
        ENV["AWS_REGION"] || ENV["AWS_DEFAULT_REGION"]
      end
    end
  end
end
