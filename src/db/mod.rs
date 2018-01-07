use failure::{err_msg,Error};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use serenity::model::UserId;
use typemap::Key;

use std::iter::FromIterator;

use model::game_server::GameServer;
use model::player::Player;

pub struct DbConnectionKey;
impl Key for DbConnectionKey {
    type Value = DbConnection;
}

pub struct DbConnection(Pool<SqliteConnectionManager>);
impl DbConnection {
    pub fn new(path: &String) -> Result<Self, Error> {
        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::new(manager)?;
        let db_conn = DbConnection(pool);
        db_conn.initialise()?;
        Ok(db_conn)
    }


    fn initialise(&self) -> Result<(), Error> {
        let conn = &*self.0.clone().get()?;
        conn.execute_batch("
            create table if not exists game_servers (
            id INTEGER NOT NULL PRIMARY KEY,
            address VARCHAR(255) NOT NULL,
            alias VARCHAR(255) NOT NULL,
            last_seen_turn int NOT NULL,
            CONSTRAINT server_address_unique UNIQUE (address)
            );

            create INDEX if not exists server_alias_index ON game_servers(alias);

            create table if not exists players (
            id INTEGER NOT NULL PRIMARY KEY,
            discord_user_id int NOT NULL,
            CONSTRAINT discord_user_id_unique UNIQUE(discord_user_id)
            );

            create table if not exists server_players (
            server_id int NOT NULL REFERENCES game_servers(id),
            player_id int NOT NULL REFERENCES players(id),
            nation_id int NOT NULL,
            CONSTRAINT server_nation_unique UNIQUE (server_id, nation_id)
            );"
        )?;
        Ok(())
    }

    pub fn insert_server_player(
            &self, 
            server_alias: &str, 
            player_user_id: &UserId, 
            nation_id: u32) -> Result<(), Error> {
        
        let conn = &*self.0.clone().get()?;
        conn.execute("INSERT INTO server_players (server_id, player_id, nation_id)
        SELECT g.id, p.id, ?1
        FROM game_servers g
        JOIN players p ON p.discord_user_id = ?2
        WHERE g.alias = ?3
        ", &[&nation_id, &(player_user_id.0 as i64), &server_alias])?;
        Ok(())
    }

    pub fn insert_game_server(&self, game_server: &GameServer) -> Result<(), Error> {
        let conn = &*self.0.clone().get()?;
        conn.execute(
            "INSERT INTO game_servers (address, alias, last_seen_turn)
            VALUES (?1, ?2, ?3)"
        , &[&game_server.address, &game_server.alias, &game_server.last_seen_turn])?;
        Ok(())
    }

    pub fn insert_player(&self, player: &Player) -> Result<(), Error> {
        let conn = &*self.0.clone().get()?;
        conn.execute(
            "INSERT INTO players (discord_user_id)
            SELECT ?1
            WHERE NOT EXISTS (select 1 from players where discord_user_id = ?1)"
        , &[&(player.discord_user_id.0 as i64)])?;
        Ok(())
    }

    pub fn retrieve_all_servers(&self) -> Result<Vec<(i32, GameServer)>, Error> {
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare("SELECT id, address, alias, last_seen_turn FROM game_servers")?;
        let foo = stmt.query_map(&[], |ref row| {
            let id = row.get(0);
            let server = GameServer {
                address: row.get(1),
                alias: row.get(2),
                last_seen_turn: row.get(3),
            };
            (id, server)
        })?;
        let iter = foo.map(|x| x.unwrap());

        Ok(Vec::from_iter(iter))
    }

    pub fn players_with_nations_for_game_alias(&self, game_alias: &str) -> Result<Vec<(i32, Player, usize)>, Error> {
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare(
            "SELECT p.id, p.discord_user_id, sp.nation_id
            FROM game_servers s
            JOIN server_players sp on sp.server_id = s.id
            JOIN players p on p.id = sp.player_id
            WHERE s.alias = ?1
            ")?;
        let foo = stmt.query_map(&[&game_alias], |ref row| {
            let id = row.get(0);
            let discord_user_id: i64 = row.get(1);
            let player = Player {
                discord_user_id: UserId(discord_user_id as u64),
            };
            let nation: i32 = row.get(2);
            (id, player, nation as usize)
        })?;
        let iter = foo.map(|x| x.unwrap());

        Ok(Vec::from_iter(iter))
    }

    pub fn game_for_alias(&self, game_alias: &str) -> Result<GameServer, Error> {
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare("SELECT id, address, alias, last_seen_turn FROM game_servers WHERE alias = ?1")?;
        let foo = stmt.query_map(&[&game_alias], |ref row| {
            let server = GameServer {
                address: row.get(1),
                alias: row.get(2),
                last_seen_turn: row.get(3),
            };
            server
        })?;
        let mut iter = foo.map(|x| x.unwrap());
        Ok(iter.next().ok_or(err_msg("could not find the game"))?)
    }

    pub fn update_game_with_possibly_new_turn(&self, game_alias: &str, current_turn: i32) -> Result<bool, Error> {
        let conn = &*self.0.clone().get()?;
        let rows = conn.execute(
            "UPDATE game_servers SET last_seen_turn = ?1 WHERE alias = ?2 AND last_seen_turn <> ?1", 
            &[&current_turn, &game_alias]
        )?;
        Ok(rows > 0)
    }

    pub fn remove_player_from_game(&self, game_alias: &str, user: UserId) -> Result<(), Error> {
        let conn = &*self.0.clone().get()?;
        conn.execute(
            "DELETE FROM server_players
            WHERE server_id IN
            (SELECT id from game_servers WHERE alias = ?1)
            AND player_id IN 
            (SELECT id from players WHERE discord_user_id = ?2)
            ",
            &[&game_alias, &(user.0 as i64)]
        )?;
        Ok(())
    }

    pub fn remove_server(&self, game_alias: &str) -> Result<(), Error> {
        let conn = &*self.0.clone().get()?;
        conn.execute(
            "DELETE FROM server_players
            WHERE server_id IN
            (SELECT id from game_servers WHERE alias = ?1)",
            &[&game_alias]
        )?;
        conn.execute(
            "DELETE FROM game_servers
            WHERE alias = ?1",
            &[&game_alias]
        )?;
        Ok(())
    }
}
