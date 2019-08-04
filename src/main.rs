mod commands;
#[cfg_attr(test, macro_use)]
mod db;
mod model;
mod server;
mod snek;

use serenity::framework::standard::StandardFramework;
use serenity::prelude::*;
use simplelog::{Config, LogLevelFilter, SimpleLogger};

use failure::*;
use log::*;
use std::env;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::thread;

use crate::db::*;
use crate::server::RealServerConnection;

use commands::servers::GameDetails;
use evmap;

use chrono::{DateTime, Utc};

type WriteHandle = evmap::WriteHandle<String, Box<(DateTime<Utc>, Option<GameDetails>)>>;
type ReadHandle = evmap::ReadHandleFactory<String, Box<(DateTime<Utc>, Option<GameDetails>)>>;

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

    let writer_mutex = Arc::new(Mutex::new(write));
    let writer_mutex_clone = writer_mutex.clone();
    thread::spawn(move || {
        crate::commands::servers::turn_check::update_details_cache_loop(
            db_conn.clone(),
            writer_mutex_clone,
        );
    });
    //    thread::spawn(move || {
    //        crate::commands::servers::turn_check::remove_old_entries_from_cache_loop(writer_mutex);
    //    });

    // start listening for events by starting a single shard
    Ok(discord_client)
}
