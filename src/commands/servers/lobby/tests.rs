use super::*;

/*
fn lobby_helper(
    db_conn: &DbConnection,
    era: Era,
    player_count: i32,
    alias: &String,
    author_id: UserId,
) -> Result<(), CommandError> {
*/

#[test]
fn add_lobby() {
    let db_conn = DbConnection::test();

    lobby_helper(&db_conn, Era::Early, 5, "foo", UserId(0)).unwrap();

    assert_eq!(db_conn.count_servers(), 1);
    assert_eq!(db_conn.count_lobby_state(), 1);
}

#[test]
fn add_two_lobbies() {
    let db_conn = DbConnection::test();

    lobby_helper(&db_conn, Era::Early, 5, "foo", UserId(4)).unwrap();
    lobby_helper(&db_conn, Era::Early, 5, "bar", UserId(4)).unwrap();

    assert_eq!(db_conn.count_servers(), 2);
    assert_eq!(db_conn.count_lobby_state(), 2);
}
/*

sqlite> .schema
CREATE TABLE server_players (
            server_id int NOT NULL REFERENCES game_servers(id),
            player_id int NOT NULL REFERENCES players(id),
            nation_id int NOT NULL,
            CONSTRAINT server_nation_unique UNIQUE (server_id, nation_id)
            );
CREATE TABLE started_servers (
    id INTEGER NOT NULL PRIMARY KEY,
    address VARCHAR(255) NOT NULL,
    last_seen_turn int NOT NULL,
    CONSTRAINT server_address_unique UNIQUE (address)
);
CREATE TABLE lobbies (
    id INTEGER NOT NULL PRIMARY KEY,
    owner_id int NOT NULL REFERENCES players(id),
    player_count int NOT NULL,
    era int NOT NULL
, description TEXT);
CREATE TABLE game_servers (
    id INTEGER NOT NULL PRIMARY KEY,
    alias VARCHAR(255) NOT NULL,

    started_server_id int REFERENCES started_servers(id),
    lobby_id int REFERENCES lobbies(id),

    CONSTRAINT server_alias_unique UNIQUE (alias)
);
CREATE TABLE players (
id INTEGER NOT NULL PRIMARY KEY,
discord_user_id int NOT NULL,
turn_notifications BOOLEAN NOT NULL,
CONSTRAINT discord_user_id_unique UNIQUE(discord_user_id)
);
CREATE TABLE __migrant_migrations(tag text unique);

*/
