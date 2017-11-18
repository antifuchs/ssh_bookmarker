// LaunchBar Action Script

function run(argument) {
    if (argument == undefined) {
        // Inform the user that there was no argument
        LaunchBar.alert('No argument was passed to the action');
    } else {
        var url = 'ssh://' + argument;
        // Return a single item that describes the argument
        LaunchBar.openURL(url);
    }
}
