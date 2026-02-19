# frozen_string_literal: true

# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require_relative "aurora_dsql/pg/version"
require_relative "aurora_dsql/pg/util"
require_relative "aurora_dsql/pg/config"
require_relative "aurora_dsql/pg/token"
require_relative "aurora_dsql/pg/token_cache"
require_relative "aurora_dsql/pg/connection"

module AuroraDsql
  module Pg
    class Error < StandardError; end

    # Create a single connection to Aurora DSQL.
    #
    # @param config [String, Config, nil] connection string or Config object
    # @param options [Hash] configuration options
    # @return [Connection]
    def self.connect(config = nil, **options)
      Connection.connect(config, **options)
    end
  end
end
