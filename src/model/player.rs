use serenity::model::id::UserId;

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub discord_user_id: UserId,
    pub turn_notifications: bool,
}
