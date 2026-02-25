# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

module AuroraDsql
  module Pg
    # Thread-safe cache for IAM authentication tokens.
    # Refreshes proactively at 80% of token lifetime.
    class TokenCache
      # Refresh at 80% of token lifetime (20% buffer before expiry).
      REFRESH_BUFFER_PERCENTAGE = 0.2

      CacheKey = Struct.new(:host, :region, :user, :duration, keyword_init: true)
      CachedToken = Struct.new(:token, :generated_at, :expires_at, keyword_init: true)

      # Create a new token cache.
      def initialize(credentials_provider: nil, profile: nil)
        @credentials = credentials_provider || Token.resolve_credentials(profile)
        @cache = {}
        @mutex = Mutex.new
      end

      # Get a token, using cache if available and not expiring soon.
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
