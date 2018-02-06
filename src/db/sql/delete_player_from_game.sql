DELETE FROM server_players
            WHERE server_id IN
            (SELECT id from game_servers WHERE alias = ?1)
            AND player_id IN
            (SELECT id from players WHERE discord_user_id = ?2)
