use super::*;

use crate::model::*;
use crate::model::enums::*;
use serenity::model::*;
use serenity::model::id::UserId;

#[test]
fn should_remove_started_server() {
    let db_conn = &DbConnection::test();
    let alias = "foo".to_owned();
    db_conn
        .insert_game_server(&GameServer {
            alias: alias.clone(),
            state: GameServerState::StartedState(
                StartedState {
                    address: "foo.bar:3000".to_owned(),
                    last_seen_turn: 23,
                },
                None,
            ),
        })
        .unwrap();

    assert_eq!(db_conn.count_servers(), 1);
    assert_eq!(db_conn.count_started_server_state(), 1);
    assert_eq!(db_conn.count_lobby_state(), 0);

    let result = db_conn.remove_server(&alias);
    assert!(result.is_ok());

    assert_eq!(db_conn.count_servers(), 0);
    assert_eq!(db_conn.count_started_server_state(), 0);
    assert_eq!(db_conn.count_lobby_state(), 0);

    let get_result_err = db_conn.game_for_alias(&alias);
    assert!(get_result_err.is_err());
}

#[test]
fn should_remove_lobby() {
    let db_conn = &DbConnection::test();
    let alias = "foo".to_owned();
    db_conn
        .insert_game_server(&GameServer {
            alias: alias.clone(),
            state: GameServerState::Lobby(LobbyState {
                owner: UserId(1),
                era: Era::Early,
                player_count: 8,
                description: None,
            }),
        })
        .unwrap();


    assert_eq!(db_conn.count_servers(), 1);
    assert_eq!(db_conn.count_started_server_state(), 0);
    assert_eq!(db_conn.count_lobby_state(), 1);

    let result = db_conn.remove_server(&alias);
    assert!(result.is_ok());

    assert_eq!(db_conn.count_servers(), 0);
    assert_eq!(db_conn.count_started_server_state(), 0);
    assert_eq!(db_conn.count_lobby_state(), 0);

    let get_result_err = db_conn.game_for_alias(&alias);
    assert!(get_result_err.is_err());
}

#[test]
fn should_remove_started_server_with_lobby() {
    let db_conn = &DbConnection::test();
    let alias = "foo".to_owned();
    db_conn
        .insert_game_server(&GameServer {
            alias: alias.clone(),
            state: GameServerState::StartedState(
                StartedState {
                    address: "foo.bar:3000".to_owned(),
                    last_seen_turn: 23,
                },
                Some(LobbyState {
                    owner: UserId(1),
                    era: Era::Early,
                    player_count: 8,
                    description: None,
                }),
            ),
        })
        .unwrap();

    assert_eq!(db_conn.count_servers(), 1);
    assert_eq!(db_conn.count_started_server_state(), 1);
    assert_eq!(db_conn.count_lobby_state(), 1);

    let result = db_conn.remove_server(&alias);
    assert!(result.is_ok());

    assert_eq!(db_conn.count_servers(), 0);
    assert_eq!(db_conn.count_started_server_state(), 0);
    assert_eq!(db_conn.count_lobby_state(), 0);

    let get_result_err = db_conn.game_for_alias(&alias);
    assert!(get_result_err.is_err());
}
