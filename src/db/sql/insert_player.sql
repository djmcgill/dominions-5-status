INSERT INTO players (discord_user_id, turn_notifications)
            SELECT ?1, ?2
            WHERE NOT EXISTS (select 1 from players where discord_user_id = ?1)