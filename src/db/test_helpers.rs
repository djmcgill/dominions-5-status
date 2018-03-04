#![macro_use]

use super::*;

#[macro_export]
macro_rules! mock_server_connection {
    ($struct_name:ident, $ret_val:expr) => {
        struct $struct_name;
        impl ServerConnection for $struct_name {
            fn get_game_data(_: &str) -> io::Result<GameData> {
                $ret_val
            }
        }
    }
}

#[macro_export]
macro_rules! mock_conditional_server_connection {
    ($struct_name:ident, $ret_fn:expr) => {
        struct $struct_name;
        impl ServerConnection for $struct_name {
            fn get_game_data(server_address: &str) -> io::Result<GameData> {
                $ret_fn(server_address)
            }
        }
    }
}

impl DbConnection {
    pub fn test() -> Self {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::builder().max_size(1).build(manager).unwrap();
        let db_conn = DbConnection(pool);
        let result = db_conn.initialise(":memory:");
        println!("TEST DB INITIALISATION: {:?}", result);
        result.unwrap();
        db_conn
    }

    pub fn noop() -> Self {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::new(manager).unwrap();
        DbConnection(pool)
    }

    pub fn count_servers(&self) -> i32 {
        let conn = &*self.0.clone().get().unwrap();
        conn.query_row("SELECT COUNT(*) FROM game_servers", &[], |r| r.get(0))
            .unwrap()
    }

    pub fn count_started_server_state(&self) -> i32 {
        let conn = &*self.0.clone().get().unwrap();
        conn.query_row("SELECT COUNT(*) FROM started_servers", &[], |r| r.get(0))
            .unwrap()
    }

    pub fn count_lobby_state(&self) -> i32 {
        let conn = &*self.0.clone().get().unwrap();
        conn.query_row("SELECT COUNT(*) FROM lobbies", &[], |r| r.get(0))
            .unwrap()
    }
}
