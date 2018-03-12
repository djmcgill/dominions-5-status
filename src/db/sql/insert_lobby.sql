INSERT INTO lobbies (owner_id, era, player_count, description)
SELECT id, ?1, ?3, ?4
FROM players
WHERE discord_user_id = ?2;
