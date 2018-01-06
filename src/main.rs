
#[macro_use]
extern crate lazy_static;

extern crate byteorder;

extern crate hex_slice;

extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

extern crate flate2;
extern crate url;

use std::fs::File;
use std::io::Read;

#[macro_use]
extern crate serenity;
use serenity::prelude::*;
use serenity::model::*;
use serenity::framework::standard::StandardFramework;
use typemap::ShareMap;
use std::{thread, time};

extern crate typemap;

extern crate bincode;

mod model;
mod commands;
mod server;
mod db;

use std::error::Error;

struct Handler;
impl EventHandler for Handler {
    fn on_ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
    if let Err(e) = do_main() {
        println!("server crashed with error {}", e)
    }
}

fn do_main() -> Result<(), Box<Error>> {
    let token = {
        let mut token_file = File::open("token")?;
        let mut temp_token = String::new();
        token_file.read_to_string(&mut temp_token)?;
        temp_token
    };

    use r2d2_sqlite::SqliteConnectionManager;
    let manager = SqliteConnectionManager::file("C:\\Users\\David\\Documents\\code\\dom5status\\dom5bot.db");
    let pool = r2d2::Pool::new(manager)?;

    let db_conn = db::DbConnection(pool);
    db_conn.initialise()?;

    let mut discord_client = Client::new(&token, Handler);

    {
        let mut data = discord_client.data.lock();
        data.insert::<db::DbConnectionKey>(db_conn);
    }

    discord_client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .simple_bucket("simple", 1)
        .command("ping", |c| c.bucket("simple").exec(commands::ping::ping))
        .command("search", |c| c.bucket("simple").exec(commands::inspector::search))
        .command("servers", |c| c.bucket("simple").exec(commands::servers::servers))
        .command("help", |c| c.bucket("simple").exec(commands::help::help))
        .before(|_, msg, _| {
            println!("received {:?}", msg);
            true
        })
        .after(|_ctx, msg, _cmd_name, result| {
            if let Err(err) = result {
                let text = format!("ERROR: {}", err.0);
                println!("{}", text);
                let _ = msg.reply(&text);
            }
        })
    );

    let foo = discord_client.data.clone();
    thread::spawn(move || {
        check_for_new_turns_every_1_min(foo.as_ref());
    });
    // start listening for events by starting a single shard
    if let Err(why) = discord_client.start() {
        println!("Client error: {:?}", why);
    }
    println!("returning");
    Ok(())
}

fn check_for_new_turns_every_1_min(mutex: &Mutex<ShareMap>) {
    loop {
        thread::sleep(time::Duration::from_secs(60));
        println!("checking for new turns!");
        message_players_if_new_turn(&mutex).unwrap_or_else(|e| {
            println!("Checking for new turns failed with: {}", e);
        });
    }
}

fn message_players_if_new_turn(mutex: &Mutex<ShareMap>) -> Result<(), Box<Error>> {
    let data = mutex.lock();
    let db_conn = data.get::<db::DbConnectionKey>().ok_or("no db connection")?;
    // TODO: transactions
    let servers = db_conn.retrieve_all_servers()?;
    for (_, server) in servers {
        println!("checking {} for new turn", server.alias);
        let game_data = server::get_game_data(&server.address)?;
        let new_turn = db_conn.update_game_with_possibly_new_turn(
            &server.alias,
            game_data.turn
        )?; 

       if new_turn {
            println!("new turn in game {}", server.alias);
            for (_, player, nation_id) in db_conn.players_with_nations_for_game_alias(&server.alias)? {
                // TODO: allow a user to disable PMs
                let &(name, era) = model::enums::nations::get_nation_desc(nation_id);
                let text = format!("your nation {} {} has a new turn in {}",
                    era,
                    name,
                    server.alias.clone());
                println!("{}", text);
                let private_channel = player.discord_user_id.create_dm_channel()?;
                private_channel.say(&text)?;
            }
       }
    }
    Ok(())
}
