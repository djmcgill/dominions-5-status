UPDATE game_servers
SET started_server_id =
    (SELECT s.id
    from started_servers s
    where s.address = ?1 and s.last_seen_turn = ?2)
WHERE alias = ?3;
