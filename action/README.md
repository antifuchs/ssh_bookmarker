# A LaunchBar Action

This directory contains
a
[LaunchBar.app action](https://www.obdev.at/products/launchbar/actions.html) that
does ~effectively the same thing as the bookmark creator, except it
also works on High Sierra (and it's written in Ruby).

## Installation

You'll need [LaunchBar](https://www.obdev.at/products/launchbar/) installed to use this action.

Get the codesigned last release of this action, unzip it and then
double-click the `SSH.lbaction` bundle that got unpacked.

## Usage

To SSH to a host, open LaunchBar, type `SSH`, space, then the name of
the host and return.

To use hostnames completed from the known_hosts and ssh config parser, simply type a substring of that host's name, and then select the correct entry using ctrl-n/ctrl-p or the arrow keys, then hit return.

You can also SSH to the last host by typing `SSH` and hitting shift-return.

## Development
Convenient links:
* [Developer docs](https://developer.obdev.at/launchbar-developer-documentation/#/actions-overview)
* [Forum](https://forums.obdev.at/viewforum.php?f=24)
* [Releases](https://github.com/antifuchs/ssh_bookmarker/releases)
