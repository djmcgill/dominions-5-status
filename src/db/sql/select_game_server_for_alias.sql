SELECT s.address, s.last_seen_turn, p.discord_user_id, l.era, l.player_count, l.description
FROM game_servers g
LEFT JOIN started_servers s ON s.id = g.started_server_id
LEFT JOIN lobbies l ON l.id = g.lobby_id
LEFT JOIN players p ON l.owner_id = p.id
WHERE g.alias = ?1;
