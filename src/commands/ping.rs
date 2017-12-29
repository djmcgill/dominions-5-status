command!(ping(_context, message) {
    println!{"ping message: {:?}", message};
    let _ = message.reply("Pong!");
});
