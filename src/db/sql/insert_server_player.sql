INSERT INTO server_players (server_id, player_id, nation_id, custom_nation_name)
SELECT g.id, p.id, ?1, ?2
FROM game_servers g
JOIN players p ON p.discord_user_id = ?3
WHERE g.alias = ?4;
