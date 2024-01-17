use serenity::model::id::UserId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub discord_user_id: UserId,
    pub turn_notifications: bool,
}
