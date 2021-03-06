extern crate regex;

use errors::*;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use {ConfigFile, Host};

pub struct SSHConfigFile {
    pathname: PathBuf,
}

impl From<PathBuf> for SSHConfigFile {
    fn from(path: PathBuf) -> SSHConfigFile {
        SSHConfigFile { pathname: path }
    }
}

impl ConfigFile for SSHConfigFile {
    fn pathname(&self) -> &Path {
        self.pathname.as_path()
    }

    fn parse_entries<R: BufRead>(&self, file: R) -> Result<Vec<Host>> {
        let mut hosts: Vec<Host> = vec![];
        for maybe_line in file.lines() {
            let line = try!(maybe_line);

            let line = line.trim();
            // Skip comments or blank lines:
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut protocols: Vec<&str> = vec!["ssh"];
            let annotated: Vec<&str> = line.split("#:").collect();
            if annotated.len() > 1 {
                protocols = annotated[1].split(',').collect();
            }

            if annotated[0].to_lowercase().starts_with("host") {
                let host_entries: Vec<&str> = annotated[0].split_whitespace().skip(1).collect();
                for proto in protocols.iter() {
                    hosts.extend(
                        host_entries
                            .as_slice()
                            .iter()
                            .map(|name| Host::new(name, proto, self.pathname())),
                    )
                }
            }
        }
        Ok(hosts)
    }
}

#[test]
fn test_ssh_config() {
    let c = SSHConfigFile::from(PathBuf::from("/tmp"));
    assert_eq!(c.pathname(), Path::new("/tmp"));
}
