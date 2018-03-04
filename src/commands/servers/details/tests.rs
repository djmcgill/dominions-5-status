use super::*;

use std::io;
use model::*;
use model::enums::*;
use serenity::model::*;
use serenity::model::id::UserId;

#[test]
fn should_return_error_on_no_connection() {
    mock_server_connection!(Mock, Err(io::Error::from_raw_os_error(-1)));

    let result = details_helper::<Mock>(&DbConnection::noop(), "");
    assert!(result.is_err());
}

//#[test]
//fn should_return_lobby_details() {
//    let ref db_conn = DbConnection::test();
//    static TEST_ALIAS: &'static str = "foo";
//
//    lazy_static! {
//        static ref TEST_GAMEDATA: GameData = GameData {
//            game_name: TEST_ALIAS.to_owned(),
//            nations: Vec::new(),
//            turn: 32,
//            turn_timer: 3 * 360,
//        };
//    }
//
//    db_conn
//        .insert_game_server(&GameServer {
//            alias: "foo".to_owned(),
//            state: GameServerState::Lobby(LobbyState {
//                owner: UserId(1),
//                era: Era::Early,
//                player_count: 24,
//            }),
//        })
//        .unwrap();
//
//    mock_conditional_server_connection!(Mock, |server_address| {
//        if server_address == "foo" {
//            Ok(TEST_GAMEDATA)
//        } else {
//            Err(io::Error::from_raw_os_error(-1))
//        }
//    });
//
//    let res = details_helper::<Mock>(db_conn, "foo");
//    assert!(res.is_ok());
//
//    let embed = res.unwrap();
//    match embed.0.get(&"fields").unwrap() {
//        Array(ref arr) => assert!(arr.is_empty()),
//        _ => panic!(),
//    }
//}

// FIXME: add these tests
//#[test]
//fn should_return_started_server_details() {}
//
//#[test]
//fn should_return_started_lobby_details() {}
//
//#[test]
//fn should_return_started_game_details() {}
//
//#[test]
//fn should_return_lobby_players() {}
//
//#[test]
//fn should_return_server_players() {}
