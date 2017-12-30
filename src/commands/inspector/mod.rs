use url::percent_encoding::{utf8_percent_encode, QUERY_ENCODE_SET};

const VALID: &[&'static str] = &["item", "spell"]; // TODO: hashset?

command!(
    search(_context, message, args) {
        let category = args.single::<String>()?;
        let search_term = utf8_percent_encode(&args.full(), QUERY_ENCODE_SET).to_string();
        if category == VALID[0] || category == VALID[1] {
            let response = format!(
                "https://larzm42.github.io/dom5inspector/?page={}&{}q={}",
            category, category, search_term);
            println!("responding with {}", response);
            let _ = message.reply(&response); 
        };
    }
);
