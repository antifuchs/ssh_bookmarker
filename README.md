# Automatically create OS X bookmark files (.webloc) from your SSH config

This program allows you to automatically create bookmarks for all
hosts that your SSH knows about. The really cool part is that it also
includes a LaunchAgent script that re-runs this every time your SSH
config changes. Include the output dir in your LaunchBar / QuickSilver
/ Alfred config, and you can SSH into hosts without even opening
Terminal (to type SSH)!

## Installation

You'll need Rust 1.10 or later, and cargo.

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
