use failure::{err_msg,Error};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use serenity::model::UserId;
use typemap::Key;
use num_traits::{FromPrimitive, ToPrimitive};

use model::*;
use model::enums::*;

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
        info!("db::initialise");
        let conn = &*self.0.clone().get()?;
        conn.execute_batch("
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
            );"
        )?;
        Ok(())
    }

    pub fn insert_server_player(
            &self, 
            server_alias: &str, 
            player_user_id: &UserId, 
            nation_id: u32) -> Result<(), Error> {
        info!("db::insert_server_player");
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
        info!("db::insert_game_server: {:?}", game_server);
        let conn = &*self.0.clone().get()?;
        match game_server.state {
            GameServerState::Lobby(ref lobby_state) => {
                // TODO: transaction
                conn.execute(
                    "INSERT INTO players (discord_user_id, turn_notifications)
                    SELECT ?1, 1
                    WHERE NOT EXISTS (select 1 from players where discord_user_id = ?1)"
                    , &[&(lobby_state.owner.0 as i64)]
                )?;

                conn.execute(
                    "INSERT INTO lobbies (owner_id, era, player_count)
                    SELECT id, ?1, ?3
                    FROM players
                    WHERE discord_user_id = ?2",
                    &[  &lobby_state.era.to_i32(),
                        &(lobby_state.owner.0 as i64),
                        &lobby_state.player_count,
                    ]
                )?;
                conn.execute(
                    "INSERT INTO game_servers (alias, lobby_id)
                    SELECT ?1, l.id
                    FROM lobbies l
                    LEFT JOIN game_servers s ON s.lobby_id = l.id
                    WHERE l.era = ?2
                    AND l.owner_id = (SELECT id FROM players where discord_user_id = ?3)
                    AND l.player_count = ?4
                    AND s.id IS NULL",
                    &[&game_server.alias,
                      &lobby_state.era.to_i32(),
                      &(lobby_state.owner.0 as i64),
                      &lobby_state.player_count,
                    ]
                )?;
                Ok(())                
            }
            GameServerState::StartedState(ref started_state, _) => {
                conn.execute(
                    "INSERT INTO started_servers (address, last_seen_turn)
                    VALUES(?1, ?2)"
                    , &[&started_state.address, &started_state.last_seen_turn]
                )?;
                conn.execute(
                    "INSERT INTO game_servers (alias, started_server_id)
                    SELECT ?1, id
                    FROM started_servers
                    WHERE address = ?2"
                    , &[&game_server.alias, &started_state.address]
                )?;
                Ok(())
            }
        }
    }

    pub fn insert_player(&self, player: &Player) -> Result<(), Error> {
        info!("db::insert_player");
        let conn = &*self.0.clone().get()?;
        conn.execute(
            "INSERT INTO players (discord_user_id, turn_notifications)
            SELECT ?1, ?2
            WHERE NOT EXISTS (select 1 from players where discord_user_id = ?1)"
        , &[&(player.discord_user_id.0 as i64), &player.turn_notifications])?;
        Ok(())
    }

    pub fn retrieve_all_servers(&self) -> Result<Vec<GameServer>, Error> {
        info!("db::retrieve_all_servers");
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare("
            SELECT g.alias, s.address, s.last_seen_turn, l.owner_id, l.era, l.player_count
            FROM game_servers g
            LEFT JOIN started_servers s ON s.id = g.started_server_id
            LEFT JOIN lobbies l ON l.id = g.lobby_id ")?;
        let foo = stmt.query_map(&[], |ref row| {
            let maybe_address: Option<String> = row.get(1);
            let maybe_last_seen_turn: Option<i32> = row.get(2);
            let alias: String = row.get(0);
            let maybe_owner: Option<i64> = row.get(3); 
            let maybe_era: Option<i32> = row.get(4);
            let maybe_player_count: Option<i32> = row.get(5);
            let server = make_game_server(
                alias,
                maybe_address,
                maybe_last_seen_turn,
                maybe_owner,
                maybe_era,
                maybe_player_count,
            ).unwrap();
            server
        })?;
        let vec = foo.collect::<Result<Vec<_>, _>>()?;
        Ok(vec)
    }

    pub fn players_with_nations_for_game_alias(&self, game_alias: &str) -> Result<Vec<(Player, usize)>, Error> {
        info!("players_with_nations_for_game_alias");
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare(
            "SELECT p.discord_user_id, sp.nation_id, p.turn_notifications
            FROM game_servers s
            JOIN server_players sp on sp.server_id = s.id
            JOIN players p on p.id = sp.player_id
            WHERE s.alias = ?1
            ")?;
        let foo = stmt.query_map(&[&game_alias], |ref row| {
            let discord_user_id: i64 = row.get(0);
            let player = Player {
                discord_user_id: UserId(discord_user_id as u64),
                turn_notifications: row.get(2),
            };
            let nation: i32 = row.get(1);
            (player, nation as usize)
        })?;
        let vec = foo.collect::<Result<Vec<_>, _>>()?;
        Ok(vec)
    }

    pub fn game_for_alias(&self, game_alias: &str) -> Result<GameServer, Error> {
        info!("db::game_for_alias");
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare("
            SELECT s.address, s.last_seen_turn, l.owner_id, l.era, l.player_count
            FROM game_servers g
            LEFT JOIN started_servers s ON s.id = g.started_server_id
            LEFT JOIN lobbies l ON l.id = g.lobby_id
            WHERE g.alias = ?1")?;
        let foo = stmt.query_map(&[&game_alias], |ref row| {
            let maybe_address: Option<String> = row.get(0);
            let maybe_last_seen_turn: Option<i32> = row.get(1);
            let maybe_owner: Option<i64> = row.get(2); 
            let maybe_era: Option<i32> = row.get(3);
            let maybe_player_count: Option<i32> = row.get(4);
            let server = make_game_server(
                game_alias.to_owned(),
                maybe_address,
                maybe_last_seen_turn,
                maybe_owner,
                maybe_era,
                maybe_player_count,
            ).unwrap();
            server
        })?;
        let vec = foo.collect::<Result<Vec<_>, _>>()?;
        if vec.len() == 1 {
            Ok(vec.into_iter().next().ok_or(err_msg("THIS SHOULD NEVER HAPPEN"))?) // TODO: *vomits*
        } else {
            Err(err_msg("could not find the game"))
        }
    }

    pub fn update_game_with_possibly_new_turn(&self, game_alias: &str, current_turn: i32) -> Result<bool, Error> {
        info!("db::update_game_with_possibly_new_turn");
        let conn = &*self.0.clone().get()?;
        let rows = conn.execute(
            "UPDATE started_servers
            SET last_seen_turn = ?1
            WHERE id = (select started_server_id from game_servers where alias = ?2)
            AND last_seen_turn <> ?1", 
            &[&current_turn, &game_alias]
        )?;
        Ok(rows > 0)
    }

    pub fn remove_player_from_game(&self, game_alias: &str, user: UserId) -> Result<(), Error> {
        info!("db::remove_player_from_game");
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
        info!("db::remove_server");
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
        conn.execute(
            "DELETE FROM started_servers
            WHERE id NOT IN (select started_server_id from game_servers)",
            &[]
        )?;
        conn.execute(
            "DELETE FROM lobbies
            WHERE id NOT IN (select lobby_id from game_servers)",
            &[]
        )?;
        Ok(())
    }

    pub fn servers_for_player(&self, user_id: UserId) -> Result<Vec<(GameServer, i32)>, Error> {
        info!("servers_for_player");
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare("
            SELECT s.address, g.alias, s.last_seen_turn, sp.nation_id, l.owner_id, l.era, l.player_count
            FROM players p
            JOIN server_players sp on sp.player_id = p.id
            JOIN game_servers g on g.id = sp.server_id
            LEFT JOIN lobbies l on l.id = g.lobby_id
            LEFT JOIN started_servers s on s.id = g.started_server_id
            WHERE p.discord_user_id = ?1
        ")?;

        let foo = stmt.query_map(&[&(user_id.0 as i64)], |ref row| {
            let alias: String = row.get(1);
            let maybe_address: Option<String> = row.get(0);
            let maybe_last_seen_turn: Option<i32> = row.get(2);
            let maybe_owner: Option<i64> = row.get(4); 
            let maybe_era: Option<i32> = row.get(5);
            let maybe_player_count: Option<i32> = row.get(6);
            let server = make_game_server(
                alias, 
                maybe_address, 
                maybe_last_seen_turn,
                maybe_owner,
                maybe_era,
                maybe_player_count,
            ).unwrap();

            let nation_id = row.get(3);
            (server, nation_id)
        })?;

        let mut ret: Vec<(GameServer, i32)> = vec![];
        for pair in foo {
            ret.push(pair?);
        }

        Ok(ret)
    }

    pub fn set_turn_notifications(&self, player: UserId, desired_turn_notifications: bool) -> Result<(), Error> {
        info!("db::set_turn_notifications");
        let conn = &*self.0.clone().get()?;
        conn.execute("
            UPDATE players
            SET turn_notifications = ?2
            WHERE discord_user_id = ?1
        ", &[&(player.0 as i64), &desired_turn_notifications])?;
        Ok(())
    }

    pub fn insert_started_state(&self, alias: &str, started_state: &StartedState) -> Result<(), Error> {
        info!("insert_started_state");
        let conn = &*self.0.clone().get()?;
        conn.execute("
            INSERT INTO started_servers (address, last_seen_turn)
            VALUES (?1, ?2)
        ", &[&started_state.address, &started_state.last_seen_turn])?;

        conn.execute("
            UPDATE game_servers
            SET started_server_id = 
                (SELECT s.id
                from started_servers s
                where s.address = ?1 and s.last_seen_turn = ?2)
            WHERE alias = ?3
        ", &[&started_state.address, &started_state.last_seen_turn, &alias])?;
        Ok(())
    }
}

fn make_game_server(
    alias: String,
    maybe_address: Option<String>,
    maybe_last_seen_turn: Option<i32>,
    maybe_owner: Option<i64>,
    maybe_era: Option<i32>,
    maybe_player_count: Option<i32>,
) -> Result<GameServer, Error> {

    let state = match (maybe_address, maybe_last_seen_turn, maybe_owner, maybe_era, maybe_player_count) {
        (Some(address), Some(last_seen_turn), None, None, None) => 
            GameServerState::StartedState (StartedState {
                address: address,
                last_seen_turn: last_seen_turn,
            }, None),
        (Some(address), Some(last_seen_turn), Some(owner), Some(era), Some(player_count)) =>
            GameServerState::StartedState (StartedState {
                address: address,
                last_seen_turn: last_seen_turn,
            }, Some(LobbyState {
                owner: UserId(owner as u64),
                era: Era::from_i32(era).ok_or(err_msg("unknown era"))?,
                player_count: player_count,
            })),
        (None, None, Some(owner), Some(era), Some(player_count)) =>
            GameServerState::Lobby (LobbyState {
                owner: UserId(owner as u64),
                era: Era::from_i32(era).ok_or(err_msg("unknown era"))?,
                player_count: player_count,
            }),
        _ => return Err(err_msg(format!("invalid db state for {}", alias)))
    };

    let server = GameServer {
        alias: alias,
        state: state,
    };
    Ok(server)
}

#[cfg(test)]
impl DbConnection {
    pub fn test() -> Self {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::builder().max_size(1).build(manager).unwrap();
        let db_conn = DbConnection(pool);
        db_conn.initialise().unwrap();
        db_conn
    }

    pub fn noop() -> Self {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::new(manager).unwrap();
        DbConnection(pool)
    }
}
