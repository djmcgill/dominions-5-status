use failure::{err_msg, Error};
use lazy_static::lazy_static;
use log::*;
use num_traits::{FromPrimitive, ToPrimitive};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use serenity::model::id::UserId;
use typemap::Key;

use crate::model::enums::*;
use crate::model::*;
use std::path::Path;

use failure::SyncFailure;

use migrant_lib::{list, Config, EmbeddedMigration, Migratable, Migrator, Settings};

#[cfg(test)]
pub mod test_helpers;

pub struct DbConnectionKey;
impl Key for DbConnectionKey {
    type Value = DbConnection;
}

lazy_static! {
    static ref MIGRATIONS: [EmbeddedMigration; 2] = {
        let mut m1 = EmbeddedMigration::with_tag("001-baseline");
        m1.up(include_str!("db/sql/migrations/001_baseline.sql"));

        let mut m2 = EmbeddedMigration::with_tag("002-lobby-description");
        m2.up(include_str!("db/sql/migrations/002_lobby_description.sql"));

        [m1, m2]
    };
}
pub struct DbConnection(Pool<SqliteConnectionManager>);
impl DbConnection {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let manager = SqliteConnectionManager::file(&path);
        let pool = Pool::new(manager)?;
        let db_conn = DbConnection(pool);
        db_conn.initialise(&path)?;
        Ok(db_conn)
    }

    fn initialise<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        info!("db::initialise");
        let settings = Settings::configure_sqlite()
            .database_path(path)
            .map_err(SyncFailure::new)?
            .build()
            .map_err(SyncFailure::new)?;
        let mut config = Config::with_settings(&settings);
        config.setup().map_err(SyncFailure::new)?;

        config
            .use_migrations(
                // there has GOT to be a better way
                MIGRATIONS[..]
                    .into_iter()
                    .cloned()
                    .map(|migration| -> Box<(dyn Migratable + 'static)> { Box::new(migration) })
                    .collect::<Vec<_>>(),
            )
            .map_err(SyncFailure::new)?;

        let config = config.reload().map_err(SyncFailure::new)?;

        Migrator::with_config(&config)
            .all(true)
            .show_output(false)
            .swallow_completion(true)
            .apply()
            .map_err(SyncFailure::new)?;

        let config = config.reload().map_err(SyncFailure::new)?;
        list(&config).map_err(SyncFailure::new)?;

        Ok(())
    }

    pub fn insert_server_player(
        &self,
        server_alias: &str,
        player_user_id: UserId,
        nation_id: u32,
    ) -> Result<(), Error> {
        info!("db::insert_server_player");
        let conn = &*self.0.clone().get()?;
        conn.execute(
            include_str!("db/sql/insert_server_player.sql"),
            params![&nation_id, &(player_user_id.0 as i64), &server_alias],
        )?;
        Ok(())
    }

    pub fn insert_game_server(&self, game_server: &GameServer) -> Result<(), Error> {
        info!("db::insert_game_server: {:?}", game_server);
        let conn = &mut *self.0.clone().get()?;

        match game_server.state {
            GameServerState::Lobby(ref lobby_state) => {
                let tx = conn.transaction()?;

                tx.execute(
                    include_str!("db/sql/insert_player.sql"),
                    params![&(lobby_state.owner.0 as i64), &true],
                )?;

                tx.execute(
                    include_str!("db/sql/insert_lobby.sql"),
                    params![
                        &lobby_state.era.to_i32(),
                        &(lobby_state.owner.0 as i64),
                        &lobby_state.player_count,
                        &lobby_state.description,
                    ],
                )?;
                tx.execute(
                    include_str!("db/sql/insert_game_server_from_lobby.sql"),
                    params![
                        &game_server.alias,
                        &lobby_state.era.to_i32(),
                        &(lobby_state.owner.0 as i64),
                        &lobby_state.player_count,
                    ],
                )?;
                tx.commit()?;
                Ok(())
            }
            GameServerState::StartedState(ref started_state, None) => {
                let tx = conn.transaction()?;
                tx.execute(
                    include_str!("db/sql/insert_started_server.sql"),
                    params![&started_state.address, &started_state.last_seen_turn],
                )?;
                tx.execute(
                    include_str!("db/sql/insert_started_game_server.sql"),
                    params![&game_server.alias, &started_state.address],
                )?;
                tx.commit()?;
                Ok(())
            }
            GameServerState::StartedState(ref started_state, Some(ref lobby_state)) => {
                let tx = conn.transaction()?;
                tx.execute(
                    include_str!("db/sql/insert_player.sql"),
                    params![&(lobby_state.owner.0 as i64), &true],
                )?;

                tx.execute(
                    include_str!("db/sql/insert_lobby.sql"),
                    params![
                        &lobby_state.era.to_i32(),
                        &(lobby_state.owner.0 as i64),
                        &lobby_state.player_count,
                        &lobby_state.description,
                    ],
                )?;
                tx.execute(
                    include_str!("db/sql/insert_game_server_from_lobby.sql"),
                    params![
                        &game_server.alias,
                        &lobby_state.era.to_i32(),
                        &(lobby_state.owner.0 as i64),
                        &lobby_state.player_count,
                    ],
                )?;
                tx.execute(
                    include_str!("db/sql/insert_started_state.sql"),
                    params![&started_state.address, &started_state.last_seen_turn],
                )?;

                tx.execute(
                    include_str!("db/sql/update_game_with_started_state.sql"),
                    params![
                        &started_state.address,
                        &started_state.last_seen_turn,
                        &game_server.alias,
                    ],
                )?;
                tx.commit()?;
                Ok(())
            }
        }
    }

    pub fn insert_player(&self, player: &Player) -> Result<(), Error> {
        info!("db::insert_player");
        let conn = &*self.0.clone().get()?;
        conn.execute(
            include_str!("db/sql/insert_player.sql"),
            params![
                &(player.discord_user_id.0 as i64),
                &player.turn_notifications,
            ],
        )?;
        Ok(())
    }

    pub fn retrieve_all_servers(&self) -> Result<Vec<GameServer>, Error> {
        info!("db::retrieve_all_servers");
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare(include_str!("db/sql/select_game_servers.sql"))?;
        let foo = stmt
            .query_and_then(params![], |ref row| -> Result<GameServer, Error> {
                let maybe_address: Option<String> = row.get(1)?;
                let maybe_last_seen_turn: Option<i32> = row.get(2)?;
                let alias: String = row.get(0)?;
                let maybe_owner: Option<i64> = row.get(3)?;
                let maybe_era: Option<i32> = row.get(4)?;
                let maybe_player_count: Option<i32> = row.get(5)?;
                let description: Option<String> = row.get(6)?;

                let game_server = make_game_server(
                    alias,
                    maybe_address,
                    maybe_last_seen_turn,
                    maybe_owner,
                    maybe_era,
                    maybe_player_count,
                    description,
                )?;

                Ok(game_server)
            })?
            .collect::<Result<Vec<GameServer>, _>>()?;
        Ok(foo)
    }

    pub fn players_with_nations_for_game_alias(
        &self,
        game_alias: &str,
    ) -> Result<Vec<(Player, usize)>, Error> {
        info!("players_with_nations_for_game_alias");
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare(include_str!("db/sql/select_players_nations.sql"))?;
        let foo = stmt.query_map(&[&game_alias], |ref row| {
            let discord_user_id: i64 = row.get(0).unwrap();
            let player = Player {
                discord_user_id: UserId(discord_user_id as u64),
                turn_notifications: row.get(2).unwrap(),
            };
            let nation: i32 = row.get(1).unwrap();
            Ok((player, nation as usize))
        })?;
        let vec = foo.collect::<Result<Vec<_>, _>>()?;
        Ok(vec)
    }

    pub fn game_for_alias(&self, game_alias: &str) -> Result<GameServer, Error> {
        info!("db::game_for_alias");
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare(include_str!("db/sql/select_game_server_for_alias.sql"))?;
        let foo = stmt.query_map(&[&game_alias], |ref row| {
            let maybe_address: Option<String> = row.get(0).unwrap();
            let maybe_last_seen_turn: Option<i32> = row.get(1).unwrap();
            let maybe_owner: Option<i64> = row.get(2).unwrap();
            let maybe_era: Option<i32> = row.get(3).unwrap();
            let maybe_player_count: Option<i32> = row.get(4).unwrap();
            let description: Option<String> = row.get(5).unwrap();
            Ok(make_game_server(
                game_alias.to_owned(),
                maybe_address,
                maybe_last_seen_turn,
                maybe_owner,
                maybe_era,
                maybe_player_count,
                description,
            )
            .unwrap())
        })?;
        let vec = foo.collect::<Result<Vec<_>, _>>()?;
        if vec.len() == 1 {
            Ok(vec
                .into_iter()
                .next()
                .ok_or(err_msg("THIS SHOULD NEVER HAPPEN"))?) // TODO: *vomits*
        } else {
            Err(err_msg(format!(
                "could not find the game with alias {}",
                game_alias
            )))
        }
    }

    pub fn update_game_with_possibly_new_turn(
        &self,
        game_alias: &str,
        current_turn: i32,
    ) -> Result<bool, Error> {
        info!("db::update_game_with_possibly_new_turn");
        let conn = &mut *self.0.clone().get()?;
        let rows = conn.execute(
            include_str!("db/sql/update_game_with_turn.sql"),
            params![&current_turn, &game_alias],
        )?;
        info!("db::update_game_with_possibly_new_turn FINISHED");
        Ok(rows > 0)
    }

    pub fn remove_player_from_game(&self, game_alias: &str, user: UserId) -> Result<usize, Error> {
        info!("db::remove_player_from_game");
        let conn = &*self.0.clone().get()?;
        Ok(conn.execute(
            include_str!("db/sql/delete_player_from_game.sql"),
            params![&game_alias, &(user.0 as i64)],
        )?)
    }

    pub fn remove_server(&self, game_alias: &str) -> Result<(), Error> {
        info!("db::remove_server");
        let conn = &mut *self.0.clone().get()?;
        let tx = conn.transaction()?;
        tx.execute(
            include_str!("db/sql/delete_server_players.sql"),
            params![&game_alias],
        )?;
        let rows_modified = tx.execute(
            include_str!("db/sql/delete_game_server.sql"),
            params![&game_alias],
        )?;
        tx.execute(include_str!("db/sql/delete_started_server.sql"), params![])?;
        tx.execute(include_str!("db/sql/delete_lobby.sql"), params![])?;
        if rows_modified != 0 {
            tx.commit()?;
            Ok(())
        } else {
            Err(err_msg(format!(
                "Could not find server with name {}",
                game_alias
            )))
        }
    }

    pub fn servers_for_player(&self, user_id: UserId) -> Result<Vec<(GameServer, i32)>, Error> {
        info!("servers_for_player");
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare(include_str!("db/sql/select_servers_for_player.sql"))?;

        let foo = stmt.query_map(&[&(user_id.0 as i64)], |ref row| {
            let alias: String = row.get(1).unwrap();
            let maybe_address: Option<String> = row.get(0).unwrap();
            let maybe_last_seen_turn: Option<i32> = row.get(2).unwrap();
            let maybe_owner: Option<i64> = row.get(4).unwrap();
            let maybe_era: Option<i32> = row.get(5).unwrap();
            let maybe_player_count: Option<i32> = row.get(6).unwrap();
            let description: Option<String> = row.get(7).unwrap();
            let server = make_game_server(
                alias,
                maybe_address,
                maybe_last_seen_turn,
                maybe_owner,
                maybe_era,
                maybe_player_count,
                description,
            )
            .unwrap();

            let nation_id = row.get(3).unwrap();
            Ok((server, nation_id))
        })?;

        let mut ret: Vec<(GameServer, i32)> = vec![];
        for pair in foo {
            ret.push(pair?);
        }

        Ok(ret)
    }

    pub fn set_turn_notifications(
        &self,
        player: UserId,
        desired_turn_notifications: bool,
    ) -> Result<(), Error> {
        info!("db::set_turn_notifications");
        let conn = &*self.0.clone().get()?;
        conn.execute(
            include_str!("db/sql/update_turn_notifications.sql"),
            params![&(player.0 as i64), &desired_turn_notifications],
        )?;
        Ok(())
    }

    pub fn insert_started_state(
        &self,
        alias: &str,
        started_state: &StartedState,
    ) -> Result<(), Error> {
        info!("insert_started_state");
        let conn = &mut *self.0.clone().get()?;
        let tx = conn.transaction()?;
        tx.execute(
            include_str!("db/sql/insert_started_state.sql"),
            params![&started_state.address, &started_state.last_seen_turn],
        )?;

        tx.execute(
            include_str!("db/sql/update_game_with_started_state.sql"),
            params![
                &started_state.address,
                &started_state.last_seen_turn,
                &alias,
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn select_lobbies(&self) -> Result<Vec<(GameServer, i32)>, Error> {
        info!("select_lobbies");
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare(include_str!("db/sql/select_lobbies.sql"))?;
        let foo = stmt.query_map(params![], |ref row| {
            let alias: String = row.get(0).unwrap();
            let maybe_owner: Option<i64> = row.get(1).unwrap();
            let maybe_era: Option<i32> = row.get(2).unwrap();
            let maybe_player_count: Option<i32> = row.get(3).unwrap();
            let registered_player_count: i32 = row.get(4).unwrap();
            let description: Option<String> = row.get(5).unwrap();
            let server = make_game_server(
                alias,
                None,
                None,
                maybe_owner,
                maybe_era,
                maybe_player_count,
                description,
            )
            .unwrap();
            Ok((server, registered_player_count))
        })?;
        let vec = foo.collect::<Result<Vec<_>, _>>()?;
        Ok(vec)
    }

    pub fn update_lobby_with_description(
        &self,
        alias: &str,
        description: &str,
    ) -> Result<(), Error> {
        info!("update_lobby_with_description");
        let conn = &*self.0.clone().get()?;
        let rows_modified = conn.execute(
            include_str!("db/sql/update_lobby_with_description.sql"),
            &[&alias, &description],
        )?;
        if rows_modified != 0 {
            Ok(())
        } else {
            Err(err_msg(format!("Could not find lobby with name {}", alias)))
        }
    }
}

fn make_game_server(
    alias: String,
    maybe_address: Option<String>,
    maybe_last_seen_turn: Option<i32>,
    maybe_owner: Option<i64>,
    maybe_era: Option<i32>,
    maybe_player_count: Option<i32>,
    description: Option<String>,
) -> Result<GameServer, Error> {
    let state = match (
        maybe_address,
        maybe_last_seen_turn,
        maybe_owner,
        maybe_era,
        maybe_player_count,
    ) {
        (Some(address), Some(last_seen_turn), None, None, None) => GameServerState::StartedState(
            StartedState {
                address,
                last_seen_turn,
            },
            None,
        ),
        (Some(address), Some(last_seen_turn), Some(owner), Some(era), Some(player_count)) => {
            GameServerState::StartedState(
                StartedState {
                    address,
                    last_seen_turn,
                },
                Some(LobbyState {
                    owner: UserId(owner as u64),
                    era: Era::from_i32(era).ok_or(err_msg("unknown era"))?,
                    player_count,
                    description,
                }),
            )
        }
        (None, None, Some(owner), Some(era), Some(player_count)) => {
            GameServerState::Lobby(LobbyState {
                owner: UserId(owner as u64),
                era: Era::from_i32(era).ok_or(err_msg("unknown era"))?,
                player_count,
                description,
            })
        }
        _ => return Err(err_msg(format!("invalid db state for {}", alias))),
    };

    let server = GameServer { alias, state };
    Ok(server)
}

#[cfg(test)]
mod db_test {
    use super::*;

    #[test]
    fn test_insert_server_and_server_player() {
        let conn: DbConnection = DbConnection::test();
        let server_alias = "foobar";
        let player_user_id = UserId(131311);
        let nation_id = 32;
        let server_address = "eg_address:3000".to_owned();
        let server_last_seen_turn = 1;

        conn.insert_game_server(&GameServer {
            alias: server_alias.to_owned(),
            state: GameServerState::StartedState(
                StartedState {
                    address: server_address.clone(),
                    last_seen_turn: server_last_seen_turn,
                },
                None,
            ),
        })
        .unwrap();

        let started_server: (u32, String, i32) = {
            let raw_conn: &rusqlite::Connection = &*conn.0.clone().get().unwrap();
            raw_conn
                .query_row(
                    "select * from started_servers where address = ?1",
                    params![&server_address],
                    |r| Ok((r.get(0).unwrap(), r.get(1).unwrap(), r.get(2).unwrap())),
                )
                .unwrap()
        };

        assert_eq!(started_server.1, server_address);
        assert_eq!(started_server.2, server_last_seen_turn);

        let game_server: (u32, String, u32, Option<u32>) = {
            let raw_conn: &rusqlite::Connection = &*conn.0.clone().get().unwrap();
            raw_conn
                .query_row(
                    "select * from game_servers where alias = ?1",
                    params![&server_alias],
                    |r| {
                        Ok((
                            r.get(0).unwrap(),
                            r.get(1).unwrap(),
                            r.get(2).unwrap(),
                            r.get(3).unwrap(),
                        ))
                    },
                )
                .unwrap()
        };

        assert_eq!(game_server.1, server_alias);
        assert_eq!(game_server.2, started_server.0);
        assert!(game_server.3.is_none());

        conn.insert_player(&Player {
            discord_user_id: player_user_id,
            turn_notifications: false,
        })
        .unwrap();

        let player: (u32, u32, bool) = {
            let raw_conn: &rusqlite::Connection = &*conn.0.clone().get().unwrap();
            raw_conn
                .query_row(
                    "select * from players where discord_user_id = ?1",
                    params![player_user_id.0 as i64],
                    |r| Ok((r.get(0).unwrap(), r.get(1).unwrap(), r.get(2).unwrap())),
                )
                .unwrap()
        };

        assert_eq!(player.1 as u64, player_user_id.0);
        assert_eq!(player.2, false);

        conn.insert_server_player(server_alias, player_user_id, nation_id)
            .unwrap();

        let server_player: (u32, u32, u32) = {
            let raw_conn: &rusqlite::Connection = &*conn.0.clone().get().unwrap();
            raw_conn
                .query_row("select * from server_players", params![], |r| {
                    Ok((r.get(0).unwrap(), r.get(1).unwrap(), r.get(2).unwrap()))
                })
                .unwrap()
        };

        assert_eq!(server_player.0, game_server.0);
        assert_eq!(server_player.1, player.0);
        assert_eq!(server_player.2, nation_id);
    }
}
