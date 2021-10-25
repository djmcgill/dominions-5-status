UPDATE lobbies SET alias = ?2
WHERE id = (SELECT lobby_id FROM game_servers WHERE alias = ?1);
