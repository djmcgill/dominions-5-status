[![CircleCI](https://circleci.com/gh/djmcgill/dominions-5-status/tree/master.svg?style=svg)](https://circleci.com/gh/djmcgill/dominions-5-status/tree/master)

# Dominions 5 Discord Bot
A discord bot containing utilities for dominions 5

Thanks to https://github.com/culka/dom4status and http://www.cs.helsinki.fi/u/aitakang/dom3_serverformat_notes

## Usage:
To run natively with Rust:
- Create a discord bot account by following https://github.com/reactiflux/discord-irc/wiki/Creating-a-discord-bot-&-getting-a-token . The instructions there give you a bot with no permissions (DM only).
- Put the bot token in a file in the resources folder called "token" (i.e. no file extension!). You can edit the place that it looks for the file in src/main.rs on line 45.
- To get it to speak in channels, follow the instructions at https://discordapi.com/permissions.html
- Then run with "cargo run --release". You need to install Rust to do this: https://www.rust-lang.org/en-US/ .
- The bot should now show as online in your server and "dom-5-bot is connected!" should show in the console.

To run in docker:
- Follow the first 3 steps about setting up the bot
- Make a folder on the host called "resources" containing the `token` file as before. This is also where the db file will be created.
- Run the command: `docker run -it -d --restart unless-stopped -v /home/dmcgill9071/dom-5-bot/resources:/usr/src/myapp/resources --log-opt max-size=10m --log-opt max-file=5 eu.gcr.io/dom-5-status/dom-5-status` except replace `/home/dmcgill9071/dom-5-bot/resources` with the location of your resource folder.


## Commands:
n.b. server alias is optional, defaults to channel name.
`<>` means an argument, `[]` means optional
- `!add <address:port> [<alias>]`:
    - save the dom5 server address
- `!list`:
    - return a list of the saved server addresses and aliases
- `!delete [<alias>]`:
    - remove the server address from the list
- `!details [<alias>]`:
    - return a list of the nations and their statuses in the game
- `!register nation_prefix [<alias>]`:
    - register yourself as a nation in a game. Tries to ignore case, punctuation etc.
- `!register-id nation_id [<alias>]`:
    - register yourself as a nation in a game using the id
- `!unregister [<alias>]`:
    - unregister yourself in a game
- `!turns`:
    - show all of the games you're in and their turn status
- `!notifications {true, false}`:
    - enable/disable turn notifications for you. Enabled by default.
- `!lobby {EA/MA/LA} <num_players> [<alias>]`:
    - create a lobby with no server
- `!lobbies`:
    - list available lobbies
- `!{item, spell, unit, site, merc, event} <text>`:
    - get dom5inspector search url
- `!start <address:port> [<alias>]`:
    - register a started server for a lobby game
- `!describe "text" [<alias>]`:
    - add a description to a lobby. Quotes required to avoid issues with spaces.
- `!unstart [<alias>]`:
    - turn a game back into a lobby, if you need to change address
- `!help`:
    - display this text

##TODO:
* more unit tests
* permissions for commands
* more embed responses
* db queries contain named arguments
* BUG: possibly crash happens when turns and details happen at the same time? https://i.imgur.com/FioCpvD.png

##MAYBE:
* easier nation selection - acronyms, nicknames, fuzzy search, etc
* edit pinned post instead of new
* register channel for notifications
* have docker volume/cache the crate registry (speed up builds)
* bot create game channels
* bot post samogging/stalls/deaths/ais in the channel with the same name as the alias
