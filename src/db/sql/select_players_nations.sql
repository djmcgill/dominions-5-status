SELECT p.discord_user_id, sp.nation_id, sp.custom_nation_name, p.turn_notifications
FROM game_servers s
JOIN server_players sp on sp.server_id = s.id
JOIN players p on p.id = sp.player_id
WHERE s.alias = ?1;
