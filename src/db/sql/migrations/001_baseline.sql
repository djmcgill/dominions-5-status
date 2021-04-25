
create table if not exists players (
    id INTEGER NOT NULL PRIMARY KEY,
    discord_user_id int NOT NULL,
    turn_notifications BOOLEAN NOT NULL,
    CONSTRAINT discord_user_id_unique UNIQUE(discord_user_id)
);

create table if not exists started_servers (
    id INTEGER NOT NULL PRIMARY KEY,
    address VARCHAR(255) NOT NULL,
    last_seen_turn int NOT NULL,
    CONSTRAINT server_address_unique UNIQUE (address)
);

create table if not exists lobbies (
    id INTEGER NOT NULL PRIMARY KEY,
    owner_id int NOT NULL REFERENCES players(id),
    player_count int NOT NULL,
    era int NOT NULL
);

create table if not exists game_servers (
    id INTEGER NOT NULL PRIMARY KEY,
    alias VARCHAR(255) NOT NULL,

    started_server_id int REFERENCES started_servers(id),
    lobby_id int REFERENCES lobbies(id),

    CONSTRAINT server_alias_unique UNIQUE (alias)
);

create table if not exists server_players (
    server_id int NOT NULL REFERENCES game_servers(id),
    player_id int NOT NULL REFERENCES players(id),
    nation_id int NOT NULL,

    CONSTRAINT server_nation_unique UNIQUE (server_id, nation_id)
);
