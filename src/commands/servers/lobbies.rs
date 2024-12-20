use crate::commands::servers::CommandResponse;
use crate::{
    db::*,
    model::game_server::{GameServer, GameServerState},
};
use serenity::framework::standard::Args;
use serenity::model::id::{ChannelId, UserId};
use serenity::{builder::CreateEmbed, framework::standard::CommandError, prelude::Context};

pub async fn lobbies(
    context: &Context,
    _channel_id: ChannelId,
    _user_id: UserId,
    _args: Args,
) -> Result<CommandResponse, CommandError> {
    let db_conn = {
        let data = context.data.read().await;
        data.get::<DbConnectionKey>()
            .ok_or_else(|| CommandError::from("No db connection"))?
            .clone()
    };

    let lobbies_and_player_count = db_conn.select_lobbies()?;
    let response = if lobbies_and_player_count.is_empty() {
        CommandResponse::Reply("No available lobbies".to_owned())
    } else {
        let embed = lobbies_helper(lobbies_and_player_count)?;
        CommandResponse::Embed(Box::new(embed))
    };
    Ok(response)
}

fn lobbies_helper(
    lobbies_and_player_count: Vec<(GameServer, i32)>,
) -> Result<CreateEmbed, CommandError> {
    let mut aliases = String::new();
    let mut player_counts = String::new();

    for (lobby, registered_count) in lobbies_and_player_count {
        if let GameServerState::Lobby(state) = &lobby.state {
            let has_hidden_description = state
                .description
                .as_ref()
                .is_some_and(|x| x.starts_with("#hidden"));
            if !has_hidden_description {
                aliases.push_str(&format!("{}\n", lobby.alias));
                player_counts.push_str(&format!("{}/{}\n", registered_count, state.player_count));
            }
        } else {
            aliases.push_str(&format!("{}\n", lobby.alias));
            player_counts.push_str("ERROR");
        }
    }

    Ok(CreateEmbed::default()
        .title("Lobbies")
        .field("Alias", aliases, true)
        .field("Players", player_counts, true))
}
