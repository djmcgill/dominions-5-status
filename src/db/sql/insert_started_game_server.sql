INSERT INTO game_servers (alias, started_server_id)
                    SELECT ?1, id
                    FROM started_servers
                    WHERE address = ?2