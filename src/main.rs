extern crate rustc_serialize;
extern crate docopt;
#[macro_use] extern crate quick_error;

use std::path::{Path, PathBuf};
use std::io;
use std::io::BufReader;
use std::io::prelude::*;
use std::fs::File;

use docopt::Docopt;
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

#[derive(Debug, PartialEq)]
struct Host {
    name: String,
    protocol: String,
}

impl Host {
    fn new(name: &str, protocol: &str) -> Host {
        Host{
            name: name.to_string(),
            protocol: protocol.to_string(),
        }
    }

    fn named(name: &str) -> Host {
        Host{
            name: name.to_string(),
            protocol: "ssh".to_string(),
        }
    }

    fn write_bookmark(&self, dir: &Path) -> Result<(), Error> {
        let name = format!("{} ({}).webloc", self.name, self.protocol);
        let namepart = Path::new(&name);

        let mut path = PathBuf::from(dir);
        if namepart.is_absolute() {
            return Err(Error::NameError(self.name.to_string(), self.protocol.to_string()));
        }
        path.push(namepart);

        let mut bookmark_text = String::new();
        bookmark_text.push_str(self.protocol.as_str());
        bookmark_text.push_str("://");
        bookmark_text.push_str(self.name.as_str());
        let bookmark = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict><key>URL</key><string>{}</string></dict></plist>
"#,
                               bookmark_text);

        let mut f = try!(File::create(path));
        try!(f.write_all(bookmark.as_bytes()));
        Ok(())
    }

    fn ineligible(&self) -> bool {
        self.name.contains('*') || self.name.contains('?')
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        KnownHostFormat(path: PathBuf, lineno: usize, line: String) {
            // context(path: &'a Path, lineno: usize, line: &'a str)
            //     -> (path.to_path_buf(), lineno, line.to_string())
            display("{} line {}: {:?}", path.to_str().unwrap_or("(unprintable path)"), lineno, line)
        }
        NameError(name: String, protocol: String) {
            display("{} with protocol {} would result in a bad filename", name, protocol)
        }
        IO(err: io::Error) {
            from()
                cause(err)
                description("Couldn't read from file")
        }
    }
}

fn create_config_entries<'a>(pathname: &Path) -> Result<Vec<Host>, Error>{
    let f = try!(File::open(pathname));
    let file = BufReader::new(&f);

    let mut hosts: Vec<Host> = vec!();
    for maybe_line in file.lines() {
        let line = try!(maybe_line);

        let line = line.trim();
        // Skip comments or blank lines:
        if line.len() == 0 || line.starts_with('#') {
            continue;
        }

        let mut protocols: Vec<&str> = vec!["ssh"];
        let annotated: Vec<&str> = line.split("#:").collect();
        if annotated.len() > 1 {
            protocols = annotated[1].split(",").collect();
        }

        if annotated[0].to_lowercase().starts_with("host") {
            let host_entries: Vec<&str> = annotated[0].split_whitespace().skip(1).collect();
            for proto in protocols.iter() {
                hosts.extend(host_entries.as_slice().into_iter().map(|name| Host::new(name, proto)))
            }
        }
    }
    Ok(hosts)
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
        hosts.push(Host::named(host));
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
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());
    if args.cmd_create {
        let mut hosts: Vec<Host> = args.flag_known_hosts.iter().flat_map (|kh| {
            create_known_hosts_entries(Path::new(kh)).unwrap()
        }).collect();
        let config_hosts: Vec<Host> = args.flag_config.iter().flat_map (|conf| {
            create_config_entries(Path::new(conf)).unwrap()
        }).collect();
        hosts.extend(config_hosts);

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

#[test]
fn test_known_hosts_entry() {
    let no_hosts: Vec<Host> = vec![];
    let comment: Vec<Host> = process_entry(Path::new("/dev/null"), 0, "# Comments allowed at start of line").unwrap();
    assert_eq!(no_hosts, comment);

    let empty: Vec<Host> = process_entry(Path::new("/dev/null"), 0, "    ").unwrap();
    assert_eq!(no_hosts, empty);

    let multiple: Vec<Host> = process_entry(Path::new("/dev/null"), 0, "closenet,closenet.example.net,192.0.2.53 1024 37 159...93 closenet.example.net ").unwrap();
    let expected_multiple: Vec<Host> = vec![Host::named("closenet"), Host::named("closenet.example.net"), Host::named("192.0.2.53")];
    assert_eq!(multiple, expected_multiple);

    let annotated: Vec<Host> = process_entry(Path::new("/dev/null"), 0, "@revoked something ssh-rsa AAAAB5W...").unwrap();
    let expected_annotated: Vec<Host> = vec![Host::named("something")];
    assert_eq!(annotated, expected_annotated);

    let hashed: Vec<Host> = process_entry(Path::new("/dev/null"), 0, "|1|JfKTdBh7rNbXkVAQCRp4OQoPfmI=|USECr3SWf1JUPsms5AqfD5QfxkM= ssh-rsa AAAAB5W...").unwrap();
    let expected_hashed: Vec<Host> = vec![];
    assert_eq!(hashed, expected_hashed);
}
