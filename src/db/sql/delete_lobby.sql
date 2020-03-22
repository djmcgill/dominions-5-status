DELETE FROM lobbies
WHERE id NOT IN (SELECT lobby_id FROM game_servers WHERE lobby_id IS NOT NULL);
