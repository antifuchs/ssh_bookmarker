require "ssh_bookmarker/version"
require "ssh_bookmarker/cli"
require 'logger'
require 'fileutils'

module SSHBookmarker
  class Parser
    def initialize(locations_dir)
      @locations_dir = locations_dir
    end

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
        parse_ssh_config(path) do |hostname, url_scheme|
          if @protocol_override
            (@protocol_override.call(hostname, url_scheme) || [url_scheme]).each do |scheme|
              make_webloc(hostname, nil, scheme)
            end
          else
            make_webloc(hostname, nil, url_scheme)
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
        hostname = line.split(',').each do |hostname|
          if ported_host_match = hostname.match(/\[(\S*[a-zA-Z]+\S*)\]:([0-9]+)/)
            host = ported_host_match[1]
            port = ported_host_match[2]
            yield(host, port)
          else
            yield(hostname) unless hostname.match(/ /) || hostname.match(/^[\[]/)  || hostname.match(/^[0-9\.]+$/) || hostname.match(/^[a-f0-9\:]+(%.*)?$/)
          end
        end
      end
    rescue Errno::ENOENT => e
      puts "Can't open #{path}: #{e}" if $debug
    end

    def parse_ssh_config(path)
      File.open(path).each do |line|
        if line.match /^\s*Host\s+([^#]+)\s*(#.+)?$/i
          host_spec = $1
          url_schemes = extract_url_scheme($2)
          hosts = host_spec.split(/\s+/)
          logger.debug("Got hosts #{hosts.inspect}")
          hosts.each do |host|
            url_schemes.each do |url_scheme|
              yield(host, nil, url_scheme) unless host.match /\*/
            end
          end
        end
      end
    rescue Errno::ENOENT => e
    end

    def make_webloc(hostname, port, url_scheme=nil)
      url_scheme ||= 'ssh'
      logger.debug "Making host entry for #{url_scheme}://#{hostname}:#{port}"
      loc_filename = File.join(@locations_dir, "#{hostname} (#{url_scheme}).webloc")
      begin
        File.open(loc_filename, 'w') do |file|
          file.write <<-XML
        <?xml version="1.0" encoding="UTF-8"?>
        <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
        <plist version="1.0">
        <dict>
            <key>URL</key>
            <string>#{url_scheme || 'ssh'}://#{hostname}#{port && ":#{port}"}</string>
        </dict>
        </plist>
      XML
        end
      rescue Exception => e
        logger.error "Can't write webloc file #{loc_filename} for host #{hostname}: #{e}"
      end
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
