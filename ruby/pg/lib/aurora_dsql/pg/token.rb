# frozen_string_literal: true

# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require "aws-sdk-dsql"

module AuroraDsql
  module Pg
    ADMIN_USER = "admin"

    # IAM token generation for Aurora DSQL.
    class Token
      # Generate an IAM authentication token.
      #
      # @param host [String] the DSQL endpoint
      # @param region [String] the AWS region
      # @param user [String] the database user
      # @param credentials [Aws::Credentials, nil] AWS credentials (uses default chain if nil)
      # @param profile [String, nil] AWS profile name
      # @param expires_in [Integer] token lifetime in seconds (default: 15 minutes)
      # @return [String] the IAM token
      # @raise [AuroraDsql::Pg::Error] if token generation fails
      def self.generate(host:, region:, user:, credentials: nil, profile: nil, expires_in: 15 * 60)
        credentials ||= resolve_credentials(profile)

        token_generator = Aws::DSQL::AuthTokenGenerator.new(credentials: credentials)
        params = { endpoint: host, region: region, expires_in: expires_in }

        begin
          if user == ADMIN_USER
            token_generator.generate_db_connect_admin_auth_token(params)
          else
            token_generator.generate_db_connect_auth_token(params)
          end
        rescue Aws::Errors::ServiceError => e
          raise AuroraDsql::Pg::Error, "Failed to generate authentication token: #{e.message}"
        end
      end

      # Resolve AWS credentials from profile or default chain.
      #
      # @param profile [String, nil] AWS profile name
      # @return [Aws::Credentials] resolved credentials
      def self.resolve_credentials(profile = nil)
        if profile && !profile.empty?
          Aws::SharedCredentials.new(profile_name: profile)
        else
          Aws::CredentialProviderChain.new.resolve
        end
      end
    end
  end
end
