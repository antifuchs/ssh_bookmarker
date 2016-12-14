pub mod ssh_config;
pub mod known_hosts;

#[macro_use] extern crate quick_error;

use std::path::{Path, PathBuf};
use std::io;
use std::io::BufReader;
use std::io::prelude::*;
use std::fs::File;

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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Host {
    name: String,
    protocol: String,
}

impl Host {
    pub fn new(name: &str, protocol: &str) -> Host {
        Host{
            name: name.to_string(),
            protocol: protocol.to_string(),
        }
    }

    pub fn named(name: &str) -> Host {
        Host{
            name: name.to_string(),
            protocol: "ssh".to_string(),
        }
    }

    pub fn write_bookmark(&self, dir: &Path) -> Result<(), Error> {
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

    pub fn ineligible(&self) -> bool {
        self.name.contains('*') || self.name.contains('?')
    }
}

pub trait ConfigFile {
    fn pathname<'a>(&'a self) -> &'a Path;

    fn parse_entries<R: BufRead>(&self, r: R) -> Result<Vec<Host>, Error>;

    fn entries(&self) -> Result<Vec<Host>, Error> {
        let f = try!(File::open(self.pathname()));
        let file = BufReader::new(&f);
        self.parse_entries(file)
    }
}

pub fn process<T>(pathnames: Vec<String>) -> Result<Vec<Host>, Error>
    where T: From<PathBuf> + ConfigFile {

    let mut hosts: Vec<Host> = vec![];
    for pn in pathnames {
        let path = PathBuf::from(pn);
        let file = T::from(path);
        hosts.extend(try!(file.entries()));
    }
    Ok(hosts)
}

#[test]
fn test_host_creation() {
    let ohai = Host::named("ohai");
    assert_eq!(ohai.name, "ohai");
    assert_eq!(ohai.protocol, "ssh");

    let mosh_ohai = Host::new("ohai", "mosh");
    assert_eq!(mosh_ohai.name, "ohai");
    assert_eq!(mosh_ohai.protocol, "mosh");
}

#[test]
fn test_host_eligibility() {
    assert_eq!(Host::named("foo*.oink.example.com").ineligible(), true);
    assert_eq!(Host::named("*").ineligible(), true);

    assert_eq!(Host::named("foobar.oink.example.com").ineligible(), false);
}
