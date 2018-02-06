INSERT INTO game_servers (alias, lobby_id)
    SELECT ?1, l.id
    FROM lobbies l
    LEFT JOIN game_servers s ON s.lobby_id = l.id
    WHERE l.era = ?2
    AND l.owner_id = (SELECT id FROM players where discord_user_id = ?3)
    AND l.player_count = ?4
    AND s.id IS NULL;
