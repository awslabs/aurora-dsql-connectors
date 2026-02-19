# frozen_string_literal: true

# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require_relative "lib/aurora_dsql/pg/version"

Gem::Specification.new do |spec|
  spec.name = "aurora-dsql-ruby-pg"
  spec.version = AuroraDsql::Pg::VERSION
  spec.authors = ["Amazon Web Services"]
  spec.email = ["aws-aurora-dsql-feedback@amazon.com"]

  spec.summary = "Aurora DSQL connector for Ruby pg gem"
  spec.description = "A connector that integrates IAM authentication for connecting Ruby applications to Amazon Aurora DSQL clusters using the pg gem"
  spec.homepage = "https://github.com/awslabs/aurora-dsql-connectors"
  spec.license = "Apache-2.0"
  spec.required_ruby_version = ">= 3.1.0"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = "https://github.com/awslabs/aurora-dsql-connectors/tree/main/ruby/pg"
  spec.metadata["changelog_uri"] = "https://github.com/awslabs/aurora-dsql-connectors/blob/main/ruby/pg/CHANGELOG.md"

  spec.files = Dir.glob("lib/**/*") + %w[README.md NOTICE]
  spec.require_paths = ["lib"]

  spec.add_dependency "pg", "~> 1.5"
  spec.add_dependency "aws-sdk-dsql", "~> 1.6"
  spec.add_dependency "connection_pool", "~> 2.4"

  spec.add_development_dependency "rspec", "~> 3.13"
  spec.add_development_dependency "webmock", "~> 3.0"
  spec.add_development_dependency "rake", "~> 13.0"
end
