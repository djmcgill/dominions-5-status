[![CircleCI](https://circleci.com/gh/djmcgill/dominions-5-status/tree/master.svg?style=svg)](https://circleci.com/gh/djmcgill/dominions-5-status/tree/master)

# Dominions Discord Bot
A discord bot containing utilities for dominions 5+6

Thanks to https://github.com/culka/dom4status and http://www.cs.helsinki.fi/u/aitakang/dom3_serverformat_notes

## Usage:
To run natively with Rust (the default option if you are just trying it out):
- Create a discord bot account by following https://github.com/reactiflux/discord-irc/wiki/Creating-a-discord-bot-&-getting-a-token . The instructions there give you a bot with no permissions (DM only).
- Put the bot token in a file in the resources folder called "token" (i.e. no file extension!). You can edit the place that it looks for the file in src/main.rs, ctrl+f for "resources/token".
- To get it to speak in channels, follow the instructions at https://discordapi.com/permissions.html
- Then run with "cargo run --release". You need to install Rust to do this: https://www.rust-lang.org/en-US/ .
- The bot should now show as online in your server and "dom-5-bot is connected!" should show in the console.

To run in docker:
- Follow the first 3 steps about setting up the bot
- Make a folder on the host called "resources" containing the `token` file as before. This is also where the db file will be created.
- Run the command: `docker run -it -d --restart unless-stopped -v /home/dmcgill9071/dom-5-bot/resources:/usr/src/myapp/resources --log-opt max-size=10m --log-opt max-file=5 eu.gcr.io/dom-5-status/dom-5-status` except replace `/home/dmcgill9071/dom-5-bot/resources` with the location of your resource folder.

## Dominions 6:
The bot now supports dominions 6 games. Both via direct connect and also via the new html status page. All new games are assumed to be dominions 6.
Old dominions 5 games are still supported via a new db column in the game_servers table, but there is currently no way to create a dom5 game until I (or a different contributor) adds in some kind of !add5 command.

## Slash commands:
Note that this bot now supports discord's [slash commands](https://discord.com/developers/docs/interactions/slash-commands) which look like:
![image](https://user-images.githubusercontent.com/1290757/120073845-9bb03100-c089-11eb-9604-880ca37670ee.png)

To do this, you need to also put a file in the resources folder called "application" containing the discord application's ID. Note that currently only "guild" i.e. server commands are supported, and not "global" commands, which I think would allow their use in the bot's DMs. All commands using "!" work as before.

## Anonymous mode:
If your game alias ends with "_anon" then the bot will refuse to show usernames in !details. Remember that you must register via DM!

## Commands:
n.b. server alias is optional, defaults to channel name.
`<>` means an argument, `[]` means optional
- `!add <address:port> [<alias>]` OR `!add <url for status page.html> [<alias>]`:
    - save the dom6 server address
- `!alias <old alias> [<new alias>]`
    - change the alias for a game server. Note, uses the discord channel name for the _new_ alias
- `!banish <user_id> [<alias>]`:
  - kick a player from a game, as if they had used !unregister
- `!list`:
    - return a list of the saved server addresses and aliases
- `!delete [<alias>]`:
    - remove the server address from the list
- `!details [<alias>]`:
    - return a list of the nations and their statuses in the game
- `!join/join-id/join-custom <nation> [<alias>]`:
    - join a game. Can use the name (tries to ignore case, punctuation etc), the ID, or some "totally custom text" in quotes.
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
- `!start <address:port> [<alias>]` OR `!start <url for status page.html> [<alias>]`:
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
* dom 6 inspector? or just rip that out

##MAYBE:
* easier nation selection - acronyms, nicknames, fuzzy search, etc
* edit pinned post instead of new
* register channel for notifications
* have docker volume/cache the crate registry (speed up builds)
* bot create game channels
* bot post samogging/stalls/deaths/ais in the channel with the same name as the alias
