extern crate rustc_serialize;
extern crate docopt;
extern crate ssh_bookmarker;

use std::path::Path;
use docopt::Docopt;

use ssh_bookmarker::{Host, ConfigFile};
use ssh_bookmarker::{ssh_config, known_hosts};

// use quick_error::ResultExt;

const USAGE: &'static str = "
Create SSH bookmarks from known_hosts and ssh_config files.

Usage:
  ssh_bookmarker create [-v...] [-c FILE...] [-k FILE...] <output>

Options:
  -h --help              Show this screen.
  -v --verbose           Log verbosely.
  -c --config FILE       ssh_config(5) file to read.
  -k --known-hosts FILE  known_hosts file to read.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_verbose: isize,
    cmd_create: bool,
    arg_output: String,
    flag_config: Vec<String>,
    flag_known_hosts: Vec<String>,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    if args.cmd_create {
        let mut hosts: Vec<Host> = args.flag_known_hosts.iter().flat_map(|kh| known_hosts::KnownHosts::new(Path::new(kh)).entries().unwrap()).collect();
        let config_hosts: Vec<Host> = args.flag_config.iter().flat_map(|kh| ssh_config::SSHConfigFile::new(Path::new(kh)).entries().unwrap()).collect();
        hosts.extend(config_hosts);
        hosts.sort();
        hosts.dedup();

        let output = Path::new(&args.arg_output);
        match std::fs::remove_dir_all(output) {
            Ok(()) => {},
            Err(e) => {
                if e.kind() != std::io::ErrorKind::NotFound {
                    panic!("Could not clear output directory: {}", e)
                }
            }
        }
        std::fs::create_dir_all(output).unwrap();
        for kh in hosts {
            if kh.ineligible() {
                continue;
            }
            kh.write_bookmark(output).unwrap();
        }
    } else {
        panic!("I don't understand what {:?} should do", args)
    }
}
