use super::*;

use std::io;
use model::GameData;

#[test]
fn should_return_error_on_no_connection() {
    struct TestServerConnection;
    impl ServerConnection for TestServerConnection {
        fn get_game_data(_server_address: &str) -> io::Result<GameData> {
            Err(io::Error::from_raw_os_error(-1))
        }
    }

    let result = add_server_helper::<TestServerConnection>("", "", &mut DbConnection::noop());
    assert!(result.is_err());
}

#[test]
fn should_insert_server_into_db() {
    static TEST_ADDRESS: &'static str = "address:1234";
    static TEST_ALIAS: &'static str = ":butts:";

    struct TestServerConnection;
    impl ServerConnection for TestServerConnection {
        fn get_game_data(server_address: &str) -> io::Result<GameData> {
            if server_address == TEST_ADDRESS {
                Ok(GameData {
                    game_name: TEST_ALIAS.to_owned(),
                    nations: Vec::new(),
                    turn: 32,
                    turn_timer: 3 * 360,
                })
            } else {
                Err(io::Error::from_raw_os_error(-1))
            }
        }
    }

    let mut db_conn = DbConnection::test();
    let result = add_server_helper::<TestServerConnection>(&TEST_ADDRESS, &TEST_ALIAS, &mut db_conn);
    println!("{:?}", result);
    assert!(result.is_ok());



}
