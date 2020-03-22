// 2 queues:
// 1) incoming server reads
// 2) outgoing pings
// and 3+N tasks:
// 1) main blocking discord client
// 2) queue 1 reader that writes to the EV map and writes to queue 2
// 3) ping sender
// 4+) X child tcp reads that are spawned every min that write to queue 1
//
//                    1                 2
// X child threads ------> map writer -----> ping sender
//       4+                    2                 1

use crate::db::DbConnection;
use crate::model::game_state::CacheEntry;
use crate::server::ServerConnection;
use std::marker::PhantomData;
use log::error;
use std::time::Duration;
use tokio::task;
use tokio::stream::{StreamExt as _};
use tokio::sync::mpsc;

// Every min or so, create X child threads
pub struct TurnCheckSender<C> {
    db_conn: DbConnection,
    sender: mpsc::Sender<CacheEntry>,
    // see https://github.com/djmcgill/rich_phantoms/blob/master/src/lib.rs#L39
    _phantom: PhantomData<fn() -> C>,
}

// `C` does not need to be `Clone`
impl<C> Clone for TurnCheckSender<C> {
    fn clone(&self) -> Self {
        Self {
            db_conn: self.db_conn.clone(),
            sender: self.sender.clone(),
            _phantom: PhantomData
        }
    }
}

impl<C: ServerConnection + 'static> TurnCheckSender<C> {
    pub fn start_in_background(
            db_conn: DbConnection,
            sender: mpsc::Sender<CacheEntry>) {

        task::spawn(async {
            let turn_check_sender = Self {
                db_conn,
                sender,
                _phantom: PhantomData,
            };
            let mut stream = tokio::time::interval(Duration::from_secs(60));
            while stream.next().await.is_some() {
                Self::tick(turn_check_sender.clone()).await;
            }
        });
    }

    async fn tick(self) {
        let result: anyhow::Result<()> = async {
            let db_conn = self.db_conn;
            let all_servers = task::spawn_blocking(move || {
                db_conn.clone().retrieve_all_servers().map_err(|e| e.compat())
            }).await??;

            let sender = self.sender;
            for server in all_servers {
                task::spawn(
                    Self::spawned_query_server(
                        server.alias.clone(),
                        sender.clone(),
                    )
                );
            }
            Ok(())

        }.await;

        if let Err(e) = result {
            error!("{}", e)
        }
    }

    async fn spawned_query_server(
        server_address: String,
        mut sender: mpsc::Sender<CacheEntry>
    )  {
        let server_address_clone = server_address.clone();
        let result: anyhow::Result<()> = async {
            let game_data = crate::server::get_game_data_async(&server_address_clone).await?;
            let option_snek_state = crate::snek::snek_details_async(&server_address_clone).await?;

            let cache_entry = CacheEntry {
                game_data,
                option_snek_state,
            };
            sender.try_send(cache_entry)?;
            Ok(())
        }.await;

        if let Err(e) = result {
            error!("Could not query server '{}' with error: {}", server_address, e);
        }
    }
}
