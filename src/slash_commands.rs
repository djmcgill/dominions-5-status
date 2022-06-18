use crate::commands::servers::{
    add_server::add_server,
    describe::describe,
    details::details,
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
use serenity::{
    builder::CreateApplicationCommandOption,
    client::Context,
    framework::standard::{Args, Delimiter},
    http::{CacheHttp, Http},
    model::interactions::{
        application_command::{ApplicationCommandInteractionData, ApplicationCommandOptionType},
        Interaction, InteractionResponseType,
    },
};

// This technically only needs to be run once, but running every time on boot
// just overrides it each time and guild commands update instantly so who cares.
pub async fn create_guild_commands(http: &Http) -> anyhow::Result<()> {
    let guilds = http
        // The bot is only ever in one server per instance, so yolo
        .get_guilds(None, Some(1))
        .await?;
    let guild = guilds
        .get(0)
        .ok_or_else(|| anyhow!("Bot user is in 0 guilds"))?;

    guild
        .id
        .set_application_commands(http, |cs| {
            cs
                .create_application_command(|c| {
                    c.name("details")
                        .description("Display the details of a game or lobby")
                        .create_option(game_name_option)
                })
                .create_application_command(|c| {
                    c.name("deets")
                        .description("Display the deets of a game or lobby")
                        .create_option(game_name_option)
                })
                .create_application_command(|c| {
                    c.name("add")
                        .description("Add a running game to the bot")
                        .create_option(|o| {
                            o.kind(ApplicationCommandOptionType::String)
                                .name("address_and_port")
                                .description("The address and port, separated by a colon, to connect to the game")
                                .required(true)
                        })
                        .create_option(game_name_option)
                })
                .create_application_command(|c|
                    c.name("list")
                        .description("List all games and lobbies")
                )
                .create_application_command(|c|
                    c.name("delete")
                        .description("Remove a game or lobby from the bot")
                        .create_option(game_name_option)
                )
                .create_application_command(|c|
                    c.name("register")
                        .description("Register yourself as a nation in a game or lobby")
                        .create_option(|o|
                            o.kind(ApplicationCommandOptionType::String)
                                .name("nation_name_prefix")
                                .description("The start (or all) of a nation name e.g. `tien`, `eatie`, `\"T'ien Ch'i\"`, or `\"EA T'ien Ch'i\"")
                                .required(true)
                        )
                        .create_option(game_name_option)
                )
                .create_application_command(|c|
                    c.name("register_id")
                        .description("Register yourself as a nation using the ID in a game or lobby")
                        .create_option(|o|
                            o.kind(ApplicationCommandOptionType::Integer)
                                .name("nation_id")
                                .description("The ID of the nation, e.g. `10` for EA TC")
                                .required(true)
                        )
                        .create_option(game_name_option)
                )
                .create_application_command(|c|
                    c.name("register_custom")
                        .description("Register yourself as some text in a game or lobby. Used for mod games. \
                                      Reregister, after uploading.")
                        .create_option(|o|
                            o.kind(ApplicationCommandOptionType::String)
                                .name("nation_text")
                                .description("Some text to represent your chosen nation")
                                .required(true)
                        )
                        .create_option(game_name_option)
                )
                .create_application_command(|c|
                    c.name("unregister")
                        .description("Unregister yourself from all nations in a game or lobby")
                        .create_option(game_name_option)
                )
                .create_application_command(|c|
                    c.name("turns")
                        .description("Have the bot PM you with the details of all games you're currently in.")
                )
                .create_application_command(|c|
                    c.name("notifications")
                        .description("Disable, or re-enable, the bot pinging you for turns.")
                        .create_option(|o|
                            o.kind(ApplicationCommandOptionType::Boolean)
                                .name("enabled")
                                .description("If true, then notifications are sent.")
                                .required(true)
                        )
                )
                .create_application_command(|c|
                    c.name("lobby")
                        .description("Create a lobby with no server for players to join")
                        .create_option(|o|
                            o.kind(ApplicationCommandOptionType::String)
                                .name("era")
                                .description("The era of the game (EA/MA/LA)")
                                .add_string_choice("EA", "EA")
                                .add_string_choice("MA", "MA")
                                .add_string_choice("LA", "LA")
                                .required(true)
                        )
                        .create_option(|o|
                            o.kind(ApplicationCommandOptionType::Integer)
                                .name("number_of_players")
                                .description("The number of players")
                                .required(true)
                        )
                        .create_option(game_name_option)
                )
                .create_application_command(|c|
                    c.name("lobbies")
                        .description("List available lobbies")
                )
                .create_application_command(|c|
                    c.name("start")
                        .description("Start a lobby with an address, after starting it on the server")
                        .create_option(|o|
                            o.kind(ApplicationCommandOptionType::String)
                                .name("address_and_port")
                                .description("The address and port, separated by a colon, of the started game")
                                .required(true)
                        )
                        .create_option(game_name_option)
                )
                .create_application_command(|c|
                    c.name("describe")
                        .description("Add a description to a lobby. Quotemarks required.")
                        .create_option(|o|
                            o.kind(ApplicationCommandOptionType::String)
                                .name("description")
                                .description("The description to add to the game. Quotemarks required.")
                                .required(true)
                        )
                        .create_option(game_name_option)
                )
                .create_application_command(|c|
                    c.name("unstart")
                        .description("Turn a game back into a lobby by forgetting its address.")
                        .create_option(game_name_option)
                )
        })
        .await
        .context("create_application_commands")?;
    Ok(())
}

fn game_name_option(
    create_option: &mut CreateApplicationCommandOption,
) -> &mut CreateApplicationCommandOption {
    create_option
        .kind(ApplicationCommandOptionType::String)
        .name("name")
        .description("The game's bot name. If not present, will use the channel name.")
}

pub async fn interaction_create(ctx: Context, interaction: Interaction) {
    if let Err(e) = interaction_create_result(ctx, interaction).await {
        error!("Failed to create interaction: {:#?}", e);
    }
}

async fn interaction_create_result(ctx: Context, interaction: Interaction) -> anyhow::Result<()> {
    info!("Incoming interaction: {:?}", interaction);

    if let Interaction::ApplicationCommand(interaction) = interaction {
        let channel_id = interaction.channel_id;
        let user_id = interaction
            .member
            .as_ref()
            .ok_or_else(|| anyhow!("No member in interaction_create"))?
            .user
            .id;

        let data = &interaction.data;

        let args = make_args(data);

        // This is quite cumbersome, it would be better to integrate with the serenity framework
        let command_response_result = match data.name.as_str() {
            "add" => add_server(&ctx, channel_id, user_id, args)
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
            other => Err(anyhow!("Unrecognised command: {}", other)),
        };
        match command_response_result {
            Ok(command_response) => {
                interaction
                    .create_interaction_response(ctx.http(), |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| match command_response {
                                CommandResponse::Reply(reply) => message.content(reply),
                                CommandResponse::Embed(embed) => message.add_embed(embed),
                            })
                    })
                    .await?;
            }
            Err(err) => {
                interaction
                    .create_interaction_response(ctx.http(), |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| {
                                let err = anyhow!(err);
                                let text = format!("ERROR: {}", err);
                                info!("command error: replying with '{}'", text);
                                message.content(text)
                            })
                    })
                    .await?
            }
        }
    }
    Ok(())
}

// okay this is VERY hacky. We're going via the delimited string for no reason at all.
// Unfortunately to fix it I'd have to do some big tokenize or pre-parse or whatever, and possibly
// even switching away from serenity's standard framework any I really can't be bothered.
fn make_args(data: &ApplicationCommandInteractionData) -> Args {
    let mut arg_string = String::new();
    for option in &data.options {
        if let Some(value) = option.value.as_ref() {
            arg_string.push_str(&format!("{} ", value));
        }
    }
    Args::new(&arg_string, &[Delimiter::Single(' ')])
}
