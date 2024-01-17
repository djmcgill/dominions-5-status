INSERT INTO game_servers (alias, dom_version, lobby_id)
    SELECT ?1, ?2, l.id
    FROM lobbies l
    WHERE l.era = ?3
    AND l.id = last_insert_rowid()
    AND l.owner_id = (SELECT id FROM players where discord_user_id = ?4)
    AND l.player_count = ?5
