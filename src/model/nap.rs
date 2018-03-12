use model::enums::NapType;
use serenity::model::id::UserId;

#[derive(PartialEq, Debug, Clone)]
pub struct Nap {
    pub nap_type: NapType,
    pub players: Vec<UserId>,
    pub game_alias: String,
}
