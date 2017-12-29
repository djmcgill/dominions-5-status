use ::server::get_game_data;
use ::nations;

command!(nation_status(_context, message, args) {
    println!{"nation_status message: {:?}", message};
    let server_address = args.single::<String>().unwrap();
    let data = get_game_data(&server_address).unwrap();
    let mut response = String::new();
    for i in 0..250 {
        let status_num = data.f[i];        
        if status_num != 0 && status_num != 3 {
            let submitted = data.f[i+250];
            let connected = data.f[i+500];
            let nation_name = nations::get_nation_desc(i-1); // why -1? No fucking idea
            response.push_str(&format!(
                "name: {}, status: {}, submitted: {}, connected: {}\n", nation_name, status_num, submitted, connected
            ))
        }
    }
    println!("responding with {}", response);
    let _ = message.reply(&response);    
});
