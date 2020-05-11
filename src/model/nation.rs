use crate::model::enums::{NationStatus, Nations, StaticNation, SubmissionStatus};
use crate::snek::SnekGameStatus;
use std::borrow::Cow;

/// We always get an ID when talking to the game
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameNationIdentifier {
    Existing(StaticNation),
    CustomId(u32),
}
impl GameNationIdentifier {
    pub fn id(&self) -> u32 {
        match *self {
            GameNationIdentifier::Existing(nation) => nation.id,
            GameNationIdentifier::CustomId(nation_id) => nation_id,
        }
    }
    pub fn name(&self, option_snek_state: Option<&SnekGameStatus>) -> Cow<'static, str> {
        match *self {
            GameNationIdentifier::Existing(nation) => existing_name(&nation).into(),
            GameNationIdentifier::CustomId(nation_id) => {
                custom_id_name(nation_id, option_snek_state).into()
            }
        }
    }
    pub fn from_id(id: u32) -> Self {
        if let Some(static_nation) = Nations::from_id(id) {
            GameNationIdentifier::Existing(static_nation)
        } else {
            GameNationIdentifier::CustomId(id)
        }
    }
}
impl From<GameNationIdentifier> for BotNationIdentifier {
    fn from(other: GameNationIdentifier) -> Self {
        match other {
            GameNationIdentifier::CustomId(custom_id) => BotNationIdentifier::CustomId(custom_id),
            GameNationIdentifier::Existing(static_nation) => {
                BotNationIdentifier::Existing(static_nation)
            }
        }
    }
}

/// Players may sign up with a known ID, an unknown ID, or just some rando string
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BotNationIdentifier {
    Existing(StaticNation),
    CustomName(String),
    CustomId(u32),
}
impl BotNationIdentifier {
    pub fn from_id(id: u32) -> Self {
        GameNationIdentifier::from_id(id).into()
    }
    pub fn from_name(name: String) -> Self {
        BotNationIdentifier::CustomName(name)
    }
    pub fn from_id_and_name(option_id: Option<u32>, option_name: Option<String>) -> Option<Self> {
        match (option_id, option_name) {
            (Some(id), None) => Some(BotNationIdentifier::from_id(id)),
            (None, Some(name)) => Some(BotNationIdentifier::from_name(name)),
            _ => None,
        }
    }

    pub fn id(&self) -> Option<u32> {
        match *self {
            BotNationIdentifier::Existing(StaticNation { id, .. }) => Some(id),
            BotNationIdentifier::CustomName(_) => None,
            BotNationIdentifier::CustomId(id) => Some(id),
        }
    }

    // TODO: also give static nations static nation strings
    pub fn name(&self, option_snek_state: Option<&SnekGameStatus>) -> Cow<'static, str> {
        match self {
            BotNationIdentifier::CustomName(name) => name.clone().into(),
            BotNationIdentifier::Existing(nation) => existing_name(&nation).into(),
            BotNationIdentifier::CustomId(nation_id) => {
                custom_id_name(*nation_id, option_snek_state).into()
            }
        }
    }
}

fn custom_id_name(nation_id: u32, option_snek_state: Option<&SnekGameStatus>) -> String {
    let snek_nation_details =
        option_snek_state.and_then(|snek_details| snek_details.nations.get(&nation_id));
    match snek_nation_details {
        Some(snek_nation) => format!("{} ({})", snek_nation.name, nation_id).into(),
        None => format!("Unknown ({})", nation_id).into(),
    }
}

fn existing_name(nation: &StaticNation) -> String {
    format!("{} ({})", nation.name, nation.id)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nation {
    pub identifier: GameNationIdentifier,
    pub status: NationStatus,
    pub submitted: SubmissionStatus,
    pub connected: bool,
}
