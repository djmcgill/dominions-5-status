use crate::model::nation::Nation;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameData {
    pub game_name: String,
    pub nations: Vec<Nation>,
    pub turn: i32,
    pub turn_deadline: DateTime<Utc>,
}
