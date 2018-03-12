INSERT INTO server_players (server_id, player_id, nation_id)
SELECT g.id, p.id, ?1
FROM game_servers g
JOIN players p ON p.discord_user_id = ?2
WHERE g.alias = ?3;
