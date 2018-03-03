#![allow(unreachable_code)]
#![allow(unused_variables)]

use serenity::builder::CreateEmbed;
use serenity::framework::standard::{Args, CommandError};
use serenity::prelude::Context;
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::utils::parse_username;

use model::Nap;
use model::enums::NapType;
use db::{DbConnection, DbConnectionKey};

pub fn nap(context: &mut Context, message: &Message, mut args: Args) -> Result<(), CommandError> {
    return Err(CommandError::from("NAP commands not implemented yet"));
    // TODO: error if not over pm

    let user_name = args.single_quoted::<String>()?;
    let game_alias = args.single_quoted::<String>()?;
    let nap_type_name = args.single_quoted::<String>()?;
    let nap_type_arg = args.single_quoted::<u32>()?;

    let user_id = parse_username(&user_name).ok_or(CommandError::from("could not find user"))?;
    let players = vec![message.author.id, UserId(user_id)];

    let nap_type = match nap_type_name.as_ref() {
        "fixed" => NapType::Fixed {
            end_turn: nap_type_arg,
        },
        "rolling" => NapType::Rolling {
            notice_length: nap_type_arg,
        },
        _ => return Err(
            CommandError::from("invalid argument, choose either \"fixed\" or \"rolling\"")
        ),
    };

    let data = context.data.lock();
    let db_connection = data.get::<DbConnectionKey>()
        .ok_or("No DbConnection was created on startup. This is a bug.")?;

    nap_helper(db_connection, players, nap_type, &game_alias)?;
    let private_channel = message.author.id.create_dm_channel()?;
    private_channel.say(&format!("Created NAP with {} in {} of type {:?}",
                        UserId(user_id).get()?,
                        game_alias,
                        nap_type,
    ))?;

    Err(CommandError::from("Not implemented yet"))
}

fn nap_helper(
    db_connection: &DbConnection,
    players: Vec<UserId>,
    nap_type: NapType,
    game_alias: &str,
) -> Result<(), CommandError> {
    let nap = Nap {
        nap_type: nap_type,
        players: players,
        game_alias: game_alias.to_owned(),
    };

    db_connection.insert_nap(&nap)?;
    Ok(())
}

pub fn naps(context: &mut Context, message: &Message) -> Result<(), CommandError> {
    return Err(CommandError::from("NAP commands not implemented yet"));
    let data = context.data.lock();
    let db_connection = data.get::<DbConnectionKey>()
        .ok_or("No DbConnection was created on startup. This is a bug.")?;

    let embed_response = naps_helper(db_connection, message.author.id)?;
    let private_channel = message.author.id.create_dm_channel()?;
    private_channel.send_message(|m| m.embed(|_| embed_response))?;
    Ok(())
}

fn naps_helper(
    db_connection: &DbConnection,
    author: UserId,
) -> Result<CreateEmbed, CommandError> {
    let naps = db_connection.select_naps(author)?;

    Err(CommandError::from("Not implemented yet"))
}

// DB
// TABLE naps
// id primary key,
// started_server non-null foreign key
// type (fixed = 1, rolling = 2),
// end_turn (null for rolling unless on notice)

// TABLE player_naps
// id primary key,
// nap_id non-null foreign key
// player_id non-null foreign key
