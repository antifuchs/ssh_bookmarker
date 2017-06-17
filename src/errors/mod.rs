use std::path::PathBuf;

// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        Format(::regex::Error);
        Io(::std::io::Error);
    }
    errors {
        ConditionFormat(spec: String) {
            display("{} is not a valid condition spec: format is FILENAME,REGEX", spec)
        }
        KnownHostFormat(path: PathBuf, lineno: usize, line: String) {
            display("{} line {}: {:?}", path.to_str().unwrap_or("(unprintable path)"), lineno, line)
        }
        NameError(name: String, protocol: String) {
            display("{} with protocol {} would result in a bad filename", name, protocol)
        }
    }
}
