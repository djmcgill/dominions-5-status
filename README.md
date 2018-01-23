A discord bot containing utilities for dominions 5

Thanks to https://github.com/culka/dom4status and http://www.cs.helsinki.fi/u/aitakang/dom3_serverformat_notes

To run:
* Create a discord bot account by following https://github.com/reactiflux/discord-irc/wiki/Creating-a-discord-bot-&-getting-a-token . The instructions there give you a bot with no permissions (DM only).
* Put the bot token in a file at the directory root called "token".
* To get it to speak in channels, follow the instructions at https://discordapi.com/permissions.html
* Then run with "cargo run --release". You need to install Rust to do this: https://www.rust-lang.org/en-US/ .
* The bot should now show as online in your server and "dom-5-bot is connected!" should show in the console.

Commands (server alias is optional, defaults to channel name): 
* !add \<address:port\> \[\<alias\>\]: save the dom5 server address
* !list: return a list of the saved server addresses and aliases
* !delete \[\<alias\>\]: remove the server address from the list
* !details \[\<alias\>\]: return a list of the nations and their statuses in the game
* !register nation_prefix \[\<alias\>\]: register yourself as a nation in a game
* !unregister \[\<alias\>\]: unregister yourself in a game
* !turns: show all of the games you're in and their turn status
* !\{item, spell, unit, site, merc, event\} \<text\>: get dom5inspector search url
* !help: display this text

TODO:
* register for games that don't exist yet
* make subfunctions pure and testable
* error if nation registering is ambiguous
* help text in embed
* turns in embed
* turn submitted colour coded
* error handling for unable to connect to server
* players still show up in turns when AI or defeated

MAYBE:
* easier nation selection - acronyms, nicknames, etc
* edit pinned post instead of new
* have docker volume/cache the crate registry (speed up builds)

PRAGMA foreign_keys = OFF;
ALTER TABLE players RENAME TO tmp_players;

create table if not exists players (
id INTEGER NOT NULL PRIMARY KEY,
discord_user_id int NOT NULL,
turn_notifications BOOLEAN NOT NULL,
CONSTRAINT discord_user_id_unique UNIQUE(discord_user_id)
);

insert into players(id, discord_user_id, turn_notifications)
select id, discord_user_id, 1
from tmp_players;
PRAGMA foreign_keys = ON;
