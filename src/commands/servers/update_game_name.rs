pub fn update_game_name(
    context: &mut Context,
    message: &Message,
    mut args: Args,
) -> Result<(), CommandError> {
    let data = context.data.lock();
    let db_conn = data
        .get::<DbConnectionKey>()
        .ok_or("No DbConnection was created on startup. This is a bug.")?;
    let new_name = args.single_quoted::<String>()?;
    let alias = alias_from_arg_or_channel_name(&mut args, &message)?;
    if !args.is_empty() {
        return Err(CommandError::from(
            "Too many arguments. TIP: spaces in arguments need to be quoted \"like this\"",
        ));
    }
    update_game_name_helper(db_conn, &address, &alias)?;
    message.reply(&"renamed {} to {}!", alias, new_name)?;
    Ok(())
}

fn update_game_name_helper(
    db_conn: &DbConnection,
    address: &str,
    alias: &str,
) -> Result<(), CommandError> {
    unimplemented!();
    Ok(())
}