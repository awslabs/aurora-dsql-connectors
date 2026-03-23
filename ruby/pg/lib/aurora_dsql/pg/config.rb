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
        token_duration: 15 * 60    # 15 minutes in seconds
      }.freeze

      attr_accessor :host, :region, :user, :database, :port,
                    :profile, :token_duration, :credentials_provider,
                    :max_lifetime, :application_name, :logger, :occ_max_retries

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
        @application_name = options[:application_name]
        @logger = options[:logger]
        @occ_max_retries = options[:occ_max_retries]
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
      # Accepts a connection String, a Config instance, nil (keyword args only),
      # or any object that responds to #to_h (e.g. a Hash).
      def self.from(config = nil, **options)
        case config
        when String then parse(config)
        when Config then config
        when nil then new(**options)
        else
          unless config.respond_to?(:to_h)
            raise ArgumentError,
                  "config must be a String, Config, Hash, or respond to #to_h, got #{config.class}"
          end
          new(**options.merge(config.to_h))
        end
      end

      # Resolve and validate config, returning an immutable ResolvedConfig.
      def resolve
        validate!

        resolved_host, resolved_region = resolve_host_and_region(@host, @region)

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
          application_name: @application_name,
          logger: @logger,
          occ_max_retries: @occ_max_retries
        ).freeze
      end

      private

      def resolve_host_and_region(host, region)
        if Util.cluster_id?(host)
          region ||= Util.region_from_env
          raise Error, "region is required when host is a cluster ID" unless region

          [Util.build_hostname(host, region), region]
        else
          region ||= begin
            Util.parse_region(host)
          rescue ArgumentError
            nil
          end
          region ||= Util.region_from_env
          raise Error, "region is required: could not parse from hostname and not set" unless region

          [host, region]
        end
      end

      def validate!
        raise Error, "host is required" if @host.nil? || @host.empty?

        if @occ_max_retries
          unless @occ_max_retries.is_a?(Integer) && @occ_max_retries > 0
            raise Error, "occ_max_retries must be a positive integer, got #{@occ_max_retries.inspect}"
          end
        end
      end
    end

    # Immutable resolved configuration ready for connection.
    ResolvedConfig = Struct.new(
      :host, :region, :user, :database, :port,
      :profile, :token_duration, :credentials_provider,
      :max_lifetime, :application_name, :logger, :occ_max_retries,
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
