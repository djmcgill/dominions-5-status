use crate::commands::servers::CommandResponse;
use crate::{commands::servers::alias_from_arg_or_channel_name, db::DbConnectionKey};
use serenity::{
    framework::standard::{Args, CommandError},
    model::id::{ChannelId, UserId},
    prelude::Context,
};

pub async fn server_set_alias(
    context: &Context,
    channel_id: ChannelId,
    user_id: UserId,
    mut args: Args,
) -> Result<CommandResponse, CommandError> {
    let db_conn = {
        let data = context.data.read().await;
        data.get::<DbConnectionKey>()
            .ok_or("No DbConnection was created on startup. This is a bug.")?
            .clone()
    };

    let alias = alias_from_arg_or_channel_name(context, channel_id, &mut args).await?;
    let new_alias = args.single_quoted::<String>()?;
    if !args.is_empty() {
        return Err(CommandError::from("Too many arguments."));
    }

    db_conn.update_lobby_with_alias(&alias, &new_alias)?;
    Ok(CommandResponse::Reply(format!(
        "Updated alias to {}",
        alias
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::servers::CommandResponse;
    use crate::db::DbConnection;
    use crate::{commands::servers::alias_from_arg_or_channel_name, db::DbConnectionKey};
    use crate::{DetailsCacheKey, Handler};
    use anyhow::Context;
    use serenity::prelude::TypeMap;
    use serenity::{
        client::Client,
        framework::standard::{Args, CommandResult},
        model::id::{ChannelId, UserId},
    };

    #[tokio::test]
    async fn test_server_set_alias() {
        // TODO: blocked because of https://github.com/jaemk/migrant_lib/issues/6
        // can't use SQLite in memory with migrant_lib

        // let db_conn = DbConnection::newInMemory().unwrap();
        // let client_builder = Client::builder("token");
        // let client = client_builder
        //     .event_handler(Handler)
        //     .type_map_insert::<DetailsCacheKey>(im::HashMap::new())
        //     .type_map_insert::<DbConnectionKey>(db_conn)
        //     // .framework(framework)
        //     .await
        //     .context("ClientBuilder::await");

        // let mut data = serenity::data::DataMap::new();
        // data.insert(
        //     DbConnectionKey,
        //     DbConnectionKey(rusqlite::Connection::open_in_memory().unwrap()),
        // );

        // let context = Context {
        //     data: Arc::new(Mutex::new(data)),
        //     ..Context::default()
        // };

        // let channel_id = ChannelId(1);
        // let user_id = UserId(2);

        // let mut args = Args::new(&["test", "test"]);
        // let result = server_set_alias(&context, channel_id, user_id, args).await;
        // // assert_eq!(
        // //     result.unwrap(),
        // //     CommandResponse
    }
}
