use crate::model::enums::*;
use crate::model::game_server::{GameServer, GameServerState, LobbyState, StartedState};
use crate::model::player::Player;

use anyhow::anyhow;
use lazy_static::lazy_static;
use log::*;
use migrant_lib::{Config, EmbeddedMigration, Migratable, Migrator, Settings};
use num_traits::{FromPrimitive, ToPrimitive};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use serenity::model::id::UserId;

use crate::model::nation::BotNationIdentifier;
use serenity::prelude::TypeMapKey;
use std::path::Path;

pub struct DbConnectionKey;
impl TypeMapKey for DbConnectionKey {
    type Value = DbConnection;
}

lazy_static! {
    static ref MIGRATIONS: [EmbeddedMigration; 3] = {
        let mut m1 = EmbeddedMigration::with_tag("001-baseline");
        m1.up(include_str!("db/sql/migrations/001_baseline.sql"));

        let mut m2 = EmbeddedMigration::with_tag("002-lobby-description");
        m2.up(include_str!("db/sql/migrations/002_lobby_description.sql"));

        let mut m3 = EmbeddedMigration::with_tag("003-register-custom");
        m3.up(include_str!("db/sql/migrations/003_register_custom.sql"));

        [m1, m2, m3]
    };
}

#[derive(Clone)]
pub struct DbConnection(Pool<SqliteConnectionManager>);
impl DbConnection {
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let manager = SqliteConnectionManager::file(&path);
        let pool = Pool::new(manager)?;
        let db_conn = DbConnection(pool);
        db_conn.initialise(&path)?;
        Ok(db_conn)
    }

    fn initialise<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        info!("db::initialise");
        let settings = Settings::configure_sqlite()
            .database_path(path)
            // https://github.com/dtolnay/anyhow/issues/72#issuecomment-599243682 lol
            .map_err(|e| anyhow!(e.to_string()))?
            .build()
            .map_err(|e| anyhow!(e.to_string()))?;

        let mut config = Config::with_settings(&settings);
        config.setup().map_err(|e| anyhow!(e.to_string()))?;

        config
            .use_migrations(
                // there has GOT to be a better way
                MIGRATIONS[..]
                    .into_iter()
                    .cloned()
                    .map(|migration| -> Box<(dyn Migratable + 'static)> { Box::new(migration) })
                    .collect::<Vec<_>>(),
            )
            .map_err(|e| anyhow!(e.to_string()))?;

        let config = config.reload().map_err(|e| anyhow!(e.to_string()))?;

        Migrator::with_config(&config)
            .all(true)
            .show_output(false)
            .swallow_completion(true)
            .apply()
            .map_err(|e| anyhow!(e.to_string()))?;

        // Used for debugging
        // let config = config.reload().map_err(|e| anyhow!(e.to_string()))?;
        // list(&config).map_err(|e| anyhow!(e.to_string()))?;

        Ok(())
    }

    pub fn insert_game_server(&self, game_server: &GameServer) -> anyhow::Result<()> {
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

    pub fn insert_player_into_server(
        &self,
        player: &Player,
        server_alias: &str,
        nation_identifier: BotNationIdentifier,
    ) -> anyhow::Result<()> {
        // We actually still want custom_nation_name = None for static nations here even when we know the name
        let (nation_id, custom_nation_name) = match nation_identifier {
            BotNationIdentifier::CustomId(nation_id) => (Some(nation_id), None),
            BotNationIdentifier::Existing(existing) => (Some(existing.id), None),
            BotNationIdentifier::CustomName(custom_name) => (None, Some(custom_name)),
        };

        info!("db::insert_player_into_server");
        let conn = &mut *self.0.clone().get()?;
        let tx = conn.transaction()?;
        tx.execute(
            include_str!("db/sql/insert_player.sql"),
            params![
                &(player.discord_user_id.0 as i64),
                &player.turn_notifications,
            ],
        )?;
        tx.execute(
            include_str!("db/sql/insert_server_player.sql"),
            params![
                &nation_id,
                &custom_nation_name,
                &(player.discord_user_id.0 as i64),
                &server_alias
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn retrieve_all_servers(&self) -> anyhow::Result<Vec<GameServer>> {
        info!("db::retrieve_all_servers");
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare(include_str!("db/sql/select_game_servers.sql"))?;
        let foo = stmt
            .query_and_then(params![], |ref row| -> anyhow::Result<GameServer> {
                let maybe_address: Option<String> = row.get(1)?;
                let maybe_last_seen_turn: Option<i32> = row.get(2)?;
                let alias: String = row.get(0)?;
                let maybe_owner: Option<i64> = row.get(3)?;
                let lobby_era: Option<i32> = row.get(4)?;
                let maybe_player_count: Option<i32> = row.get(5)?;
                let description: Option<String> = row.get(6)?;

                let game_server = make_game_server(
                    alias,
                    maybe_address,
                    maybe_last_seen_turn,
                    maybe_owner,
                    lobby_era,
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
    ) -> anyhow::Result<Vec<(Player, BotNationIdentifier)>> {
        info!("players_with_nations_for_game_alias");
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare(include_str!("db/sql/select_players_nations.sql"))?;
        let foo = stmt.query_map(&[&game_alias], |ref row| {
            let discord_user_id: i64 = row.get(0).unwrap();
            let player = Player {
                discord_user_id: UserId(discord_user_id as u64),
                turn_notifications: row.get(3).unwrap(),
            };
            let nation_id_i32: Option<i32> = row.get(1).unwrap();
            let nation_id_u32 = nation_id_i32.map(|id| id as u32);
            let custom_nation_name: Option<String> = row.get(2).unwrap();
            let nation_identifier =
                BotNationIdentifier::from_id_and_name(nation_id_u32, custom_nation_name).unwrap();
            Ok((player, nation_identifier))
        })?;
        let vec = foo.collect::<Result<Vec<_>, _>>()?;
        Ok(vec)
    }

    pub fn game_for_alias(&self, game_alias: &str) -> anyhow::Result<GameServer> {
        info!("db::game_for_alias");
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare(include_str!("db/sql/select_game_server_for_alias.sql"))?;
        let foo = stmt.query_map(&[&game_alias], |ref row| {
            let maybe_address: Option<String> = row.get(0).unwrap();
            let maybe_last_seen_turn: Option<i32> = row.get(1).unwrap();
            let maybe_owner: Option<i64> = row.get(2).unwrap();
            let maybe_lobby_era: Option<i32> = row.get(3).unwrap();
            let maybe_player_count: Option<i32> = row.get(4).unwrap();
            let description: Option<String> = row.get(5).unwrap();
            Ok(make_game_server(
                game_alias.to_owned(),
                maybe_address,
                maybe_last_seen_turn,
                maybe_owner,
                maybe_lobby_era,
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
                .ok_or(anyhow!("THIS SHOULD NEVER HAPPEN"))?) // TODO: *vomits*
        } else {
            Err(anyhow!("could not find the game with alias {}", game_alias))
        }
    }

    pub fn update_game_with_possibly_new_turn(
        &self,
        game_alias: &str,
        current_turn: i32,
    ) -> anyhow::Result<bool> {
        info!("db::update_game_with_possibly_new_turn");
        let conn = &mut *self.0.clone().get()?;
        let rows = conn.execute(
            include_str!("db/sql/update_game_with_turn.sql"),
            params![&current_turn, &game_alias],
        )?;
        info!("db::update_game_with_possibly_new_turn FINISHED");
        Ok(rows > 0)
    }

    pub fn remove_player_from_game(&self, game_alias: &str, user: UserId) -> anyhow::Result<usize> {
        info!("db::remove_player_from_game");
        let conn = &*self.0.clone().get()?;
        Ok(conn.execute(
            include_str!("db/sql/delete_player_from_game.sql"),
            params![&game_alias, &(user.0 as i64)],
        )?)
    }

    pub fn remove_server(&self, game_alias: &str) -> anyhow::Result<()> {
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
            Err(anyhow!("Could not find server with name {}", game_alias))
        }
    }

    pub fn servers_for_player(
        &self,
        user_id: UserId,
    ) -> anyhow::Result<Vec<(GameServer, BotNationIdentifier)>> {
        info!("servers_for_player");
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare(include_str!("db/sql/select_servers_for_player.sql"))?;

        let foo = stmt.query_map(&[&(user_id.0 as i64)], |ref row| {
            let alias: String = row.get(1).unwrap();
            let maybe_address: Option<String> = row.get(0).unwrap();
            let maybe_last_seen_turn: Option<i32> = row.get(2).unwrap();
            let maybe_owner: Option<i64> = row.get(5).unwrap();
            let maybe_lobby_era: Option<i32> = row.get(6).unwrap();
            let maybe_player_count: Option<i32> = row.get(7).unwrap();
            let description: Option<String> = row.get(8).unwrap();
            let server = make_game_server(
                alias,
                maybe_address,
                maybe_last_seen_turn,
                maybe_owner,
                maybe_lobby_era,
                maybe_player_count,
                description,
            )
            .unwrap();

            let nation_id_i32: Option<i32> = row.get(3).unwrap();
            let nation_id_u32 = nation_id_i32.map(|id| id as u32);
            let custom_nation_name: Option<String> = row.get(4).unwrap();
            Ok((server, nation_id_u32, custom_nation_name))
        })?;

        let mut ret: Vec<(GameServer, BotNationIdentifier)> = vec![];
        for row_result in foo {
            let (game_server, option_nation_id, option_custom_nation_name) = row_result?;
            let nation_identifier = match (option_nation_id, option_custom_nation_name) {
                (Some(nation_id), None) => {
                    if let Some(static_nation) = Nations::from_id(nation_id) {
                        Ok(BotNationIdentifier::Existing(static_nation))
                    } else {
                        Ok(BotNationIdentifier::CustomId(nation_id))
                    }
                }
                (None, Some(custom_nation_name)) => {
                    Ok(BotNationIdentifier::CustomName(custom_nation_name))
                }
                _ => Err(anyhow!(
                    "No nation info for user '{}' in game '{}'! This should not happen!",
                    user_id,
                    game_server.alias
                )),
            }?;

            ret.push((game_server, nation_identifier));
        }

        Ok(ret)
    }

    pub fn set_turn_notifications(
        &self,
        player: UserId,
        desired_turn_notifications: bool,
    ) -> anyhow::Result<()> {
        info!("db::set_turn_notifications");
        let conn = &*self.0.clone().get()?;
        conn.execute(
            include_str!("db/sql/update_turn_notifications.sql"),
            params![&(player.0 as i64), &desired_turn_notifications],
        )?;
        Ok(())
    }

    pub fn remove_started_state(&self, alias: &str) -> anyhow::Result<()> {
        info!("remove_started_state");
        let conn = &mut *self.0.clone().get()?;
        let tx = conn.transaction()?;

        let rows_modified = tx.execute(
            include_str!("db/sql/update_game_with_null_started_state.sql"),
            params![&alias],
        )?;
        tx.execute(include_str!("db/sql/delete_started_server.sql"), params![])?;
        tx.commit()?;

        if rows_modified == 0 {
            Err(anyhow!(
                "Could not find started lobby with alias '{}'",
                alias
            ))?
        } else {
            Ok(())
        }
    }

    pub fn insert_started_state(
        &self,
        alias: &str,
        started_state: &StartedState,
    ) -> anyhow::Result<()> {
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

    pub fn select_lobbies(&self) -> anyhow::Result<Vec<(GameServer, i32)>> {
        info!("select_lobbies");
        let conn = &*self.0.clone().get()?;
        let mut stmt = conn.prepare(include_str!("db/sql/select_lobbies.sql"))?;
        let foo = stmt.query_map(params![], |ref row| {
            let alias: String = row.get(0).unwrap();
            let maybe_owner: Option<i64> = row.get(1).unwrap();
            let maybe_lobby_era: Option<i32> = row.get(2).unwrap();
            let maybe_player_count: Option<i32> = row.get(3).unwrap();
            let registered_player_count: i32 = row.get(4).unwrap();
            let description: Option<String> = row.get(5).unwrap();
            let server = make_game_server(
                alias,
                None,
                None,
                maybe_owner,
                maybe_lobby_era,
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
    ) -> anyhow::Result<()> {
        info!("update_lobby_with_description");
        let conn = &*self.0.clone().get()?;
        let rows_modified = conn.execute(
            include_str!("db/sql/update_lobby_with_description.sql"),
            &[&alias, &description],
        )?;
        if rows_modified != 0 {
            Ok(())
        } else {
            Err(anyhow!("Could not find lobby with name {}", alias))
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
) -> anyhow::Result<GameServer> {
    let state = match (
        maybe_address,
        maybe_last_seen_turn,
        maybe_owner,
        maybe_player_count,
        maybe_era,
    ) {
        (Some(address), Some(last_seen_turn), None, None, None) => GameServerState::StartedState(
            StartedState {
                address,
                last_seen_turn,
            },
            None,
        ),
        (Some(address), Some(last_seen_turn), Some(owner), Some(player_count), Some(era)) => {
            GameServerState::StartedState(
                StartedState {
                    address,
                    last_seen_turn,
                },
                Some(LobbyState {
                    owner: UserId(owner as u64),
                    era: Era::from_i32(era).ok_or_else(|| anyhow!("unknown era"))?,
                    player_count,
                    description,
                }),
            )
        }
        (None, None, Some(owner), Some(player_count), Some(era)) => {
            GameServerState::Lobby(LobbyState {
                owner: UserId(owner as u64),
                era: Era::from_i32(era).ok_or_else(|| anyhow!("unknown era"))?,
                player_count,
                description,
            })
        }
        _ => return Err(anyhow!("invalid db state for {}", alias)),
    };

    let server = GameServer { alias, state };
    Ok(server)
}
