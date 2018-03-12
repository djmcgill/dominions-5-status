use serenity::model::channel::Message;
use serenity::framework::standard::CommandError;

pub fn help(message: &Message) -> Result<(), CommandError> {
    debug!("HELP COMMAND");
    let _ = message.reply(
        "Commands (server alias is optional, defaults to channel name): \n\
        - !add <address:port> <alias>: save the dom5 server address\n\
        - !list: return a list of the saved server addresses and aliases\n\
        - !delete <alias>: remove the server address from the list\n\
        - !details <alias>: return a list of the nations and their statuses in the game\n\
        - !register nation_prefix <alias>: register yourself as a nation in a game\n\
        - !unregister <alias>: unregister yourself in a game\n\
        - !turns: show all of the games you're in and their turn status\n\
        - !notifications {true, false}: enable/disable turn notifications\n\
        - !lobby {EA/MA/LA} <num_players> <alias>: create a lobby with no server\n\
        - !lobbies: list available lobbies\n\
        - !start <address:port> <alias>: register a started server for a lobby game\n\
        - !{item, spell, unit, site, merc, event} <text>: get dom5inspector search url\n\
        - !help: display this text\n\
        - !describe \"text\" <alias>: add a description to a lobby. Quotes required."
    )?;
    Ok(())
}
