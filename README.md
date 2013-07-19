# Automatically create OS X bookmark files (.webloc) from your SSH config

This set of scripts allows you to automatically create bookmarks for
all hosts that your SSH knows about. The really cool part is that it
also includes a LaunchAgent script that re-runs this every time your
SSH config changes. Include the output dir in your LaunchBar /
QuickSilver / Alfred config, and you can SSH into hosts without even
opening Terminal (to type SSH)!

## Installation

You have to install it under the system ruby, so run:

    $ sudo env RBENV_VERSION=system RVM_VERSION=system gem install ssh_bookmarker

## Usage

You can either use `create-ssh-bookmarks` as a one-off script to
generate SSH bookmarks in a specific directory, or, if you have found
a set of command line args that works for you, use those same args
with `generate-ssh-bookmark-launchagent`.

## Contributing

1. Fork it
2. Create your feature branch (`git checkout -b my-new-feature`)
3. Commit your changes (`git commit -am 'Add some feature'`)
4. Push to the branch (`git push origin my-new-feature`)
5. Create new Pull Request
