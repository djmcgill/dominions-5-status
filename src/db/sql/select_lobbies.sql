SELECT g.alias, l.owner_id, l.era, l.player_count, count(sp.player_id)
FROM game_servers g
LEFT JOIN lobbies l ON l.id = g.lobby_id
JOIN server_players sp on sp.server_id = g.id
WHERE g.started_server_id IS NULL
GROUP BY sp.server_id