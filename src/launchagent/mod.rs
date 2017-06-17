use std::env;
use ::errors::*;

pub fn create(configs: Vec<String>, known_hosts: Vec<String>, include: Vec<String>, exclude: Vec<String>, output: String) -> Result<String> {
    let curr_exe = env::current_exe().
        chain_err(|| "Couldn't determine the currently running program")?;
    let exe = curr_exe.
        to_str().ok_or("How did you get a non-unicodeable executable name?")?;
    Ok(create_for_exe(exe, configs, known_hosts, include, exclude, output.as_str()))
}

fn command_lineify<'a>(prefix: &'a str, args: &[&'a str]) -> Vec<&'a str> {
    let prefix_iter = vec![prefix].into_iter().cycle();
    prefix_iter.zip(args.into_iter()).flat_map(|(p, arg)| vec![p, arg]).collect()
}

fn plist_stringify(args: &[&str]) -> String {
    if args.len() > 0 {
        let mut interspersed = String::from("<string>");
        interspersed.push_str(args.join("</string><string>").as_str());
        interspersed.push_str("</string>");
        interspersed
    } else {
        String::from("")
    }
}

fn create_for_exe(exe: &str, configs: Vec<String>, known_hosts: Vec<String>, include: Vec<String>, exclude: Vec<String>, output: &str) -> String {
    let configs: Vec<&str> = configs.iter().map(|&ref s| s.as_str()).collect();
    let config_slice = configs.as_slice();
    let known_hosts: Vec<&str> = known_hosts.iter().map(|&ref s| s.as_str()).collect();
    let known_hosts_slice = known_hosts.as_slice();
    let include: Vec<&str> = include.iter().map(|&ref s| s.as_str()).collect();
    let include_slice = include.as_slice();
    let exclude: Vec<&str> = exclude.iter().map(|&ref s| s.as_str()).collect();
    let exclude_slice = exclude.as_slice();

    format!(r##"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key>
    <string>net.boinkor.ssh-bookmarker</string>
    <key>ProgramArguments</key>
    <array><string>{}</string><string>create</string>{}{}{}{}{}</array>
    <key>QueueDirectories</key>
    <array/>
    <key>RunAtLoad</key>
    <true/>
    <key>StartCalendarInterval</key>
    <dict>
      <key>Hour</key>
      <integer>0</integer>
      <key>Minute</key>
      <integer>0</integer>
    </dict>
    <key>WatchPaths</key>
    <array>{}{}</array>
  </dict>
</plist>"##,
            exe,
            plist_stringify(command_lineify("-c", config_slice).as_slice()),
            plist_stringify(command_lineify("-k", known_hosts_slice).as_slice()),
            plist_stringify(command_lineify("-I", include_slice).as_slice()),
            plist_stringify(command_lineify("-X", exclude_slice).as_slice()),
            plist_stringify(vec![output].as_slice()),
            plist_stringify(config_slice), plist_stringify(known_hosts_slice))
}

#[test]
fn test_command_lineify() {
    let expected: Vec<String> = ["-c", "foo", "-c", "bar"].iter().map(|&s| s.into()).collect();
    assert_eq!(command_lineify("-c", &["foo", "bar"]), expected);
}

#[test]
fn test_plist_stringify() {
    assert_eq!(plist_stringify(&["foo", "bar"]), "<string>foo</string><string>bar</string>".to_string());
    let empty: Vec<&str> = vec![];
    assert_eq!(plist_stringify(empty.as_slice()), "".to_string());
}

#[test]
fn test_create_for_exe() {
    assert_eq!(create_for_exe("program", vec!["/etc/ssh/ssh_config".to_string()], vec!["/etc/ssh/ssh_known_hosts".to_string()], vec!["foo:bar".to_string()], vec!["baz:qux".to_string()], "/tmp/foo"),
    r##"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key>
    <string>net.boinkor.ssh-bookmarker</string>
    <key>ProgramArguments</key>
    <array><string>program</string><string>create</string><string>-c</string><string>/etc/ssh/ssh_config</string><string>-k</string><string>/etc/ssh/ssh_known_hosts</string><string>-I</string><string>foo:bar</string><string>-X</string><string>baz:qux</string><string>/tmp/foo</string></array>
    <key>QueueDirectories</key>
    <array/>
    <key>RunAtLoad</key>
    <true/>
    <key>StartCalendarInterval</key>
    <dict>
      <key>Hour</key>
      <integer>0</integer>
      <key>Minute</key>
      <integer>0</integer>
    </dict>
    <key>WatchPaths</key>
    <array><string>/etc/ssh/ssh_config</string><string>/etc/ssh/ssh_known_hosts</string></array>
  </dict>
</plist>"##)
}
