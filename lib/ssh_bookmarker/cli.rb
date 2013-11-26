require 'logger'
require 'optparse'

module SSHBookmarker
  class CLI
    attr_accessor :hosts_files
    attr_accessor :config_files
    attr_accessor :dir

    def initialize
      @hosts_files = SSHBookmarker::Parser::DEFAULT_KNOWN_HOSTS_FILES
      @config_files = SSHBookmarker::Parser::DEFAULT_CONFIG_FILES

      @mosh_positive_domain_patterns = []
      @mosh_negative_domain_patterns = []

      @debug_level = Logger::WARN
    end

    attr_writer :banner
    def banner
      @banner || <<-USAGE
Usage: #{$0} [options] output_dir

This script generates SSH (and, optionally, mosh) bookmarks in
output_dir.

(Note that patterns can be either substring matches or regexes, if wrapped in //)
USAGE
    end

    def parse_options(args=ARGV)
      optparse = OptionParser.new do |opts|
        opts.banner = banner
        opts.on('-k', '--known_hosts=FILE', 'Add file to list of known hosts') do |file|
          @hosts_files << file
        end

        opts.on('-c', '--ssh_config=FILE', 'Add file to list of ssh config files') do |file|
          @config_files << file
        end

        opts.on('-m PATTERN', '--mosh=PATTERN', 'Emit a mosh bookmark for host names matching PATTERN') do |pattern|
          @mosh_positive_domain_patterns << to_match_expr(pattern)
        end

        opts.on('-M PATTERN', '--prevent-mosh=PATTERN', 'Prevent emitting a mosh bookmark for host names matching PATTERN') do |pattern|
          @mosh_negative_domain_patterns << to_match_expr(pattern)
        end

        opts.on('-v', '--verbose', 'More debug chunder') do
          @debug_level -= 1
        end
      end
      optparse.parse!(args)
      if args.length != 1
        $stderr.puts optparse
        exit 1
      end
      @dir = args.first
      self
    end

    def to_match_expr(str)
      if str.match(%r{\A/(.*)/\z})
        Regexp.new($1)
      else
        str
      end
    end

    def parser
      parser = SSHBookmarker::Parser.new(@dir)

      logger = Logger.new(STDERR)
      logger.level = @debug_level
      parser.logger = logger

      if @mosh_positive_domain_patterns.length > 0
        parser.protocol_override = proc do |hostnames, url_scheme|
          if @mosh_positive_domain_patterns.find {|pattern| hostnames.any?{ |hn| hn.match(pattern) }} &&
              !@mosh_negative_domain_patterns.find {|pattern| hostnames.any?{ |hn| hn.match(pattern) }}
            ['mosh']
          end
        end
      end

      parser
    end

    def run
      Dir[File.join(@dir, '*.webloc')].each {|f| File.unlink(f)}
      FileUtils.mkdir_p(@dir)
      parser.process_files(@config_files, @hosts_files)
    end
  end
end
