# Automatically create OS X bookmark files (.webloc) from your SSH config

This program allows you to automatically create bookmarks for all
hosts that your SSH knows about. The really cool part is that it also
includes a LaunchAgent script that re-runs this every time your SSH
config changes. Include the output dir in your LaunchBar / QuickSilver
/ Alfred config, and you can SSH into hosts without even opening
Terminal (to type SSH)!

## High Sierra compatibility warning

*MacOS High Sierra does not allow you to use `ssh://` URLs in .webloc files!*

If you use this tool together with LaunchBar, I have created an Action
extension for it that you can find
in [the action subdir of this repo](action/)

## Installation

You'll need Rust 1.13 or later, and cargo.

### Installing from crates.io:

`cargo install ssh_bookmarker`

### Installing from git:

`cargo install ssh_bookmarker --git https://github.com/antifuchs/ssh_bookmarker.git`

## Usage

You can use `ssh_bookmarker create` as a one-off script to generate
SSH bookmarks in a specific directory. Specify SSH config file
locations with `-c` and known_host files with `-k` (There are no
defaults for file locations, so you'll have to specify them all
yourself).

### Watching your SSH config files

You can use `ssh_bookmarker launchagent` with the same options as you
would `create` to create a LaunchAgent definition. The agent will
watch all the SSH config and known_hosts files you specify, and invoke
the ssh_bookmarker program every time launchd detects changes. Here's
an example:

``` sh
$ mkdir -p ~/Library/LaunchAgents
$ ssh_bookmarker launchagent \
  -c /etc/ssh/ssh_config -c ~/.ssh/config \
  -k /etc/ssh/ssh_known_hosts -k ~/.ssh/known_hosts \
  ~/Library/"SSH Locations" > ~/Library/LaunchAgents/net.boinkor.ssh-bookmarker.plist

$ launchctl unload ~/Library/LaunchAgents/net.boinkor.ssh-bookmarker.plist ; launchctl load ~/Library/LaunchAgents/net.boinkor.ssh-bookmarker.plist
```

Now, all the files in `~/Library/SSH Locations` should be re-created
whenever `~/.ssh/config` or `/etc/ssh/ssh_known_hosts` or any of the
other files listed change.
