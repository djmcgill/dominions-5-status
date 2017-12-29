command!(
    item(_context, message, args) {
        println!{"item message: {:?}", message};
        let name = args.single::<String>()?;
        let response = format!("https://larzm42.github.io/dom5inspector/?page=item&itemq={}", name);
        println!("responding with {}", response);
        let _ = message.reply(&response); 
    }
);
