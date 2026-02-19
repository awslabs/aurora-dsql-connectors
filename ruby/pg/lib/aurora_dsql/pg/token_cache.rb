# frozen_string_literal: true

# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

module AuroraDsql
  module Pg
    # Thread-safe cache for IAM authentication tokens.
    #
    # Caches tokens by (host, region, user, duration) and refreshes them
    # proactively when they reach 80% of their lifetime.
    #
    # Ruby's Mutex is exclusive-only (no RWMutex like Go), so double-checked
    # locking adds overhead without concurrency benefit. Single synchronize block
    # is the correct simplification here.
    class TokenCache
      # Refresh at 80% of token lifetime (20% buffer before expiry).
      REFRESH_BUFFER_PERCENTAGE = 0.2

      CacheKey = Struct.new(:host, :region, :user, :duration, keyword_init: true)
      CachedToken = Struct.new(:token, :generated_at, :expires_at, keyword_init: true)

      # Create a new token cache.
      #
      # @param credentials_provider [Aws::Credentials, nil] pre-resolved credentials
      # @param profile [String, nil] AWS profile name
      def initialize(credentials_provider: nil, profile: nil)
        @credentials = credentials_provider || Token.resolve_credentials(profile)
        @cache = {}
        @mutex = Mutex.new
      end

      # Get a token, using cache if available and not expiring soon.
      #
      # @param host [String] the DSQL endpoint
      # @param region [String] the AWS region
      # @param user [String] the database user
      # @param duration [Integer] token lifetime in seconds
      # @return [String] the IAM token
      def get_token(host:, region:, user:, duration:)
        key = CacheKey.new(host: host, region: region, user: user, duration: duration)

        @mutex.synchronize do
          cached = @cache[key]
          return cached.token if cached && !expiring_soon?(cached)

          token = Token.generate(
            host: host,
            region: region,
            user: user,
            credentials: @credentials,
            expires_in: duration
          )

          now = Time.now
          @cache[key] = CachedToken.new(
            token: token,
            generated_at: now,
            expires_at: now + duration
          )
          token
        end
      end

      # Clear all cached tokens.
      def clear
        @mutex.synchronize { @cache.clear }
      end

      private

      def expiring_soon?(cached)
        buffer = (cached.expires_at - cached.generated_at) * REFRESH_BUFFER_PERCENTAGE
        Time.now > (cached.expires_at - buffer)
      end
    end
  end
end
