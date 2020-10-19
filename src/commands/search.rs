use url::percent_encoding::{utf8_percent_encode, QUERY_ENCODE_SET};

use log::*;
use serenity::{CacheAndHttp, framework::standard::{Args, CommandError}};
use serenity::model::channel::Message;

// enum InspectorCategoryV {Item, Spell, Unit, ...}

// TODO: implement some kind of static enum macro or library
// see https://users.rust-lang.org/t/enum-field-types-datasort-refinements/11323
trait InspectorCategory: Copy {
    fn show() -> &'static str;
    fn event_append() -> &'static str;
    // fn reify() -> InspectorCategoryV;
}

#[derive(Clone, Copy)]
struct Item;
impl InspectorCategory for Item {
    fn show() -> &'static str {
        "item"
    }
    fn event_append() -> &'static str {
        ""
    }
}
#[derive(Clone, Copy)]
struct Spell;
impl InspectorCategory for Spell {
    fn show() -> &'static str {
        "spell"
    }
    fn event_append() -> &'static str {
        ""
    }
}
#[derive(Clone, Copy)]
struct Unit;
impl InspectorCategory for Unit {
    fn show() -> &'static str {
        "unit"
    }
    fn event_append() -> &'static str {
        ""
    }
}
#[derive(Clone, Copy)]
struct Site;
impl InspectorCategory for Site {
    fn show() -> &'static str {
        "site"
    }
    fn event_append() -> &'static str {
        ""
    }
}
#[derive(Clone, Copy)]
struct Merc;
impl InspectorCategory for Merc {
    fn show() -> &'static str {
        "merc"
    }
    fn event_append() -> &'static str {
        ""
    }
}
#[derive(Clone, Copy)]
struct Event;
impl InspectorCategory for Event {
    fn show() -> &'static str {
        "event"
    }
    fn event_append() -> &'static str {
        "&loadEvents=1"
    }
}

use serenity::framework::standard::macros::command;

#[command]
fn search<I: InspectorCategory>(message: &Message, args: &Args) -> Result<(), CommandError> {
    let search_term = utf8_percent_encode(&args.message(), QUERY_ENCODE_SET).to_string();
    let response = format!(
        "https://larzm42.github.io/dom5inspector/?page={}&{}q={}&showmodcmds=1&showmoddinginfo=1&showids=1{}",
    I::show(), I::show(), search_term, I::event_append());
    info!("responding with {}", response);
    let _ = message.reply(CacheAndHttp::default(), &response); // TODO: more research on wheter cache and http is fine for our use
    Ok(())
}

use serenity::framework::standard::StandardFramework;
pub trait WithSearchCommands: Sized {
    fn get_standard_framework(self) -> StandardFramework;
    fn with_search_commands(self, bucket: &str) -> StandardFramework {
        self.get_standard_framework()
            .command(Item::show(), |c| {
                c.bucket(bucket).exec(|_, m, a| search::<Item>(m, &a))
            })
            .command(Spell::show(), |c| {
                c.bucket(bucket).exec(|_, m, a| search::<Spell>(m, &a))
            })
            .command(Unit::show(), |c| {
                c.bucket(bucket).exec(|_, m, a| search::<Unit>(m, &a))
            })
            .command(Site::show(), |c| {
                c.bucket(bucket).exec(|_, m, a| search::<Site>(m, &a))
            })
            .command(Merc::show(), |c| {
                c.bucket(bucket).exec(|_, m, a| search::<Merc>(m, &a))
            })
            .command(Event::show(), |c| {
                c.bucket(bucket).exec(|_, m, a| search::<Event>(m, &a))
            })
    }
}
impl WithSearchCommands for StandardFramework {
    fn get_standard_framework(self) -> StandardFramework {
        self
    }
}
