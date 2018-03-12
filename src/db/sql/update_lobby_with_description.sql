UPDATE lobbies SET description = ?2
WHERE id = (SELECT lobby_id FROM game_servers WHERE alias = ?1);