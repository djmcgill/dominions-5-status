use url::percent_encoding::{utf8_percent_encode, QUERY_ENCODE_SET};

const VALID: &[&'static str] = &["item", "spell", "unit", "site", "merc", "event"]; // TODO: hashset?

command!(
    search(_context, message, args) {
        let category = args.single::<String>()?;
        let search_term = utf8_percent_encode(&args.full(), QUERY_ENCODE_SET).to_string();
        if category == VALID[0] || category == VALID[1] ||
            category == VALID[2] || category == VALID[3] ||
            category == VALID[4] || category == VALID[5] { // TODO: so gross
            let response = format!(
                "https://larzm42.github.io/dom5inspector/?page={}&{}q={}",
            category, category, search_term);
            println!("responding with {}", response);
            let _ = message.reply(&response); 
        };
    }
);
