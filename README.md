A discord bot containing utilities for dominions 5

Thanks to https://github.com/culka/dom4status and http://www.cs.helsinki.fi/u/aitakang/dom3_serverformat_notes

To run:
* Create a discord bot account by following https://github.com/reactiflux/discord-irc/wiki/Creating-a-discord-bot-&-getting-a-token . The instructions there give you a bot with no permissions (DM only).
* Put the bot token in a file at the directory root called "token".
* To get it to speak in channels, follow the instructions at https://discordapi.com/permissions.html
* Then run with "cargo run --release". You need to install Rust to do this: https://www.rust-lang.org/en-US/ .
* The bot should now show as online in your server and "dom-5-bot is connected!" should show in the console.

Commands (server alias is optional, defaults to channel name): 
* servers add \<address> \[\<alias\>\]: save the dom5 server address
* servers list: return a list of the saved server addresses and aliases
* servers remove \[\<alias\>\]: remove the server address from the list
* search {item, spell, unit, site, merc, event} \<text\>: get dom5inspector search url
* game_name \[\<alias\>\]: display the ingame name
* help: display this text
* nation_status \[\<alias\>\]: return a list of the nations and their statuses in the game
* ping: return \"pong\"

TODO:
* proper logging and error handling
* make subfunctions pure and testable
* remember players in each game - allow to register
* register for games that don't exist yet
* polling for status and deliver turn notifications
* start game notifications
* persist state in MSSQL
