# frozen_string_literal: true

# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

module AuroraDsql
  module Pg
    VERSION = "1.0.0"
    APPLICATION_NAME = "aurora-dsql-ruby-pg/#{VERSION}"

    # Build application_name with optional ORM prefix.
    # If orm_prefix is provided, returns "prefix:aurora-dsql-ruby-pg/VERSION"
    # Otherwise returns the base APPLICATION_NAME.
    #
    # @param orm_prefix [String, nil] optional ORM name (e.g., "rails", "sequel")
    # @return [String] the formatted application_name
    def self.build_application_name(orm_prefix = nil)
      return APPLICATION_NAME if orm_prefix.nil? || orm_prefix.to_s.strip.empty?

      "#{orm_prefix.to_s.strip}:#{APPLICATION_NAME}"
    end
  end
end
