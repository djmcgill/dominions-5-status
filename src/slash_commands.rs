use crate::commands::servers::{
    add_server::add_server,
    alias::server_set_alias,
    describe::describe,
    details::details,
    kick::kick_player,
    list_servers::list_servers,
    lobbies::lobbies,
    lobby::lobby,
    notifications::notifications,
    register_player::{register_player, register_player_custom, register_player_id},
    remove_server::remove_server,
    start::start,
    turns::turns,
    unregister_player::unregister_player,
    unstart::unstart,
    CommandResponse,
};
use anyhow::{anyhow, Context as _};
use log::{error, info};
use serenity::all::{CommandDataOptionValue, CreateCommand};
use serenity::{
    all::{
        CommandData, CommandOptionType, CreateInteractionResponse,
        CreateInteractionResponseMessage, Interaction,
    },
    builder::CreateCommandOption,
    client::Context,
    framework::standard::{Args, Delimiter},
    http::{CacheHttp, Http},
};

// This technically only needs to be run once, but running every time on boot
// just overrides it each time and guild commands update instantly so who cares.
pub async fn create_guild_commands(http: &Http) -> anyhow::Result<()> {
    let guilds = http
        // The bot is only ever in one server per instance, so yolo
        .get_guilds(None, Some(1))
        .await?;
    let guild = guilds
        .first()
        .ok_or_else(|| anyhow!("Bot user is in 0 guilds"))?;

    guild
        .id
        .set_commands(
            http,
            vec![
                CreateCommand::new("details")
                    .description("Display the details of a game or lobby")
                    .add_option(game_name_option()),
                CreateCommand::new("deets")
                    .description("Display the deets of a game or lobby")
                    .add_option(game_name_option()),
                CreateCommand::new("add")
                    .description("Add a running game to the bot")
                    .add_option(
                        CreateCommandOption::new(
                            CommandOptionType::String,
                            "address_and_port",
                            "The address and port, separated by a colon, to connect to the game",
                        )
                        .required(true),
                    )
                    .add_option(game_name_option()),
                CreateCommand::new("list").description("List all games and lobbies"),
                CreateCommand::new("delete")
                    .description("Remove a game or lobby from the bot")
                    .add_option(game_name_option()),
                CreateCommand::new("register")
                    .description("Register yourself as a nation in a game or lobby")
                    .add_option(CreateCommandOption::new(
                        CommandOptionType::String,
                        "nation_name_prefix",
                        "The start (or all) of a nation name e.g. `tien`, `eatie`, `\"T'ien Ch'i\"`, or `\"EA T'ien Ch'i\""
                    ).required(true))
                    .add_option(game_name_option()),
                CreateCommand::new("register-id")
                    .description("Register yourself as a nation using the ID in a game or lobby")
                    .add_option(CreateCommandOption::new(CommandOptionType::Integer, "nation_id", "The ID of the nation, e.g. `10` for EA TC"))
                    .add_option(game_name_option()),
                CreateCommand::new("register-custom")
                    .description("Register yourself as some text in a mod game or lobby. You must reregister after uploading.")
                    .add_option(CreateCommandOption::new(CommandOptionType::String, "nation_text", "Some text to represent your chosen nation").required(true))
                    .add_option(game_name_option()),
                CreateCommand::new("join")
                    .description("Join a game or lobby")
                    .add_option(CreateCommandOption::new(
                        CommandOptionType::String,
                        "nation_name_prefix",
                        "The start (or all) of a nation name e.g. `tien`, `eatie`, `\"T'ien Ch'i\"`, or `\"EA T'ien Ch'i\""
                    ).required(true))
                    .add_option(game_name_option()),
                CreateCommand::new("join-id")
                    .description("Join a game or lobby using a nation ID")
                    .add_option(CreateCommandOption::new(CommandOptionType::Integer, "nation_id", "The ID of the nation, e.g. `10` for EA TC"))
                    .add_option(game_name_option()),
                CreateCommand::new("join-custom")
                    .description("Join a mod game or lobby. You must reregister after uploading.")
                    .add_option(CreateCommandOption::new(CommandOptionType::String, "nation_text", "Some text to represent your chosen nation.").required(true))
                    .add_option(game_name_option()),
                CreateCommand::new("unregister")
                    .description("Unregister yourself from all nations in a game or lobby")
                    .add_option(game_name_option()),
                CreateCommand::new("turns")
                    .description("Have the bot PM you with the details of all games you're currently in."),
                CreateCommand::new("notifications")
                    .description("Disable, or re-enable, the bot pinging you for turns.")
                    .add_option(CreateCommandOption::new(CommandOptionType::Boolean, "enabled", "If true, then notifications are sent.").required(true)),
                CreateCommand::new("lobby")
                    .description("Create a lobby with no server for players to join")
                    .add_option(CreateCommandOption::new(CommandOptionType::String, "era", "The era of the game (EA/MA/LA)")
                        .add_string_choice("EA", "EA")
                        .add_string_choice("MA", "MA")
                        .add_string_choice("LA", "LA")
                        .required(true)
                    )
                    .add_option(CreateCommandOption::new(CommandOptionType::Integer, "number_of_players", "The number of players").required(true))
                    .add_option(game_name_option()),
                CreateCommand::new("lobbies")
                    .description("List available lobbies"),
                CreateCommand::new("start")
                    .description("Start a lobby with an address, after starting it on the server")
                    .add_option(CreateCommandOption::new(CommandOptionType::String, "address", "Either \"web.site:1234\" or \"www.illwinter.com/mygame.html\"").required(true))
                    .add_option(game_name_option()),
            CreateCommand::new("describe")
                    .description("Add a description to a lobby. Quotemarks required.")
                    .add_option(CreateCommandOption::new(CommandOptionType::String, "description", "The description to add to the game. Quotemarks required.").required(true))
                    .add_option(game_name_option()),
            CreateCommand::new("unstart")
                    .description("Turn a game back into a lobby by forgetting its address.")
                    .add_option(game_name_option()),
            CreateCommand::new("alias")
                    .description("Set a new alias for a server.")
                    .add_option(CreateCommandOption::new(CommandOptionType::String, "alias", "The current alias for the server").required(true))
                    .add_option(CreateCommandOption::new(CommandOptionType::String, "new_alias", "The new alias for the server. If not present, will use the channel name.")),
            CreateCommand::new("banish")
                    .description("Kick a user from a game.")
                    .add_option(CreateCommandOption::new(CommandOptionType::User, "user", "The user to kick from this game").required(true))
                    .add_option(game_name_option()),
            ],
        )
        .await
        .context("create_application_commands")?;
    Ok(())
}

