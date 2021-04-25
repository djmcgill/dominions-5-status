UPDATE game_servers
SET started_server_id = NULL
WHERE alias = ?1 AND lobby_id IS NOT NULL;
