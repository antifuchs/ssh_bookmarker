extern crate docopt;
extern crate rustc_serialize;
extern crate ssh_bookmarker;
#[macro_use]
extern crate error_chain;

use docopt::Docopt;
use std::path::Path;

use ssh_bookmarker::launchagent;
use ssh_bookmarker::process;
use ssh_bookmarker::{known_hosts, ssh_config};
use ssh_bookmarker::{Condition, Conditions};

use ssh_bookmarker::errors::*;

// use quick_error::ResultExt;

const USAGE: &str = "
Create SSH bookmarks from known_hosts and ssh_config files.

Usage:
  ssh_bookmarker create [-v...] [-c FILE...] [-k FILE...] [-I SPEC...] [-X SPEC...] <output>
  ssh_bookmarker launchagent [-c FILE...] [-k FILE...] [-I SPEC...] [-X SPEC...] <output>
  ssh_bookmarker --help

Options:
  -h --help                Show this screen.
  -v --verbose             Log verbosely.
  -c --config FILE         ssh_config(5) file to read.
  -k --known-hosts FILE    known_hosts file to read.
  -I --include SPEC        In a given file, include only hosts matching the
                           given regex. SPEC format is \"FILE,REGEX\".
  -X --exclude SPEC        Like --include, exclude hosts matching the regex
                           from the file.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_verbose: isize,
    cmd_create: bool,
    cmd_launchagent: bool,
    arg_output: String,
    flag_config: Vec<String>,
    flag_known_hosts: Vec<String>,
    flag_include: Vec<String>,
    flag_exclude: Vec<String>,
}

quick_main!(run);
fn run() -> Result<()> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    if args.cmd_create {
        let conds = create_conditions(args.flag_include, args.flag_exclude)?;
        let mut hosts = process::<known_hosts::KnownHosts>(args.flag_known_hosts)?;
        hosts.extend(process::<ssh_config::SSHConfigFile>(args.flag_config)?);
        hosts.sort();
        hosts.dedup();

        let output = Path::new(&args.arg_output);
        std::fs::remove_dir_all(output)
            .chain_err(|| format!("Could not clear output directory {:?}", output))?;
        std::fs::create_dir_all(output)
            .chain_err(|| format!("Couldn't re-create output directory {:?}", output))?;

        for kh in hosts {
            if kh.ineligible(&conds) {
                continue;
            }
            kh.write_bookmark(output)
                .chain_err(|| format!("Couldn't write bookmark {:?}", kh))?;
        }
        Ok(())
    } else if args.cmd_launchagent {
        println!(
            "{}",
            launchagent::create(
                &args.flag_config,
                &args.flag_known_hosts,
                &args.flag_include,
                &args.flag_exclude,
                &args.arg_output
            )?
        );
        Ok(())
    } else {
        bail!("Don't know what to do!");
    }
}

fn create_conditions(include: Vec<String>, exclude: Vec<String>) -> Result<Conditions> {
    let mut conds = Conditions::default();
    for inc in include.into_iter() {
        let (pn, cond) = Condition::include_from(&inc)?;
        conds.add(pn, cond);
    }
    for exc in exclude.into_iter() {
        let (pn, cond) = Condition::exclude_from(&exc)?;
        conds.add(pn, cond);
    }
    Ok(conds)
}
