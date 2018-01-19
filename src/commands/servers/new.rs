// use serenity::framework::standard::{Args, CommandError};
// use serenity::prelude::Context;
// use serenity::model::{Message, UserId};

// use server::get_game_data;
// use model::GameServer;
// use db::{DbConnection, DbConnectionKey};

// pub fn new_server(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
//     let data = context.data.lock();
//     let db_connection = data.get_mut::<DbConnectionKey>().ok_or("No DbConnection was created on startup. This is a bug.")?;

// }
