# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require "aws-sdk-dsql"

module AuroraDsql
  module Pg
    # IAM token generation for Aurora DSQL.
    class Token
      ADMIN_USER = "admin"
      # Generate an IAM authentication token.
      def self.generate(host:, region:, user:, credentials: nil, profile: nil, expires_in: 15 * 60)
        credentials ||= resolve_credentials(profile)

        token_generator = Aws::DSQL::AuthTokenGenerator.new(credentials: credentials)
        params = { endpoint: host, region: region, expires_in: expires_in }

        begin
          if user == ADMIN_USER
            token_generator.generate_db_connect_admin_auth_token(**params)
          else
            token_generator.generate_db_connect_auth_token(**params)
          end
        rescue Aws::Errors::ServiceError => e
          raise AuroraDsql::Pg::Error, "Failed to generate authentication token: #{e.message}"
        end
      end

      # Resolve AWS credentials from profile or default chain.
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
