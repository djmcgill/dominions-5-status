INSERT INTO lobbies (owner_id, era, player_count)
                    SELECT id, ?1, ?3
                    FROM players
                    WHERE discord_user_id = ?2