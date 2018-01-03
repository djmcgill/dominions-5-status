use serenity::model::UserId;
use rusqlite::{Connection, Error};
use std::iter::FromIterator;
use typemap::Key;

use model::player::Player;
use model::game_server::GameServer;

pub struct DbConnectionKey;
impl Key for DbConnectionKey {
    type Value = DbConnection;
}

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
pub struct DbConnection(pub Pool<SqliteConnectionManager>);
impl DbConnection {
    pub fn initialise(&self) -> Result<(), Error> {
        let conn = &*self.0.clone().get().unwrap();
        conn.execute("
            create table game_servers (
            id INTEGER NOT NULL PRIMARY KEY,
            address VARCHAR(255) NOT NULL,
            alias VARCHAR(255) NOT NULL,
            last_seen_turn int NOT NULL,
            CONSTRAINT server_address_unique UNIQUE (address)
            );

            create INDEX server_alias_index ON game_servers(alias);

            create table players (
            id INTEGER NOT NULL PRIMARY KEY,
            discord_user_id int NOT NULL,
            CONSTRAINT discord_user_id_unique UNIQUE(discord_user_id)
            );

            create table server_players (
            server_id int NOT NULL REFERENCES game_servers(id),
            player_id int NOT NULL REFERENCES players(id),
            CONSTRAINT server_player_unique UNIQUE (server_id, player_id)
            );"
        , &[])?;
        Ok(())
    }

    pub fn insert_game_server(&self, game_server: &GameServer) -> Result<(), Error> {
        let conn = &*self.0.clone().get().unwrap();
        conn.execute(
            "INSERT INTO game_servers (address, alias, last_seen_turn)
            VALUES (?1, ?2, ?3)"
        , &[&game_server.address, &game_server.alias, &game_server.last_seen_turn])?;
        Ok(())
    }

    pub fn insert_player(&self, player: &Player) -> Result<(), Error> {
        let conn = &*self.0.clone().get().unwrap();
        conn.execute(
            "INSERT INTO players (discord_user_id)
            VALUES (?1)"
        , &[&(player.discord_user_id.0 as i32)])?;
        Ok(())
    }

    pub fn register_player_for_game(&self, game_id: i32, player_id: i32) -> Result<(), Error> {
        let conn = &*self.0.clone().get().unwrap();
        conn.execute(
            "INSERT INTO server_players (server_id, player_id)
            VALUES (?1, ?2)"
        , &[&game_id, &player_id])?;
        Ok(())
    }

    pub fn retrieve_all_servers(&self) -> Result<Vec<(i32, GameServer)>, Error> {
        let conn = &*self.0.clone().get().unwrap();
        let mut stmt = conn.prepare("SELECT id, address, alias, last_seen_turn FROM game_servers").unwrap();
        let foo = stmt.query_map(&[], |ref row| {
            let id = row.get(0);
            let server = GameServer {
                address: row.get(1),
                alias: row.get(2),
                last_seen_turn: row.get(3),
            };
            (id, server)
        }).unwrap();
        let iter = foo.map(|x| x.unwrap());

        Ok(Vec::from_iter(iter))
    }

    pub fn players_for_game_alias(&self, game_alias: String) -> Result<Vec<(i32, Player)>, Error> {
        let conn = &*self.0.clone().get().unwrap();
        let mut stmt = conn.prepare(
            "SELECT p.id, p.discord_user_id
            FROM game_servers s
            JOIN server_players sp on sp.server_id = s.id
            JOIN players p on p.id = sp.player_id
            WHERE s.alias = ?1
            ").unwrap();
        let foo = stmt.query_map(&[&game_alias], |ref row| {
            let id = row.get(0);
            let discord_user_id: i32 = row.get(1);
            let player = Player {
                discord_user_id: UserId(discord_user_id as u64),
            };
            (id, player)
        }).unwrap();
        let iter = foo.map(|x| x.unwrap());

        Ok(Vec::from_iter(iter))
    }

    pub fn game_for_alias(&self, game_alias: String) -> Result<GameServer, Error> {
        let conn = &*self.0.clone().get().unwrap();
        let mut stmt = conn.prepare("SELECT id, address, alias, last_seen_turn FROM game_servers WHERE alias = ?1").unwrap();
        let foo = stmt.query_map(&[&game_alias], |ref row| {
            let server = GameServer {
                address: row.get(1),
                alias: row.get(2),
                last_seen_turn: row.get(3),
            };
            server
        }).unwrap();
        let mut iter = foo.map(|x| x.unwrap());
        Ok(iter.next().unwrap())
    }
}
