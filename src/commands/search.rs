use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serenity::framework::standard::{macros::*, Args, CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;

#[group]
#[commands(
    search_item,
    search_spell,
    search_unit,
    search_site,
    search_merc,
    search_event
)]
struct Search;

#[command]
#[aliases("item")]
async fn search_item(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let response = search::<Item>(args)?;
    msg.reply((&ctx.cache, ctx.http.as_ref()), response).await?;
    Ok(())
}
#[command]
#[aliases("spell")]
async fn search_spell(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let response = search::<Spell>(args)?;
    msg.reply((&ctx.cache, ctx.http.as_ref()), response).await?;
    Ok(())
}
#[command]
#[aliases("unit")]
async fn search_unit(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let response = search::<Unit>(args)?;
    msg.reply((&ctx.cache, ctx.http.as_ref()), response).await?;
    Ok(())
}
#[command]
#[aliases("site")]
async fn search_site(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let response = search::<Site>(args)?;
    msg.reply((&ctx.cache, ctx.http.as_ref()), response).await?;
    Ok(())
}
#[command]
#[aliases("merc")]
async fn search_merc(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let response = search::<Merc>(args)?;
    msg.reply((&ctx.cache, ctx.http.as_ref()), response).await?;
    Ok(())
}
#[command]
#[aliases("event")]
async fn search_event(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let response = search::<Event>(args)?;
    msg.reply((&ctx.cache, ctx.http.as_ref()), response).await?;
    Ok(())
}

// TODO: just turn this into an enum already
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

fn search<I: InspectorCategory>(args: Args) -> Result<String, CommandError> {
    let search_term = utf8_percent_encode(args.rest(), NON_ALPHANUMERIC).to_string();
    Ok(format!(
        "https://larzm42.github.io/dom5inspector/\
        ?page={}&{}q={}&showmodcmds=1&showmoddinginfo=1&showids=1{}",
        I::show(),
        I::show(),
        search_term,
        I::event_append()
    ))
}
