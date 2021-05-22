use anyhow::{anyhow, Context as _};
use serenity::client::Context;
use serenity::http::CacheHttp;
use serenity::model::interactions::{Interaction, InteractionResponseType};
use serenity::{
    builder::CreateApplicationCommandOption,
    http::{GuildPagination, Http},
    model::{id::GuildId, prelude::ApplicationCommandOptionType},
};

// This technically only needs to be rerun once, but running every time on boot
// just overrides it each time and guild commands update instantly so who cares.
pub async fn create_guild_commands(http: &Http) -> anyhow::Result<()> {
    let guilds = http
        // The bot is only ever in one server per instance, so yolo
        .get_guilds(&GuildPagination::After(GuildId(0)), 1)
        .await?;
    let guild = guilds
        .get(0)
        .ok_or_else(|| anyhow!("Bot user is in 0 guilds"))?;

    let create_results = guild
        .id
        .create_application_commands(http, |cs| {
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
    println!("COMMANDS: {:?}", create_results);
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
        println!("AHHHHHHHH");
    }
}

async fn interaction_create_result(ctx: Context, interaction: Interaction) -> anyhow::Result<()> {
    let channel = interaction
        .channel_id
        .as_ref()
        .ok_or_else(|| anyhow!("No channel in interaction_create"))?;
    let data = interaction
        .data
        .as_ref()
        .ok_or_else(|| anyhow!("No data in interaction_create"))?;
    let command = &data.name;

    interaction
        .create_interaction_response(ctx.http(), |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                // FIXME
                .interaction_response_data(|message| message.set_embed(unimplemented!()))
        })
        .await?;
    Ok(())
}
