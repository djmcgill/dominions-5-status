A discord bot containing utilities for dominions 5

Thanks to https://github.com/culka/dom4status and http://www.cs.helsinki.fi/u/aitakang/dom3_serverformat_notes

To run:
* Create a discord bot account by following https://github.com/reactiflux/discord-irc/wiki/Creating-a-discord-bot-&-getting-a-token . The instructions there give you a bot with no permissions (DM only).
* Put the bot token in a file in the resources folder called "token".
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
* !notifications \{true, false\}: enable/disable turn notifications
* !lobby \{EA/MA/LA\} \<num\_players\> \[\<alias\>\]: create a lobby with no server
* !lobbies: list available lobbies
* !\{item, spell, unit, site, merc, event\} \<text\>: get dom5inspector search url
* !start \<address:port\> \[\<alias\>\]: register a started server for a lobby game
* !describe \"text\" \[\<alias\>\]: add a description to a lobby
* !help: display this text

TODO:
* more unit tests
* add more detail to turn notifications (who went AI, who maybe stalled)
* turn submitted colour coded?
* permissions for commands
* more embed responses
* modded nations for each game and/or show nation number (and sign up for it?)
* show registered nation during upload
* NAP helper
* db queries contain named arguments
* check invalid data???
    ```
    14:02:55 [DEBUG] dom5status::server: game name: ^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^
    ```
* BUG: possibly crash happens when turns and details happen at the same time? https://i.imgur.com/FioCpvD.png
* record last seen player state in the db, so can work out stalls and new AI/defeats

MAYBE:
* easier nation selection - acronyms, nicknames, etc
* edit pinned post instead of new
* channel alerts of new turns, who went AI, who stalled
* have docker volume/cache the crate registry (speed up builds)
* bot create game channels

how I run it

docker build -t dom-5-bot .

docker run -it -d --restart unless-stopped --volume /home/ec2-user/dominions-5-status/resources:/usr/src/myapp/resources dom-5-bot

if you run out of disk:
 docker system prune -a

a cheat to update the app without rebuilding the container:
docker exec -it \<container\> bash

then inside the container:
    git pull
    cargo install --force
    exit

then:
 docker restart \<container\>
