# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

module AuroraDsql
  module Pg
    VERSION = "1.0.0"
    APPLICATION_NAME = "aurora-dsql-ruby-pg/#{VERSION}"

    # Build application_name with optional ORM prefix.
    def self.build_application_name(orm_prefix = nil)
      prefix = orm_prefix.to_s.strip
      return APPLICATION_NAME if prefix.empty?

      "#{prefix}:#{APPLICATION_NAME}"
    end
  end
end
