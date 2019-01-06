use errors::*;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use {ConfigFile, Host};

pub struct KnownHosts {
    pathname: PathBuf,
}

impl From<PathBuf> for KnownHosts {
    fn from(path: PathBuf) -> KnownHosts {
        KnownHosts { pathname: path }
    }
}

impl ConfigFile for KnownHosts {
    fn pathname<'a>(&'a self) -> &'a Path {
        self.pathname.as_path()
    }

    fn parse_entries<R: BufRead>(&self, file: R) -> Result<Vec<Host>> {
        let mut hosts: Vec<Host> = vec![];
        for (lineno, maybe_line) in file.lines().enumerate() {
            let line = try!(maybe_line);
            hosts.extend(try!(process_entry(self.pathname(), lineno, &line)));
        }
        Ok(hosts)
    }
}

fn process_entry(pathname: &Path, lineno: usize, line: &str) -> Result<Vec<Host>> {
    let mut hosts: Vec<Host> = vec![];
    let line = line.trim();
    // Skip comments or blank lines:
    if line.len() == 0 || line.starts_with('#') {
        return Ok(vec![]);
    }

    let mut items = line.split_whitespace();
    let mut host_item = try!(items.next().ok_or(ErrorKind::KnownHostFormat(
        pathname.to_path_buf(),
        lineno,
        line.to_string()
    )));
    if host_item.starts_with('@') {
        // the hosts list is the next item if the first is a marker
        host_item = try!(items.next().ok_or(ErrorKind::KnownHostFormat(
            pathname.to_path_buf(),
            lineno,
            line.to_string()
        )));
    }
    if host_item.starts_with('|') {
        // hashed hosts can't be processed meaningfully, so don't do anything:
        return Ok(vec![]);
    }
    for host in host_item.split(',') {
        hosts.push(Host::named(host, pathname));
    }
    Ok(hosts)
}

#[test]
fn test_known_hosts_entry() {
    let from = Path::new("/dev/null");
    let no_hosts: Vec<Host> = vec![];
    let comment: Vec<Host> = process_entry(from, 0, "# Comments allowed at start of line").unwrap();
    assert_eq!(no_hosts, comment);

    let empty: Vec<Host> = process_entry(Path::new("/dev/null"), 0, "    ").unwrap();
    assert_eq!(no_hosts, empty);

    let multiple: Vec<Host> = process_entry(
        from,
        0,
        "closenet,closenet.example.net,192.0.2.53 1024 37 159...93 closenet.example.net ",
    )
    .unwrap();
    let expected_multiple: Vec<Host> = vec![
        Host::named("closenet", from),
        Host::named("closenet.example.net", from),
        Host::named("192.0.2.53", from),
    ];
    assert_eq!(multiple, expected_multiple);

    let annotated: Vec<Host> =
        process_entry(from, 0, "@revoked something ssh-rsa AAAAB5W...").unwrap();
    let expected_annotated: Vec<Host> = vec![Host::named("something", from)];
    assert_eq!(annotated, expected_annotated);

    let hashed: Vec<Host> = process_entry(
        from,
        0,
        "|1|JfKTdBh7rNbXkVAQCRp4OQoPfmI=|USECr3SWf1JUPsms5AqfD5QfxkM= ssh-rsa AAAAB5W...",
    )
    .unwrap();
    let expected_hashed: Vec<Host> = vec![];
    assert_eq!(hashed, expected_hashed);
}
