mod commands;
#[cfg_attr(test, macro_use)]
mod db;
mod model;
mod server;
mod snek;

#[cfg(test)]
pub mod it;

use serenity::framework::standard::StandardFramework;
use serenity::prelude::*;
use simplelog::{Config, LogLevelFilter, SimpleLogger};

use failure::*;
use log::*;
use std::env;
use std::fs::File;
use std::io::Read;
use std::thread;

use crate::db::*;
use crate::server::RealServerConnection;

use commands::servers::GameDetails;
use evmap;

use serenity::framework::standard::CommandError;

use chrono::{DateTime, Duration, Utc};
use lazy_static::lazy_static;

use crate::commands::servers::get_details_for_alias;
use std::collections::HashSet;
use std::time;

type WriteHandle = evmap::WriteHandle<String, Box<(DateTime<Utc>, Option<GameDetails>)>>;
type ReadHandle = evmap::ReadHandleFactory<String, Box<(DateTime<Utc>, Option<GameDetails>)>>;

pub fn update_details_cache_loop(db_conn: DbConnection, write_handle_mutex: &Mutex<WriteHandle>) {
    loop {
        info!("Checking for new turns!");
        for mut write_handle in write_handle_mutex.try_lock() {
            update_details_cache_for_all_games(&db_conn, &mut write_handle);
        }
        thread::sleep(time::Duration::from_secs(60));
    }
}

// FIXME: should just be regular error
fn update_details_cache_for_game(
    alias: &str,
    db_conn: &DbConnection,
    write_handle: &mut WriteHandle,
) -> Result<(), CommandError> {
    info!("Checking turn for {}", alias);

    let result_details = get_details_for_alias::<RealServerConnection>(db_conn, alias);
    let now = Utc::now();
    match result_details {
        Err(e) => {
            error!(
                "Got an error when checking for details for alias {}: {:?}",
                alias, e
            );
            write_handle.update(alias.to_owned(), Box::new((now, None)));
        }
        Ok(details) => {
            write_handle.update(alias.to_owned(), Box::new((now, Some(details))));
        }
    }

    // FIXME: might just want to store the hash
    info!("Checking turn for {}: SUCCESS", alias);
    Ok(())
}

fn update_details_cache_for_all_games(db_conn: &DbConnection, write_handle: &mut WriteHandle) {
    match db_conn.retrieve_all_servers() {
        Err(e) => {
            error!("Could not query the db for all servers with error: {:?}", e);
        }
        Ok(servers) => {
            // FIXME: might want to parallelise
            for server in servers {
                match update_details_cache_for_game(&server.alias, db_conn, write_handle) {
                    Ok(()) => {}
                    Err(e) => {
                        error!("Could not update game {} with error {:?}", server.alias, e);
                    }
                }
            }
            write_handle.refresh();
        }
    }
}

fn remove_old_entries_from_cache(write_handle: &mut WriteHandle) {
    let values: Vec<String> = write_handle.map_into(|key, _| key.clone());

    for value in values {
        write_handle.retain(value.clone(), move |box_value| {
            let (ref last_updated, ref details) = **box_value;
            let two_hours_ago = Utc::now().checked_sub_signed(Duration::hours(2)).unwrap();

            // If we've updated successfully more recently than 2 hours ago, keep it.
            let out_of_date = last_updated > &two_hours_ago;
            if out_of_date {
                info!("Removing game {} from cache", value);
            };
            !out_of_date
        });
    }
}

struct Handler;
impl EventHandler for Handler {}

fn main() {
    if let Err(e) = do_main() {
        info!("server crashed with error {:?}", e)
    }
}

fn do_main() -> Result<(), Error> {
    SimpleLogger::init(LogLevelFilter::Info, Config::default())?;
    info!("Logger initialised");

    let mut discord_client = create_discord_client().context("Creating discord client")?;
    if let Err(why) = discord_client.start() {
        error!("Client error: {:?}", why);
    }
    Ok(())
}

fn read_token() -> Result<String, Error> {
    let mut token_file = File::open("resources/token").context("Opening file 'resources/token'")?;
    let mut temp_token = String::new();
    token_file
        .read_to_string(&mut temp_token)
        .context("Reading contents of file")?;
    info!("Read discord bot token");
    Ok(temp_token)
}

struct DetailsReadHandleKey;
impl typemap::Key for DetailsReadHandleKey {
    type Value = ReadHandle;
}

fn create_discord_client() -> Result<Client, Error> {
    let token = read_token().context("Reading token file")?;

    let path = env::current_dir()?;
    let path = path.join("resources/dom5bot.db");
    let db_conn =
        DbConnection::new(&path).context(format!("Opening database '{}'", path.display()))?;
    info!("Opened database connection");

    let (reader, write) = evmap::new();

    let mut discord_client = Client::new(&token, Handler).map_err(SyncFailure::new)?;
    info!("Created discord client");
    {
        let mut data = discord_client.data.lock();
        data.insert::<DbConnectionKey>(db_conn.clone());
        data.insert::<DetailsReadHandleKey>(reader.factory());
    }

    use crate::commands::servers::WithServersCommands;
    use crate::commands::WithSearchCommands;
    discord_client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("!"))
            .simple_bucket("simple", 1)
            .with_search_commands("simple")
            .with_servers_commands::<RealServerConnection>("simple")
            .help(|_, msg, _, _, _| commands::help(msg))
            .before(|_, msg, _| {
                info!("received message {:?}", msg);
                !msg.author.bot // ignore bots
            })
            .after(|_ctx, msg, _cmd_name, result| {
                if let Err(err) = result {
                    print!("command error: ");
                    let text = format!("ERROR: {}", err.0);
                    info!("replying with {}", text);
                    let _ = msg.reply(&text);
                }
            }),
    );
    info!("Configured discord client");

    let data_clone = discord_client.data.clone();
    let writer_mutex = Mutex::new(write);
    thread::spawn(move || {
        update_details_cache_loop(db_conn.clone(), &writer_mutex);
    });

    // start listening for events by starting a single shard
    Ok(discord_client)
}
