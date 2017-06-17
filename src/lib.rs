pub mod ssh_config;
pub mod known_hosts;
pub mod launchagent;
pub mod errors;

#[macro_use] extern crate error_chain;

use std::path::{Path, PathBuf};
use std::io::BufReader;
use std::io::prelude::*;
use std::fs::File;

use errors::*;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Host {
    name: String,
    protocol: String,
    from: PathBuf,
}

impl Host {
    pub fn new(name: &str, protocol: &str, from: &Path) -> Host {
        Host{
            name: name.to_string(),
            protocol: protocol.to_string(),
            from: from.to_path_buf(),
        }
    }

    pub fn named(name: &str, from: &Path) -> Host {
        Host{
            name: name.to_string(),
            protocol: "ssh".to_string(),
            from: from.to_path_buf(),
        }
    }

    pub fn write_bookmark(&self, dir: &Path) -> Result<()> {
        let name = format!("{} ({}).webloc", self.name, self.protocol);
        let namepart = Path::new(&name);

        let mut path = PathBuf::from(dir);
        if namepart.is_absolute() {
            bail!(ErrorKind::NameError(self.name.to_string(), self.protocol.to_string()));
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

    fn parse_entries<R: BufRead>(&self, r: R) -> Result<Vec<Host>>;

    fn entries(&self) -> Result<Vec<Host>> {
        let f = try!(File::open(self.pathname()));
        let file = BufReader::new(&f);
        self.parse_entries(file)
    }
}

pub fn process<T>(pathnames: Vec<String>) -> Result<Vec<Host>>
    where T: From<PathBuf> + ConfigFile {

    let mut hosts: Vec<Host> = vec![];
    for pn in pathnames {
        let path = PathBuf::from(pn);
        let file = T::from(path);
        match file.entries() {
            Ok(entries) => hosts.extend(entries),
            Err(e) => println!("Could not read config file {:?} ({}), continuing", file.pathname(), e)
        }
    }
    Ok(hosts)
}

#[test]
fn test_host_creation() {
    let from = Path::new("/dev/null");
    let ohai = Host::named("ohai", from);
    assert_eq!(ohai.name, "ohai");
    assert_eq!(ohai.protocol, "ssh");
    assert_eq!(ohai.from, from);

    let mosh_ohai = Host::new("ohai", "mosh", from);
    assert_eq!(mosh_ohai.name, "ohai");
    assert_eq!(mosh_ohai.protocol, "mosh");
    assert_eq!(mosh_ohai.from, from);
}

#[test]
fn test_host_eligibility() {
    let from = Path::new("/dev/null");
    assert_eq!(Host::named("foo*.oink.example.com", from).ineligible(), true);
    assert_eq!(Host::named("*", from).ineligible(), true);

    assert_eq!(Host::named("foobar.oink.example.com", from).ineligible(), false);
}
