INSERT INTO game_servers (alias, dom_version, started_server_id)
SELECT ?1, ?2, id
FROM started_servers
WHERE address = ?3;
