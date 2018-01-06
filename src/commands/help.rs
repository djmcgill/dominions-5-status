command!(help(_context, message) {
    let _ = message.reply(
        "Commands (server alias is optional, defaults to channel name): \n\
        - servers add <address> [<alias>]: save the dom5 server address\n\
        - servers list: return a list of the saved server addresses and aliases\n\
        - servers remove [<alias>]: remove the server address from the list\n\
        - search {item, spell, unit, site, merc, event} <text>: get dom5inspector search url\n\
        - help: display this text\n\
        - servers details [<alias>]: return a list of the nations and their statuses in the game\n\
        - ping: return \"pong\"\n\
        - servers register nation_prefix [<alias>]: register yourself as a nation in a game\n\
        - servers unregister [<alias>]: unregister yourself in a game\n\
        - servers show_registered [<alias>]: display the users registered for a game"
    );
});
