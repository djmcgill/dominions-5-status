use ::server::get_game_data;

command!(game_name(_context, message, args) {
    println!{"game_name message: {:?}", message};
    let server_address = args.single::<String>().unwrap();
    let response = get_game_data(&server_address).unwrap().game_name;
    let _ = message.reply(&format!("Game name at {} is {}", server_address, response));
});
