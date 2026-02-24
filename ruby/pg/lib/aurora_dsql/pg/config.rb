# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

require "uri"

module AuroraDsql
  module Pg
    # Configuration for Aurora DSQL connections.
    class Config
      DEFAULTS = {
        user: "admin",
        database: "postgres",
        port: 5432,
        max_lifetime: 55 * 60,      # 55 minutes in seconds
        token_duration: 15 * 60,    # 15 minutes in seconds
        pool_size: 5
      }.freeze

      attr_accessor :host, :region, :user, :database, :port,
                    :profile, :token_duration, :credentials_provider,
                    :max_lifetime, :pool_size,
                    :application_name, :logger

      def initialize(**options)
        @host = options[:host]
        @region = options[:region]
        @user = options[:user]
        @database = options[:database]
        @port = options[:port]
        @profile = options[:profile]
        @token_duration = options[:token_duration]
        @credentials_provider = options[:credentials_provider]
        @max_lifetime = options[:max_lifetime]
        @pool_size = options[:pool_size]
        @application_name = options[:application_name]
        @logger = options[:logger]
      end

      # Parse a connection string into a Config.
      VALID_SCHEMES = %w[postgres postgresql].freeze

      def self.parse(conn_string)
        uri = URI.parse(conn_string)

        unless VALID_SCHEMES.include?(uri.scheme)
          raise Error, "unsupported URI scheme '#{uri.scheme}', expected 'postgres' or 'postgresql'"
        end

        config = new(
          host: uri.host,
          user: uri.user,
          database: uri.path&.delete_prefix("/"),
          port: uri.port
        )

        # Parse query params for DSQL-specific options
        if uri.query
          params = URI.decode_www_form(uri.query).to_h
          config.region = params["region"] if params["region"]
          config.profile = params["profile"] if params["profile"]
          config.token_duration = params["tokenDurationSecs"].to_i if params["tokenDurationSecs"]
        end

        config
      end

      # Build a Config from various input types.
      def self.from(config = nil, **options)
        case config
        when String then parse(config)
        when Config then config
        when nil then new(**options)
        else new(**options.merge(config.to_h))
        end
      end

      # Resolve and validate config, returning an immutable ResolvedConfig.
      def resolve
        validate!

        resolved_host = @host
        resolved_region = @region

        # Handle cluster ID vs full hostname
        if Util.cluster_id?(@host)
          resolved_region ||= Util.region_from_env
          raise Error, "region is required when host is a cluster ID" unless resolved_region

          resolved_host = Util.build_hostname(@host, resolved_region)
        else
          resolved_region ||= begin
            Util.parse_region(@host)
          rescue ArgumentError
            nil
          end
          resolved_region ||= Util.region_from_env
          raise Error, "region is required: could not parse from hostname and not set" unless resolved_region
        end

        ResolvedConfig.new(
          host: resolved_host,
          region: resolved_region,
          user: @user || DEFAULTS[:user],
          database: @database || DEFAULTS[:database],
          port: @port || DEFAULTS[:port],
          profile: @profile,
          token_duration: @token_duration || DEFAULTS[:token_duration],
          credentials_provider: @credentials_provider,
          max_lifetime: @max_lifetime || DEFAULTS[:max_lifetime],
          pool_size: @pool_size || DEFAULTS[:pool_size],
          application_name: @application_name,
          logger: @logger
        ).freeze
      end

      private

      def validate!
        raise Error, "host is required" if @host.nil? || @host.empty?

        if @port
          raise Error, "port must be an integer, got #{@port.class}" unless @port.is_a?(Integer)
          raise Error, "port must be between 1 and 65535, got #{@port}" if @port < 1 || @port > 65_535
        end
      end
    end

    # Immutable resolved configuration ready for connection.
    ResolvedConfig = Struct.new(
      :host, :region, :user, :database, :port,
      :profile, :token_duration, :credentials_provider,
      :max_lifetime, :pool_size,
      :application_name, :logger,
      keyword_init: true
    ) do
      # Convert to pg connection parameters hash.
      def to_pg_params(password:)
        params = {
          host: host,
          port: port,
          user: user,
          dbname: database,
          password: password,
          sslmode: "verify-full",
          application_name: AuroraDsql::Pg.build_application_name(application_name)
        }

        # Direct SSL negotiation (libpq 17+)
        params[:sslnegotiation] = "direct" if PG.library_version >= 170_000

        params
      end
    end
  end
end