fn game_name_option() -> CreateCommandOption {
    CreateCommandOption::new(
        CommandOptionType::String,
        "name",
        "The game's bot name. If not present, will use the channel name.",
    )
}

pub async fn interaction_create(ctx: Context, interaction: Interaction) {
    if let Err(e) = interaction_create_result(ctx, interaction).await {
        error!("Failed to create interaction: {:#?}", e);
    }
}

async fn interaction_create_result(ctx: Context, interaction: Interaction) -> anyhow::Result<()> {
    info!("Incoming interaction: {:?}", interaction);

    if let Interaction::Command(interaction) = interaction {
        let channel_id = interaction.channel_id;
        let user_id = interaction
            .member
            .as_ref()
            .ok_or_else(|| anyhow!("No member in interaction_create"))?
            .user
            .id;

        let data = &interaction.data;

        let args = make_args(data)?;

        // This is quite cumbersome, it would be better to integrate with the serenity framework
        let command_response_result = match data.name.as_str() {
            // all new servers are for dom6
            "add" => add_server(&ctx, channel_id, user_id, 6, args)
                .await
                .map_err(|e| anyhow!("ddd_server slash command failed with: {}", e)),
            "describe" => describe(&ctx, channel_id, user_id, args)
                .await
                .map_err(|e| anyhow!("describe slash command failed with: {}", e)),
            "details" | "deets" => details(&ctx, channel_id, user_id, args)
                .await
                .map_err(|e| anyhow!("details slash command failed with: {}", e)),
            "list" => list_servers(&ctx, channel_id, user_id, args)
                .await
                .map_err(|e| anyhow!("list slash command failed with: {}", e)),
            "lobbies" => lobbies(&ctx, channel_id, user_id, args)
                .await
                .map_err(|e| anyhow!("lobbies slash command failed with: {}", e)),
            "lobby" => lobby(&ctx, channel_id, user_id, args)
                .await
                .map_err(|e| anyhow!("lobby slash command failed with: {}", e)),
            "notifications" => notifications(&ctx, channel_id, user_id, args)
                .await
                .map_err(|e| anyhow!("notifications slash command failed with: {}", e)),
            "register" => register_player(&ctx, channel_id, user_id, args)
                .await
                .map_err(|e| anyhow!("register slash command failed with: {}", e)),
            "register_id" => register_player_id(&ctx, channel_id, user_id, args)
                .await
                .map_err(|e| anyhow!("register_id slash command failed with: {}", e)),
            "register_custom" => register_player_custom(&ctx, channel_id, user_id, args)
                .await
                .map_err(|e| anyhow!("register_custom slash command failed with: {}", e)),
            "delete" => remove_server(&ctx, channel_id, user_id, args)
                .await
                .map_err(|e| anyhow!("delete slash command failed with: {}", e)),
            "start" => start(&ctx, channel_id, user_id, args)
                .await
                .map_err(|e| anyhow!("start slash command failed with: {}", e)),
            "turns" => turns(&ctx, channel_id, user_id, args)
                .await
                .map_err(|e| anyhow!("turns slash command failed with: {}", e)),
            "unregister" => unregister_player(&ctx, channel_id, user_id, args)
                .await
                .map_err(|e| anyhow!("unregister slash command failed with: {}", e)),
            "unstart" => unstart(&ctx, channel_id, user_id, args)
                .await
                .map_err(|e| anyhow!("unstart slash command failed with: {}", e)),
            "alias" => server_set_alias(&ctx, channel_id, user_id, args)
                .await
                .map_err(|e| anyhow!("alias slash command failed with: {}", e)),
            "banish" => kick_player(&ctx, channel_id, user_id, args)
                .await
                .map_err(|e| anyhow!("banish slash command failed with: {}", e)),
            other => Err(anyhow!("Unrecognised command: {}", other)),
        };
        match command_response_result {
            Ok(command_response) => {
                interaction
                    .create_response(
                        ctx.http(),
                        CreateInteractionResponse::Message(match command_response {
                            CommandResponse::Reply(reply) => {
                                CreateInteractionResponseMessage::new().content(reply)
                            }
                            CommandResponse::Embed(embed) => {
                                CreateInteractionResponseMessage::new().embed(*embed)
                            }
                        }),
                    )
                    .await?;
            }
            Err(err) => {
                interaction
                    .create_response(
                        ctx.http(),
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new().content({
                                let err = anyhow!(err);
                                let text = format!("ERROR: {}", err);
                                info!("command error: replying with '{}'", text);
                                text
                            }),
                        ),
                    )
                    .await?
            }
        }
    }
    Ok(())
}

// okay this is VERY hacky. We're going via the delimited string for no reason at all.
// Unfortunately to fix it I'd have to do some big tokenize or pre-parse or whatever, and possibly
// even switching away from serenity's standard framework and I really can't be bothered.
fn make_args(data: &CommandData) -> anyhow::Result<Args> {
    let mut arg_string = String::new();
    for option in &data.options {
        let str_option = match &option.value {
            CommandDataOptionValue::String(x) => format!("{} ", x),
            CommandDataOptionValue::Boolean(x) => format!("{} ", x),
            CommandDataOptionValue::Integer(x) => format!("{} ", x),
            CommandDataOptionValue::User(x) => format!("{} ", x.get()),
            _ => {
                return Err(anyhow!(
                    "Unsupported CommandDataOptionValue: {:?}",
                    option.value
                ))
            }
        };

        arg_string.push_str(&format!("{} ", str_option));
    }
    Ok(Args::new(&arg_string, &[Delimiter::Single(' ')]))
}
