command!(help(_context, message) {
    let _ = message.reply(
        "Commands (alias is optional, defaults to channel name): \n\
        - add_server <address> [<alias>]:\n\
        - game_name [<alias>]: display the ingame name\n\
        - help: display this text\n\
        - item <text>: get dom5inspector search url. The text cannot contain spaces\n\
        - list_servers: return a list of the saved server addresses and aliases\n\
        - nation_status [<alias>]: return a list of the nations and their statuses in the game\n\
        - ping: return \"pong\"\n\
        - remove_server [<alias>]: remove the server address from the list\n\
        - spell <text>: get dom5inspector search url. The text cannot contain spaces"
    );
});
