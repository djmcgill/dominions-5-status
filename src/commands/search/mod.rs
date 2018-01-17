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
            "https://larzm42.github.io/dom5inspector/?page={}&{}q={}&showmodcmds=1&showmoddinginfo=1&showids=1&loadEvents=1",
        category, category, search_term);
        info!("responding with {}", response);
        let _ = message.reply(&response); 
    };
    Ok(())
}

use serenity::framework::standard::StandardFramework;
pub trait WithSearchCommands: Sized {
    fn get_standard_framework(self) -> StandardFramework;
    fn with_search_commands(self) -> StandardFramework {
        self.get_standard_framework().command("item", |c| c
            .bucket("simple")
            .exec(|_, m, a| search(&"item", m, a))
        )
        .command("spell", |c| c
            .bucket("simple")
            .exec(|_, m, a| search(&"spell", m, a))
        )
        .command("unit", |c| c
            .bucket("simple")
            .exec(|_, m, a| search(&"unit", m, a))
        )
        .command("site", |c| c
            .bucket("simple")
            .exec(|_, m, a| search(&"site", m, a))
        )
        .command("merc", |c| c
            .bucket("simple")
            .exec(|_, m, a| search(&"merc", m, a))
        )
        .command("event", |c| c
            .bucket("simple")
            .exec(|_, m, a| search(&"event", m, a))
        )
    }
}
impl WithSearchCommands for StandardFramework {
    fn get_standard_framework(self) -> StandardFramework {
        self
    }
}
