// warning: use of deprecated associated function `serenity::framework::standard::*`:
// The standard framework is deprecated, and will be removed in 0.13. Please migrate to `poise` for command handling
#![allow(deprecated)]
mod commands;
mod db;
mod model;
mod server;
mod slash_commands;
mod snek;

use crate::{
    commands::servers::turn_check::update_details_cache_loop, db::*, model::game_state::CacheEntry,
};
use anyhow::{anyhow, Context as _};
use chrono::{DateTime, Utc};
use log::*;
use serenity::all::standard::BucketBuilder;
use serenity::all::{ApplicationId, Interaction};
use serenity::framework::standard::Configuration;
use serenity::{async_trait, framework::standard::StandardFramework, prelude::*};
use simplelog::{Config, LevelFilter, SimpleLogger};
use std::time::Duration;
use std::{env, fs::File, io::Read as _, path::Path, str::FromStr, sync::Arc};

pub const SERVER_POLL_INTERVAL: Duration = Duration::from_secs(60);

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
            .ok_or_else(|| {
                anyhow!(
                    "Not yet got a response from server for game '{}', try again in 1 min",
                    alias
                )
            })?
            .as_ref()
            .clone();
        option_cache_entry.ok_or_else(|| {
            anyhow!(
                "Got an error when trying to connect to the server for game '{}'",
                alias
            )
        })
    }
}

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        slash_commands::interaction_create(ctx, interaction).await
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    SimpleLogger::init(LevelFilter::Warn, Config::default())?;
    info!("Logger initialised");

    let mut discord_client = create_discord_client().await?;
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
    let token_str = temp_token.trim();
    info!("Read discord bot token");
    Ok(token_str.to_owned())
}

fn read_application_id() -> anyhow::Result<Option<ApplicationId>> {
    let application_path = Path::new("resources/application");
    if !application_path.exists() {
        return Ok(None);
    }
    let mut token_file = File::open(application_path).context("Opening application path file")?;
    let mut temp_token = String::new();
    token_file
        .read_to_string(&mut temp_token)
        .context("Reading contents of file")?;
    let token_str = temp_token.trim();
    info!("Read discord application id");

    let uint = u64::from_str(token_str).context("u64::from_str")?;
    if uint == 0 {
        Err(anyhow!("Invalid UserId of 0"))
    } else {
        Ok(Some(ApplicationId::from(uint)))
    }
}

async fn create_discord_client() -> anyhow::Result<Client> {
    let token = read_token()?;
    let option_application_id = read_application_id()?;

    let path = env::current_dir()?;
    let path = path.join("resources/dom5bot.db");
    let db_conn =
        DbConnection::new(&path).context(format!("Opening database '{}'", path.display()))?;
    info!("Opened database connection");

    let framework = StandardFramework::new()
        .bucket("simple", BucketBuilder::default().delay(1))
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
                    let text = format!("ERROR: {:?}", err);
                    info!("command error: replying with '{}'", text);
                    let _ = msg.reply((&ctx.cache, ctx.http.as_ref()), &text).await;
                }
            })
        })
        .group(&crate::commands::servers::SERVER_GROUP);
    framework.configure(Configuration::new().prefix("!"));

    let cache_loop_db_conn = db_conn.clone();

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut discord_client_builder = Client::builder(&token, intents);

    if let Some(&application_id) = option_application_id.as_ref() {
        discord_client_builder = discord_client_builder.application_id(application_id);
    }

    let discord_client = discord_client_builder
        .event_handler(Handler)
        .type_map_insert::<DetailsCacheKey>(im::HashMap::new())
        .type_map_insert::<DbConnectionKey>(db_conn)
        .framework(framework)
        .await
        .context("ClientBuilder::await")?;
    info!("Created discord client");

    let write_handle_mutex = DetailsCacheHandle(Arc::clone(&discord_client.data));
    let cache = Arc::clone(&discord_client.cache);
    let http = Arc::clone(&discord_client.http);

    tokio::spawn(async move {
        update_details_cache_loop(cache_loop_db_conn, write_handle_mutex, (cache, http)).await;
    });

    if option_application_id.is_some() {
        slash_commands::create_guild_commands(discord_client.http.as_ref())
            .await
            .context("create_guild_commands")?;
    }

    // start listening for events by starting a single shard
    Ok(discord_client)
}
