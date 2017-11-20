#!/usr/bin/env ruby
#
# LaunchBar Action Script
#
require 'set'
require 'logger'
require 'json'

module SSHBookmarker
  class Parser
    def logger=(logger)
      @logger = logger
    end

    def logger
      @logger ||= begin
                    Logger.new(STDERR)
                  end
    end

    def protocol_override=(block)
      @protocol_override = block
    end

    DEFAULT_CONFIG_FILES=['/etc/ssh/ssh_config', File.expand_path('~/.ssh/config')]
    DEFAULT_KNOWN_HOSTS_FILES=['/etc/ssh/known_hosts', File.expand_path('~/.ssh/known_hosts')]

    def process_files(ssh_config_files=DEFAULT_CONFIG_FILES, known_host_files=DEFAULT_KNOWN_HOSTS_FILES)
      ssh_config_files.each do |path|
        if File.exists?(path)
          logger.info("Parsing SSH config file #{path}")
        else
          logger.info("Skipping missing SSH config file #{path}")
          next
        end
        parse_ssh_config(path) do |hostnames, url_scheme|
          if @protocol_override
            (@protocol_override.call(hostnames, url_scheme) || [url_scheme]).each do |scheme|
              hostnames.each { |hostname| make_webloc(hostname, nil, scheme) }
            end
          else
            hostnames.each { |hostname| make_webloc(hostname, nil, url_scheme) }
          end
        end
      end

      known_host_files.each do |path|
        if File.exists?(path)
          logger.info("Parsing known_hosts file #{path}")
        else
          logger.info("Skipping missing known_hosts file #{path}")
          next
        end
        parse_known_hosts_file(path) do |hostname, port|
          make_webloc(hostname, port)
        end
      end
    end

    def parse_known_hosts_file(path)
      File.open(path).each_line do |line|
        line = line.split(' ')[0]
        hosts = line.split(',')
        next if hosts.nil?
        hosts.each do |hostname|
          if ported_host_match = hostname.match(/\[(\S*[a-zA-Z]+\S*)\]:([0-9]+)/)
            host = ported_host_match[1]
            port = ported_host_match[2]
            yield(host, port)
          else
            yield([hostname]) unless hostname.match(/ /) || hostname.match(/^[\[]/)  || hostname.match(/^[0-9\.]+$/) || hostname.match(/^[a-f0-9\:]+(%.*)?$/)
          end
        end
      end
    rescue Errno::ENOENT => e
      puts "Can't open #{path}: #{e}" if $debug
    end

    def parse_ssh_config(path, include_path: File.dirname(path), already_included: Set.new(), &blk)
      logger.debug("Parsing #{path}")
      included_files = Set.new()
      File.open(path).each do |line|
        if line.match(/^\s*Host\s+([^#]+)\s*(#.+)?$/i)
          host_spec = $1
          url_schemes = extract_url_scheme($2)
          hosts = host_spec.split(/\s+/)
          url_schemes.each do |url_scheme|
            blk.call(hosts, nil, url_scheme) unless hosts.any?{ |hn| hn.match(/\*/) }
          end
        elsif line.match(/^\s*Include\s+([^#]+)/i)
          included_files << File.expand_path($1.chomp, include_path)
        end
      end
      (included_files - already_included).each do |f|
        already_included << f
        logger.debug("Found & will traverse included file #{f}")
        parse_ssh_config(f, include_path: include_path, already_included: already_included, &blk)
      end
    rescue Errno::ENOENT
    end

    def extract_url_scheme(scheme_comment)
      if scheme_comment && scheme_comment =~ /^#:(.*)$/
        $1.split(',')
      else
        ["ssh"]
      end
    end
  end
end

class Entry
  attr_accessor :host, :url_scheme
  def initialize(precision, position, host, url_scheme)
    @host = host
    @url_scheme = url_scheme
    @precision = precision
    @position = position
  end

  def eql?(other)
    other.host == self.host && other.url_scheme == self.url_scheme
  end

  def hash
    [host, url_scheme].hash
  end

  def to_h
    {
      title: "#{url_scheme}://#{host}",
      label: host,
      actionRunsInBackground: true,
      badge: url_scheme,
      icon: 'font-awesome:terminal',
    }
  end

  def relevance
    [@precision, @position]
  end
end

def main
  ## If you want to debug this script, this is a safe bet for STDERR redirection:
  # debug = File.open('/tmp/ssh_to.debug.txt', 'w')
  # debug.sync = true
  # STDERR.reopen(debug)

  seeking = ARGV[0]
  raise "Need an argument" unless seeking
  hosts = Set.new()
  position = 0

  p = SSHBookmarker::Parser.new
  SSHBookmarker::Parser::DEFAULT_CONFIG_FILES.each do |cfg|
    p.parse_ssh_config(cfg) do |hostnames, _, url_scheme|
      hostnames.each do |host|
        idx = host.index(seeking)
        next if idx.nil?

        hosts << Entry.new(idx, position, host, url_scheme)
        position+=1
      end
    end
  end
  SSHBookmarker::Parser::DEFAULT_KNOWN_HOSTS_FILES.each do |kh|
    p.parse_known_hosts_file(kh) do |host, port|
      idx = host.index(seeking)
      p.logger.debug("oh hai #{seeking}/#{host}, #{idx}")
      next if idx.nil?
      hosts << Entry.new(idx, position, host, 'ssh')
      position+=1
    end
  end
  result = hosts.to_a.sort_by(&:relevance).map(&:to_h).to_json
  p.logger.debug("Got hosts: #{result}")
  puts result
end

main if File.basename($0) == File.basename(__FILE__)
