use crate::db::DbConnection;
use tokio::sync::mpsc;
use tokio::stream::{StreamExt as _};
use tokio::task;
use crate::model::game_state::CacheEntry;
use crate::model::turn_notification::TurnNotification;
use log::error;

pub fn start_receiver_thread_in_background(
    db_conn: DbConnection,
    mut server_state_receiver: mpsc::Receiver<CacheEntry>,
    notification_sender: mpsc::Sender<TurnNotification>,
) {
    task::spawn( async move {
        while let Some(server_state) = server_state_receiver.next().await {
            if let Err(e) = update_server_state(server_state, db_conn.clone(), notification_sender.clone()).await {
                error!("Could not update server state: {}", e);
            }
        }
    });
}

async fn update_server_state(
    server_state: CacheEntry,
    db_conn: DbConnection,
    notification_sender: mpsc::Sender<TurnNotification>,) -> anyhow::Result<()> {
        let new_turn_number = server_state.game_data.turn;
        let game_alias = server_state.game_data.game_name;

        let db_conn_clone =  db_conn.clone();
        let turn_updated_result = task::spawn_blocking(move ||
            db_conn_clone.update_game_with_possibly_new_turn(&game_alias, new_turn_number)
        ).await;

        let turn_updated = turn_updated_result.unwrap().unwrap(); // FIXME
        if turn_updated {
            for nation in server_state.game_data.nations {
                let turn_notification: TurnNotification = unimplemented!();
                notification_sender.clone().try_send(turn_notification).unwrap(); // FIXME
            }
        }
        Ok(())
    }
