UPDATE started_servers
SET last_seen_turn = ?1
WHERE id = (select started_server_id from game_servers where alias = ?2)
AND last_seen_turn < ?1;
