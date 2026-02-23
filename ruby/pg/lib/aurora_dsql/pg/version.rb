# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

module AuroraDsql
  module Pg
    VERSION = "1.0.0"
    APPLICATION_NAME = "aurora-dsql-ruby-pg/#{VERSION}"

    # Build application_name with optional ORM prefix.
    def self.build_application_name(orm_prefix = nil)
      return APPLICATION_NAME if orm_prefix.nil? || orm_prefix.to_s.strip.empty?

      "#{orm_prefix.to_s.strip}:#{APPLICATION_NAME}"
    end
  end
end
