 SELECT s.address, g.alias, s.last_seen_turn, sp.nation_id, l.owner_id, l.era, l.player_count
FROM players p
JOIN server_players sp on sp.player_id = p.id
JOIN game_servers g on g.id = sp.server_id
LEFT JOIN lobbies l on l.id = g.lobby_id
LEFT JOIN started_servers s on s.id = g.started_server_id
WHERE p.discord_user_id = ?1;
