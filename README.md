A discord bot containing utilities for dominions 5

Thanks to https://github.com/culka/dom4status and http://www.cs.helsinki.fi/u/aitakang/dom3_serverformat_notes

To run:
* Create a discord bot account by following https://github.com/reactiflux/discord-irc/wiki/Creating-a-discord-bot-&-getting-a-token . The instructions there give you a bot with no permissions (DM only).
* Put the bot token in a file at the directory root called "token".
* To get it to speak in channels, follow the instructions at https://discordapi.com/permissions.html
* Then run with "cargo run --release". You need to install Rust to do this: https://www.rust-lang.org/en-US/ .
* The bot should now show as online in your server and "dom-5-bot is connected!" should show in the console.

Commands:
* !item \<foo\>: search dom5inspector for that item and return a link to it.
* !spell \<foo\>: see above but for spells
* !game_name \<address\>: connect to the dom 5 server and return the game name
* !nation_status \<address\>: connect to the dom 5 server and return the nation statuses
* !ping: check that the bot is connected to the discord server and can speak
    

TODO:
* URL encode characters and handle spaces for the item and spell commands
* remember server addresses
* remember players in each game
* polling for status and deliver turn notifications
