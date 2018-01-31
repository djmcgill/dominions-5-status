use serenity::model::UserId;

#[derive(Debug, Clone)]
pub struct Player {
    pub discord_user_id: UserId,
    pub turn_notifications: bool,
}
