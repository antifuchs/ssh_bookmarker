# coding: utf-8
lib = File.expand_path('../lib', __FILE__)
$LOAD_PATH.unshift(lib) unless $LOAD_PATH.include?(lib)
require 'ssh_bookmarker/version'

Gem::Specification.new do |spec|
  spec.name          = "ssh_bookmarker"
  spec.version       = SSHBookmarker::VERSION
  spec.authors       = ["Andreas Fuchs"]
  spec.email         = ["asf@boinkor.net"]
  spec.description   = %q{A tool that lets you automatically generate SSH bookmarks}
  spec.summary       = %q{This gem installs a script that lets you generate SSH bookmarks from the (non-wildcard) entries in your SSH config and known_hosts files. It supports Mosh, too!}
  spec.homepage      = ""
  spec.license       = "MIT"

  spec.files         = `git ls-files`.split($/)
  spec.executables   = spec.files.grep(%r{^bin/}) { |f| File.basename(f) }
  spec.test_files    = spec.files.grep(%r{^(test|spec|features)/})
  spec.require_paths = ["lib"]

  spec.add_development_dependency "bundler", "~> 1.3"
  spec.add_development_dependency "rake"
end
