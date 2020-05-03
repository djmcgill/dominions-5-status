create table if not exists server_players_temp (
    server_id int NOT NULL REFERENCES game_servers(id),
    player_id int NOT NULL REFERENCES players(id),
    nation_id int NOT NULL,
);

insert into server_players_temp (server_id, player_id, nation_id)
    select server_id, player_id, nation_id from server_players;

drop table if exists server_players;

create table if not exists server_players_temp (
    server_id int NOT NULL REFERENCES game_servers(id),
    player_id int NOT NULL REFERENCES players(id),
    nation_id int,
    custom_nation_name text,

    CONSTRAINT server_nation_unique UNIQUE (server_id, nation_id),
    CHECK(
        (nation_id IS NULL AND custom_nation_name IS NOT NULL) OR
            (nation_id IS NOT NULL AND custom_nation_name IS NULL)
    )
);

insert into server_players (server_id, player_id, nation_id, custom_nation_name)
    select server_id, player_id, nation_id, NULL from server_players_temp;

drop table if exists server_players_temp;
