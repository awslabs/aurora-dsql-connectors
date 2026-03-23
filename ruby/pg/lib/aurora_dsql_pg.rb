# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require_relative "aurora_dsql/pg/version"
require_relative "aurora_dsql/pg/util"
require_relative "aurora_dsql/pg/config"
require_relative "aurora_dsql/pg/token"
require_relative "aurora_dsql/pg/pool"
require_relative "aurora_dsql/pg/occ_retry"

module AuroraDsql
  module Pg
    class Error < StandardError; end

    # Create a single connection to Aurora DSQL.
    # Returns a raw PG::Connection — no wrapper.
    def self.connect(config = nil, **options)
      resolved = Config.from(config, **options).resolve
      token = Token.generate(
        host: resolved.host, region: resolved.region,
        user: resolved.user, credentials: resolved.credentials_provider,
        profile: resolved.profile, expires_in: resolved.token_duration
      )
      PG.connect(resolved.to_pg_params(password: token))
    end

    # Create a connection pool for Aurora DSQL.
    # Pass pool: { size: N, timeout: N } to configure ConnectionPool.
    def self.create_pool(config = nil, pool: {}, **options)
      Pool.create(config, pool: pool, **options)
    end
  end
end
