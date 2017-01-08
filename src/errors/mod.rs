use std::path::PathBuf;

// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        Io(::std::io::Error);
    }
    errors {
        KnownHostFormat(path: PathBuf, lineno: usize, line: String) {
            display("{} line {}: {:?}", path.to_str().unwrap_or("(unprintable path)"), lineno, line)
        }
        NameError(name: String, protocol: String) {
            display("{} with protocol {} would result in a bad filename", name, protocol)
        }
    }
}
