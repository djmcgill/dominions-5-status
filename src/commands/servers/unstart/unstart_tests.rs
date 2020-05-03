use super::*;
use crate::model::enums::Era;
use serenity::model::id::UserId;

#[test]
fn remove_started_server() {
    let db_conn = &DbConnection::test();
    let alias = "foo".to_owned();
    db_conn
        .insert_game_server(&GameServer {
            alias: alias.clone(),
            state: GameServerState::StartedState(
                StartedState {
                    address: "foo.bar:3000".to_owned(),
                    last_seen_turn: 23,
                    era: Era::Early,
                },
                Some(LobbyState {
                    owner: UserId(0),
                    era: Era::Early,
                    player_count: 12,
                    description: None,
                }),
            ),
        })
        .unwrap();

    db_conn
        .insert_game_server(&GameServer {
            alias: "foo2".to_owned(),
            state: GameServerState::StartedState(
                StartedState {
                    address: "foo2.bar:3000".to_owned(),
                    last_seen_turn: 23,
                    era: Era::Early,
                },
                Some(LobbyState {
                    owner: UserId(0),
                    era: Era::Early,
                    player_count: 12,
                    description: None,
                }),
            ),
        })
        .unwrap();

    db_conn.remove_started_state(&alias).unwrap();

    let result = db_conn.game_for_alias(&alias).unwrap();
    if let GameServerState::StartedState(_, _) = result.state {
        panic!("Did not stop started state!")
    }

    if let GameServerState::Lobby(_) = db_conn.game_for_alias("foo2").unwrap().state {
        panic!("messed with a different game!");
    }
}

#[test]
fn should_error_when_game_not_found() {
    let db_conn = &DbConnection::test();
    let alias = "foo".to_owned();

    db_conn
        .insert_game_server(&GameServer {
            alias: "foo2".to_owned(),
            state: GameServerState::StartedState(
                StartedState {
                    address: "foo2.bar:3000".to_owned(),
                    last_seen_turn: 23,
                    era: Era::Early,
                },
                Some(LobbyState {
                    owner: UserId(0),
                    era: Era::Early,
                    player_count: 12,
                    description: None,
                }),
            ),
        })
        .unwrap();

    assert!(db_conn.remove_started_state(&alias).is_err());

    if let GameServerState::Lobby(_) = db_conn.game_for_alias("foo2").unwrap().state {
        panic!("messed with a different game!");
    }
}

#[test]
fn should_error_when_game_not_lobby() {
    let db_conn = &DbConnection::test();
    let alias = "foo".to_owned();
    db_conn
        .insert_game_server(&GameServer {
            alias: alias.clone(),
            state: GameServerState::StartedState(
                StartedState {
                    address: "foo.bar:3000".to_owned(),
                    last_seen_turn: 23,
                    era: Era::Early,
                },
                None,
            ),
        })
        .unwrap();

    db_conn
        .insert_game_server(&GameServer {
            alias: "foo2".to_owned(),
            state: GameServerState::StartedState(
                StartedState {
                    address: "foo2.bar:3000".to_owned(),
                    last_seen_turn: 23,
                    era: Era::Early,
                },
                Some(LobbyState {
                    owner: UserId(0),
                    era: Era::Early,
                    player_count: 12,
                    description: None,
                }),
            ),
        })
        .unwrap();

    assert!(db_conn.remove_started_state(&alias).is_err());

    if let GameServerState::Lobby(_) = db_conn.game_for_alias("foo2").unwrap().state {
        panic!("messed with a different game!");
    }
}

#[test]
fn should_not_error_when_game_not_started() {
    let db_conn = &DbConnection::test();
    let alias = "foo".to_owned();
    db_conn
        .insert_game_server(&GameServer {
            alias: alias.clone(),
            state: GameServerState::Lobby(LobbyState {
                owner: UserId(0),
                era: Era::Early,
                player_count: 12,
                description: None,
            }),
        })
        .unwrap();

    db_conn
        .insert_game_server(&GameServer {
            alias: "foo2".to_owned(),
            state: GameServerState::StartedState(
                StartedState {
                    address: "foo2.bar:3000".to_owned(),
                    last_seen_turn: 23,
                    era: Era::Early,
                },
                Some(LobbyState {
                    owner: UserId(0),
                    era: Era::Early,
                    player_count: 12,
                    description: None,
                }),
            ),
        })
        .unwrap();

    db_conn.remove_started_state(&alias).unwrap();

    if let GameServerState::Lobby(_) = db_conn.game_for_alias("foo2").unwrap().state {
        panic!("messed with a different game!");
    }
}
