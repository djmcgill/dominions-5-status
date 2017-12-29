command!(
    spell(_context, message, args) {
        println!{"spell message: {:?}", message};
        let name = args.single::<String>()?;
        let response = format!("https://larzm42.github.io/dom5inspector/?page=spell&spellq={}", name);
        println!("responding with {}", response);
        let _ = message.reply(&response); 
    }
);
