 alter table game_servers add column dom_version integer;
 update game_servers set dom_version = 5;
