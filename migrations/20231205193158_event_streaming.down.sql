DROP TRIGGER IF EXISTS "t_before_update_views_table" ON "views";
DROP TRIGGER IF EXISTS "t_before_update_locks_table" ON "locks";
DROP TRIGGER IF EXISTS t_on_insert_on_events ON events;
DROP TRIGGER IF EXISTS t_on_insert_or_update_on_views ON "views";
DROP FUNCTION IF EXISTS before_update_views_table();
DROP FUNCTION IF EXISTS before_update_locks_table();
DROP FUNCTION IF EXISTS on_insert_on_events();
DROP FUNCTION IF EXISTS on_insert_or_update_on_views();
DROP FUNCTION IF EXISTS stream_events();
DROP TABLE IF EXISTS views CASCADE;
DROP TABLE IF EXISTS locks CASCADE;


