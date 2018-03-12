DELETE FROM lobbies
WHERE id NOT IN (select lobby_id from game_servers);
