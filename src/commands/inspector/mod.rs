use url::percent_encoding::{utf8_percent_encode, QUERY_ENCODE_SET};

use serenity::framework::standard::{Args, CommandError};
use serenity::model::Message;

const VALID: &[&'static str] = &["item", "spell", "unit", "site", "merc", "event"]; // TODO: hashset?

pub fn search(category: &str, message: &Message, args: Args) -> Result<(), CommandError> {
    let search_term = utf8_percent_encode(&args.full(), QUERY_ENCODE_SET).to_string();
    if category == VALID[0] || category == VALID[1] ||
        category == VALID[2] || category == VALID[3] ||
        category == VALID[4] || category == VALID[5] { // TODO: so gross
        let response = format!(
            "https://larzm42.github.io/dom5inspector/?page={}&{}q={}",
        category, category, search_term);
        info!("responding with {}", response);
        let _ = message.reply(&response); 
    };
    Ok(())
}
