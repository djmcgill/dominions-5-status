DELETE FROM started_servers
WHERE id NOT IN
    (SELECT started_server_id FROM game_servers WHERE started_server_id IS NOT NULL);
