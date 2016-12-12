#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate rustc_serialize;
extern crate docopt;
#[macro_use] extern crate quick_error;

use std::path::{Path, PathBuf};
use std::io;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;

// use quick_error::ResultExt;

docopt!(Args derive Debug, "
Create SSH bookmarks from known_hosts and ssh_config files.

Usage:
  ssh_bookmarker create [-v...] [-c FILE...] [-k FILE...] <output>

Options:
  -h --help              Show this screen.
  --version              Show version.
  -v --verbose           Log verbosely.
  -c --config FILE       ssh_config(5) file to read.
  -k --known-hosts FILE  known_hosts file to read.
");

#[derive(Debug, PartialEq)]
struct Host {
    name: String,
}

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        KnownHostFormat(path: PathBuf, lineno: usize, line: String) {
            // context(path: &'a Path, lineno: usize, line: &'a str)
            //     -> (path.to_path_buf(), lineno, line.to_string())
            display("{} line {}: {:?}", path.to_str().unwrap_or("(unprintable path)"), lineno, line)
        }
        IO(err: io::Error) {
            from()
                cause(err)
                description("Couldn't read from file")
        }
    }
}

fn create_config_entries<'a>(pathname: &Path) -> Result<Vec<Host>, Error>{
    Ok(vec!())
}

fn process_entry(pathname: &Path, lineno: usize, line: &str) -> Result<Vec<Host>, Error> {
    let mut hosts: Vec<Host> = vec!();
    let line = line.trim();
    // Skip comments or blank lines:
    if line.len() == 0 || line.starts_with('#') {
        return Ok(vec![]);
    }

    let mut items = line.split_whitespace();
    let mut host_item = try!(items.next().ok_or(Error::KnownHostFormat(pathname.to_path_buf(), lineno, line.to_string())));
    if host_item.starts_with('@') {
        // the hosts list is the next item if the first is a marker
        host_item = try!(items.next().ok_or(Error::KnownHostFormat(pathname.to_path_buf(), lineno, line.to_string())));
    }
    if host_item.starts_with('|') {
        // hashed hosts can't be processed meaningfully, so don't do anything:
        return Ok(vec![]);
    }
    for host in host_item.split(',') {
        hosts.push(Host{name: host.to_string()});
    }
    Ok(hosts)
}

fn create_known_hosts_entries<'a>(pathname: &Path) -> Result<Vec<Host>, Error>{
    let f = try!(File::open(pathname));
    let file = BufReader::new(&f);

    let mut hosts: Vec<Host> = vec!();
    for (lineno, maybe_line) in file.lines().enumerate() {
        let line = try!(maybe_line);
        hosts.extend(try!(process_entry(pathname, lineno, &line)));
    }
    Ok(hosts)
}

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    if args.cmd_create {
        let known_hosts: Vec<Host> = args.flag_known_hosts.iter().flat_map (|kh| {
            create_known_hosts_entries(Path::new(kh)).unwrap()
        }).collect();
        let config_hosts = args.flag_config.iter().flat_map (|conf| {
            create_config_entries(Path::new(conf)).unwrap()
        });
        println!("known hosts: {:?}", known_hosts);
        println!("ssh config: {:?}", config_hosts);
    } else {
        panic!("I don't understand what {:?} should do", args)
    }
}

#[test]
fn test_known_hosts_entry() {
    let no_hosts: Vec<Host> = vec![];
    let comment: Vec<Host> = process_entry(Path::new("/dev/null"), 0, "# Comments allowed at start of line").unwrap();
    assert_eq!(no_hosts, comment);

    let empty: Vec<Host> = process_entry(Path::new("/dev/null"), 0, "    ").unwrap();
    assert_eq!(no_hosts, empty);

    let multiple: Vec<Host> = process_entry(Path::new("/dev/null"), 0, "closenet,closenet.example.net,192.0.2.53 1024 37 159...93 closenet.example.net ").unwrap();
    let expected_multiple: Vec<Host> = vec![Host{name: "closenet".to_string()}, Host{name: "closenet.example.net".to_string()}, Host{name: "192.0.2.53".to_string()}];
    assert_eq!(multiple, expected_multiple);

    let annotated: Vec<Host> = process_entry(Path::new("/dev/null"), 0, "@revoked something ssh-rsa AAAAB5W...").unwrap();
    let expected_annotated: Vec<Host> = vec![Host{name: "something".to_string()}];
    assert_eq!(annotated, expected_annotated);

    let hashed: Vec<Host> = process_entry(Path::new("/dev/null"), 0, "|1|JfKTdBh7rNbXkVAQCRp4OQoPfmI=|USECr3SWf1JUPsms5AqfD5QfxkM= ssh-rsa AAAAB5W...").unwrap();
    let expected_hashed: Vec<Host> = vec![];
    assert_eq!(hashed, expected_hashed);
}
