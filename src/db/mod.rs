use failure::{err_msg, Error};
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;
use serenity::model::id::UserId;
use typemap::Key;
use num_traits::{FromPrimitive, ToPrimitive};

use model::*;
use model::enums::*;
use std::path::Path;

use failure::SyncFailure;

use migrant_lib::{Settings, Config, Migrator, list, EmbeddedMigration, Migratable};

#[cfg(test)]
pub mod test_helpers;

pub struct DbConnectionKey;
impl Key for DbConnectionKey {
    type Value = DbConnection;
}

lazy_static! {
    static ref MIGRATIONS: [Box<EmbeddedMigration>; 2] = [
        Box::new(EmbeddedMigration {
            tag: "001-baseline".to_owned(),
            up: Some(include_str!("sql/migrations/001_baseline.sql")),
            down: None,
        }),
        Box::new(EmbeddedMigration {
            tag: "002-lobby-description".to_owned(),
            up: Some(include_str!("sql/migrations/002_lobby_description.sql")),
            down: None,
        }),
    ];
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

        let migrations: Vec<Box<Migratable>> =
            MIGRATIONS
                .iter()
                .cloned()
                .map(|x| x as Box<Migratable>) // TODO: do NOT map cast
                .collect::<Vec<_>>();
        config.use_migrations(
            &migrations
        ).map_err(SyncFailure::new)?;

        let config = config.reload().map_err(SyncFailure::new)?;

        Migrator::with_config(&config)
            .all(true)
            .show_output(false)
            .swallow_completion(true)
            .apply().map_err(SyncFailure::new)?;

        let config = config.reload().map_err(SyncFailure::new)?;
        list(&config).map_err(SyncFailure::new)?;

        Ok(())
    }

    pub fn insert_server_player(
        &self,
        server_alias: &str,
        player_user_id: &UserId,
        nation_id: u32,
    ) -> Result<(), Error> {
        info!("db::insert_server_player");
        let conn = &*self.0.clone().get()?;
        conn.execute(
            include_str!("sql/insert_server_player.sql"),
            &[&nation_id, &(player_user_id.0 as i64), &server_alias],
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
                    include_str!("sql/insert_game_server_lobby_state.sql"),
                    &[&(lobby_state.owner.0 as i64)],
                )?;

                tx.execute(
                    include_str!("sql/insert_lobby.sql"),
                    &[
                        &lobby_state.era.to_i32(),
                        &(lobby_state.owner.0 as i64),
                        &lobby_state.player_count,
                        &lobby_state.description,
                    ],
                )?;
                tx.execute(
                    include_str!("sql/insert_game_server_from_lobby.sql"),
                    &[
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
                    include_str!("sql/insert_started_server.sql"),
                    &[&started_state.address, &started_state.last_seen_turn],
                )?;
                tx.execute(
                    include_str!("sql/insert_started_game_server.sql"),
                    &[&game_server.alias, &started_state.address],
                )?;
                tx.commit()?;
                Ok(())
            }
            GameServerState::StartedState(ref started_state, Some(ref lobby_state)) => {
                let tx = conn.transaction()?;
                tx.execute(
                    include_str!("sql/insert_game_server_lobby_state.sql"),
                    &[&(lobby_state.owner.0 as i64)],
                )?;

                tx.execute(
                    include_str!("sql/insert_lobby.sql"),
                    &[
                        &lobby_state.era.to_i32(),
                        &(lobby_state.owner.0 as i64),
                        &lobby_state.player_count,
                    ],
                )?;
                tx.execute(
                    include_str!("sql/insert_game_server_from_lobby.sql"),
                    &[
                        &game_server.alias,
                        &lobby_state.era.to_i32(),
                        &(lobby_state.owner.0 as i64),
                        &lobby_state.player_count,
                    ],
                )?;
                tx.execute(
                    include_str!("sql/insert_started_state.sql"),
                    &[&started_state.address, &started_state.last_seen_turn],
                )?;

                tx.execute(
                    include_str!("sql/update_game_with_started_state.sql"),
                    &[
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
            include_str!("sql/insert_player.sql"),
            &[
                &(player.discord_user_id.0 as i64),
                &player.turn_notifications,
            ],
        )?;
        Ok(())
    }

    #[allow(unused_variables)]
    pub fn insert_nap(&self, nap: &Nap) -> Result<(), Error> {
        // insert players
        // insert nap
        // insert nap_type
        Err(err_msg("db::insert_nap not implemented yet"))
    }

    #[allow(unused_variables)]
    pub fn select_naps(&self, player: UserId) -> Result<Vec<Nap>, Error> {
        Err(err_msg("db::select_naps not implemented yet"))
    }

    pub fn retrieve_all_servers(&self) -> Result<Vec<GameServer>, Error> {
        info!("db::retrieve_all_servers");
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare(include_str!("sql/select_game_servers.sql"))?;
        let foo = stmt.query_map(&[], |ref row| {
            let maybe_address: Option<String> = row.get(1);
            let maybe_last_seen_turn: Option<i32> = row.get(2);
            let alias: String = row.get(0);
            let maybe_owner: Option<i64> = row.get(3);
            let maybe_era: Option<i32> = row.get(4);
            let maybe_player_count: Option<i32> = row.get(5);
            let description: Option<String> = row.get(6);
            let server = make_game_server(
                alias,
                maybe_address,
                maybe_last_seen_turn,
                maybe_owner,
                maybe_era,
                maybe_player_count,
                description,
            ).unwrap();
            server
        })?;
        let vec = foo.collect::<Result<Vec<_>, _>>()?;
        Ok(vec)
    }

    pub fn players_with_nations_for_game_alias(
        &self,
        game_alias: &str,
    ) -> Result<Vec<(Player, usize)>, Error> {
        info!("players_with_nations_for_game_alias");
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare(include_str!("sql/select_players_nations.sql"))?;
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
        let mut stmt = conn.prepare(include_str!("sql/select_game_server_for_alias.sql"))?;
        let foo = stmt.query_map(&[&game_alias], |ref row| {
            let maybe_address: Option<String> = row.get(0);
            let maybe_last_seen_turn: Option<i32> = row.get(1);
            let maybe_owner: Option<i64> = row.get(2);
            let maybe_era: Option<i32> = row.get(3);
            let maybe_player_count: Option<i32> = row.get(4);
            let description: Option<String> = row.get(5);
            let server = make_game_server(
                game_alias.to_owned(),
                maybe_address,
                maybe_last_seen_turn,
                maybe_owner,
                maybe_era,
                maybe_player_count,
                description,
            ).unwrap();
            server
        })?;
        let vec = foo.collect::<Result<Vec<_>, _>>()?;
        if vec.len() == 1 {
            Ok(vec.into_iter()
                .next()
                .ok_or(err_msg("THIS SHOULD NEVER HAPPEN"))?) // TODO: *vomits*
        } else {
            Err(err_msg(
                format!("could not find the game with alias {}", game_alias),
            ))
        }
    }

    pub fn update_game_with_possibly_new_turn(
        &self,
        game_alias: &str,
        current_turn: i32,
    ) -> Result<bool, Error> {
        fn trace_fn(s: &str) {
            info!("TRACE: {}", s);
        }

        info!("db::update_game_with_possibly_new_turn");
        let conn = &mut *self.0.clone().get()?;
        conn.trace(Some(trace_fn));
        let rows = conn.execute(
            include_str!("sql/update_game_with_turn.sql"),
            &[&current_turn, &game_alias],
        )?;
        info!("db::update_game_with_possibly_new_turn FINISHED");
        Ok(rows > 0)
    }

    pub fn remove_player_from_game(&self, game_alias: &str, user: UserId) -> Result<(), Error> {
        info!("db::remove_player_from_game");
        let conn = &*self.0.clone().get()?;
        conn.execute(
            include_str!("sql/delete_player_from_game.sql"),
            &[&game_alias, &(user.0 as i64)],
        )?;
        Ok(())
    }

    pub fn remove_server(&self, game_alias: &str) -> Result<(), Error> {
        info!("db::remove_server");
        let conn = &mut *self.0.clone().get()?;
        let tx = conn.transaction()?;
        tx.execute(
            include_str!("sql/delete_server_players.sql"),
            &[&game_alias],
        )?;
        let rows_modified = tx.execute(include_str!("sql/delete_game_server.sql"), &[&game_alias])?;
        tx.execute(include_str!("sql/delete_started_server.sql"), &[])?;
        tx.execute(include_str!("sql/delete_lobby.sql"), &[])?;
        if rows_modified != 0 {
            tx.commit()?;
            Ok(())
        } else {
            Err(err_msg(format!("Could not find server with name {}", game_alias)))
        }
    }

    pub fn servers_for_player(&self, user_id: UserId) -> Result<Vec<(GameServer, i32)>, Error> {
        info!("servers_for_player");
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare(include_str!("sql/select_servers_for_player.sql"))?;

        let foo = stmt.query_map(&[&(user_id.0 as i64)], |ref row| {
            let alias: String = row.get(1);
            let maybe_address: Option<String> = row.get(0);
            let maybe_last_seen_turn: Option<i32> = row.get(2);
            let maybe_owner: Option<i64> = row.get(4);
            let maybe_era: Option<i32> = row.get(5);
            let maybe_player_count: Option<i32> = row.get(6);
            let description: Option<String> = row.get(7);
            let server = make_game_server(
                alias,
                maybe_address,
                maybe_last_seen_turn,
                maybe_owner,
                maybe_era,
                maybe_player_count,
                description,
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

    pub fn set_turn_notifications(
        &self,
        player: UserId,
        desired_turn_notifications: bool,
    ) -> Result<(), Error> {
        info!("db::set_turn_notifications");
        let conn = &*self.0.clone().get()?;
        conn.execute(
            include_str!("sql/update_turn_notifications.sql"),
            &[&(player.0 as i64), &desired_turn_notifications],
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
            include_str!("sql/insert_started_state.sql"),
            &[&started_state.address, &started_state.last_seen_turn],
        )?;

        tx.execute(
            include_str!("sql/update_game_with_started_state.sql"),
            &[
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
        let mut stmt = conn.prepare(include_str!("sql/select_lobbies.sql"))?;
        let foo = stmt.query_map(&[], |ref row| {
            let alias: String = row.get(0);
            let maybe_owner: Option<i64> = row.get(1);
            let maybe_era: Option<i32> = row.get(2);
            let maybe_player_count: Option<i32> = row.get(3);
            let registered_player_count: i32 = row.get(4);
            let description: Option<String> = row.get(5);
            let server = make_game_server(
                alias,
                None,
                None,
                maybe_owner,
                maybe_era,
                maybe_player_count,
                description,
            ).unwrap();
            (server, registered_player_count)
        })?;
        let vec = foo.collect::<Result<Vec<_>, _>>()?;
        Ok(vec)
    }

    pub fn update_lobby_with_description(&self, alias: &str, description: &str) -> Result<(), Error> {
        info!("update_lobby_with_description");
        let conn = &*self.0.clone().get()?;
        let rows_modified = conn.execute(
            include_str!("sql/update_lobby_with_description.sql"),
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
                address: address,
                last_seen_turn: last_seen_turn,
            },
            None,
        ),
        (Some(address), Some(last_seen_turn), Some(owner), Some(era), Some(player_count)) => {
            GameServerState::StartedState(
                StartedState {
                    address: address,
                    last_seen_turn: last_seen_turn,
                },
                Some(LobbyState {
                    owner: UserId(owner as u64),
                    era: Era::from_i32(era).ok_or(err_msg("unknown era"))?,
                    player_count: player_count,
                    description: description,
                }),
            )
        }
        (None, None, Some(owner), Some(era), Some(player_count)) => {
            GameServerState::Lobby(LobbyState {
                owner: UserId(owner as u64),
                era: Era::from_i32(era).ok_or(err_msg("unknown era"))?,
                player_count: player_count,
                description: description,
            })
        }
        _ => return Err(err_msg(format!("invalid db state for {}", alias))),
    };

    let server = GameServer {
        alias: alias,
        state: state,
    };
    Ok(server)
}
