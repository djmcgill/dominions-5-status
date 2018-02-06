DELETE FROM server_players
            WHERE server_id IN
            (SELECT id from game_servers WHERE alias = ?1)