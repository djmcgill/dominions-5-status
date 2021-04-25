INSERT INTO game_servers (alias, lobby_id)
    SELECT ?1, l.id
    FROM lobbies l
    WHERE l.era = ?2
    AND l.id = last_insert_rowid()
    AND l.owner_id = (SELECT id FROM players where discord_user_id = ?3)
    AND l.player_count = ?4
