mod commands;
// #[cfg_attr(test, macro_use)]
mod db;
mod model;
mod server;
mod snek;

use anyhow::{anyhow, Context as _};
use chrono::{DateTime, Utc};
use log::*;
use serenity::framework::standard::StandardFramework;
use serenity::prelude::*;
use simplelog::{Config, LevelFilter, SimpleLogger};
use std::{env, fs::File, io::Read as _};

use crate::commands::servers::turn_check::update_details_cache_loop;
use crate::db::*;
use model::game_state::CacheEntry;
use std::sync::Arc;

// TODO: should this be im-rc? Do I care really?
pub type DetailsCache = im::HashMap<String, Box<(DateTime<Utc>, Option<CacheEntry>)>>;
struct DetailsCacheKey;
impl TypeMapKey for DetailsCacheKey {
    type Value = DetailsCache;
}
#[derive(Clone)]
pub struct DetailsCacheHandle(Arc<RwLock<TypeMap>>);
impl DetailsCacheHandle {
    // FIXME: cannot return value referencing local variable `read_lock`
    // async fn get(&self, alias: &str) -> anyhow::Result<&CacheEntry> {
    //     let read_lock = self.0.read().await;
    //     let details_cache = read_lock
    //         .get::<DetailsCacheKey>()
    //         .ok_or_else(|| anyhow!("Details cache initialisation error!!!"))?;
    //     let (_, option_cache_entry) = &*details_cache
    //         .get(alias)
    //         .ok_or_else(|| anyhow!("Not yet got a response from server, try again in 1 min"))?
    //         .as_ref();
    //     option_cache_entry
    //         .as_ref()
    //         .ok_or_else(|| anyhow!("Got an error when trying to connect to the server"))
    // }

    // TODO: still feel like this could be done without cloning
    async fn get_clone(&self, alias: &str) -> anyhow::Result<CacheEntry> {
        let read_lock = self.0.read().await;
        let details_cache = read_lock
            .get::<DetailsCacheKey>()
            .ok_or_else(|| anyhow!("Details cache initialisation error!!!"))?;
        let (_, option_cache_entry) = details_cache
            .get(alias)
            .ok_or_else(|| anyhow!("Not yet got a response from server, try again in 1 min"))?
            .as_ref()
            .clone();
        option_cache_entry
            .ok_or_else(|| anyhow!("Got an error when trying to connect to the server"))
    }
}

struct Handler;
impl EventHandler for Handler {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    SimpleLogger::init(LevelFilter::Debug, Config::default()).unwrap();
    info!("Logger initialised");

    let mut discord_client = create_discord_client().await.unwrap();
    info!("Starting discord client");
    discord_client.start().await?;
    error!("Finished discord client");
    Ok(())
}

fn read_token() -> anyhow::Result<String> {
    let mut token_file = File::open("resources/token").context("Opening file 'resources/token'")?;
    let mut temp_token = String::new();
    token_file
        .read_to_string(&mut temp_token)
        .context("Reading contents of file")?;
    info!("Read discord bot token");
    Ok(temp_token)
}
async fn create_discord_client() -> anyhow::Result<Client> {
    let token = read_token().unwrap();

    let path = env::current_dir().unwrap();
    let path = path.join("resources/dom5bot.db");
    let db_conn = DbConnection::new(&path)
        .context(format!("Opening database '{}'", path.display()))
        .unwrap();
    info!("Opened database connection");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .bucket("simple", |b| b.delay(1))
        .await
        .help(&crate::commands::help::HELP)
        .before(|_, msg, _| {
            Box::pin(async move {
                info!("received message {:?}", msg);
                !msg.author.bot // ignore bots
            })
        })
        .after(|ctx, msg, _cmd_name, result| {
            Box::pin(async move {
                if let Err(err) = result {
                    let err = anyhow!(err);
                    let text = format!("ERROR: {}", err);
                    info!("command error: replying with '{}'", text);
                    let _ = msg.reply((&ctx.cache, ctx.http.as_ref()), &text);
                }
            })
        })
        .group(&crate::commands::servers::SERVER_GROUP)
        .group(&crate::commands::search::SEARCH_GROUP);

    let cache_loop_db_conn = db_conn.clone();

    let discord_client = Client::builder(&token)
        .event_handler(Handler)
        .type_map_insert::<DetailsCacheKey>(im::HashMap::new())
        .type_map_insert::<DbConnectionKey>(db_conn)
        .framework(framework)
        .await
        .context("ClientBuilder::await")?;
    info!("Created discord client");

    let write_handle_mutex = DetailsCacheHandle(Arc::clone(&discord_client.data));
    let cache_and_http = Arc::clone(&discord_client.cache_and_http);

    let _ = tokio::spawn(async move {
        update_details_cache_loop(cache_loop_db_conn, write_handle_mutex, cache_and_http).await;
    });

    // start listening for events by starting a single shard
    Ok(discord_client)
}
