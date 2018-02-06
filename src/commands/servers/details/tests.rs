use super::*;

use std::io;
use model::*;

#[test]
fn should_return_error_on_no_connection() {
    struct TestServerConnection;
    impl ServerConnection for TestServerConnection {
        fn get_game_data(_server_address: &str) -> io::Result<GameData> {
            Err(io::Error::from_raw_os_error(-1))
        }
    }

    let result = details_helper::<TestServerConnection>(&DbConnection::noop(), "");
    assert!(result.is_err());
}
