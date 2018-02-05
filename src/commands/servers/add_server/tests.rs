use super::*;

use std::io;
use model::GameData;

#[test]
fn should_return_error_on_no_connection() {
    struct TestServerConnection;
    impl ServerConnection for TestServerConnection {
        fn get_game_data(server_address: &str) -> io::Result<GameData> {
            Err(io::Error::from_raw_os_error(-1))
        }
    }

    let result = add_server_helper::<TestServerConnection>("", "", &mut DbConnection::noop());
    assert!(result.is_err())
}
