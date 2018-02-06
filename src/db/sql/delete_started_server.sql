DELETE FROM started_servers
            WHERE id NOT IN (select started_server_id from game_servers)