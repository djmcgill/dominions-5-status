A discord bot containing utilities for dominions 5

Thanks to https://github.com/culka/dom4status and http://www.cs.helsinki.fi/u/aitakang/dom3_serverformat_notes

To run:
* Create a discord bot account by following https://github.com/reactiflux/discord-irc/wiki/Creating-a-discord-bot-&-getting-a-token . The instructions there give you a bot with no permissions (DM only).
* Put the bot token in a file at the directory root called "token".
* To get it to speak in channels, follow the instructions at https://discordapi.com/permissions.html
* Then run with "cargo run --release". You need to install Rust to do this: https://www.rust-lang.org/en-US/ .
* The bot should now show as online in your server and "dom-5-bot is connected!" should show in the console.

Commands (alias is optional, defaults to channel name):
* add_server \<address\> \[\<alias\>\]:
* game_name \[\<alias\>\]: display the ingame name
* help: display this text
* item \<text\>: get dom5inspector search url. The text cannot contain spaces
* list_servers: return a list of the saved server addresses and aliases
* nation_status \[\<alias\>\]: return a list of the nations and their statuses in the game
* ping: return \"pong\"
* remove_server \[\<alias\>\]: remove the server address from the list
* spell \<text\>: get dom5inspector search url. The text cannot contain spaces

TODO:
* display nation status better
* remember players in each game - allow to register
* register for games that don't exist yet
* polling for status and deliver turn notifications
* add other inspector searches
* persist state in MSSQL
* update README commands and help commands
