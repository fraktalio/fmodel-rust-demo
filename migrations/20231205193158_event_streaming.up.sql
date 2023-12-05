CREATE TABLE IF NOT EXISTS views
(
    -- view identifier/name
    "view"          TEXT,
    -- pooling_delay represent the frequency of pooling the database for the new events / 500 ms by default
    "pooling_delay" BIGINT                   DEFAULT 500   NOT NULL,
    -- the point in time form where the event streaming/pooling should start / NOW is by default, but you can specify the binging of time if you want
    "start_at"      TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    -- the timestamp of the view insertion.
    "created_at"    TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    -- the timestamp of the view update.
    "updated_at"    TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    PRIMARY KEY ("view")
);

CREATE TABLE IF NOT EXISTS locks
(
    -- view identifier/name
    "view"         TEXT                                                    NOT NULL,
    -- business identifier for the decider
    "decider_id"   TEXT                                                    NOT NULL,
    -- current offset of the event stream for decider_id
    "offset"       BIGINT                                                  NOT NULL,
    -- the offset of the last event being processed
    "last_offset"  BIGINT                                                  NOT NULL,
    -- a lock / is this event stream for particular decider_id locked for reading or not
    "locked_until" TIMESTAMP WITH TIME ZONE DEFAULT NOW() - INTERVAL '1ms' NOT NULL,
    -- an indicator if the offset is final / offset will not grow any more
    "offset_final" BOOLEAN                                                 NOT NULL,
    -- the timestamp of the view insertion.
    "created_at"   TIMESTAMP WITH TIME ZONE DEFAULT NOW()                  NOT NULL,
    -- the timestamp of the view update.
    "updated_at"   TIMESTAMP WITH TIME ZONE DEFAULT NOW()                  NOT NULL,
    PRIMARY KEY ("view", "decider_id"),
    FOREIGN KEY ("view") REFERENCES views ("view") ON DELETE CASCADE
);

-- SIDE EFFECT:  before_update_views_table - automatically bump "updated_at" when modifying a view
CREATE OR REPLACE FUNCTION "before_update_views_table"() RETURNS trigger AS
'
    BEGIN
        NEW.updated_at = NOW();
        RETURN NEW;
    END;
'
    LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS "t_before_update_views_table" ON "views";
CREATE TRIGGER "t_before_update_views_table"
    BEFORE UPDATE
    ON "views"
    FOR EACH ROW
EXECUTE FUNCTION "before_update_views_table"();

-- SIDE EFFECT:  before_update_locks_table - automatically bump "updated_at" when modifying a lock
CREATE OR REPLACE FUNCTION "before_update_locks_table"() RETURNS trigger AS
'
    BEGIN
        NEW.updated_at = NOW();
        RETURN NEW;
    END;
'
    LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS "t_before_update_locks_table" ON "locks";
CREATE TRIGGER "t_before_update_locks_table"
    BEFORE UPDATE
    ON "locks"
    FOR EACH ROW
EXECUTE FUNCTION "before_update_locks_table"();

--  SIDE EFFECT: after appending a new event (with new decider_id), the lock is upserted
CREATE OR REPLACE FUNCTION on_insert_on_events() RETURNS trigger AS
'
    BEGIN

        INSERT INTO locks
        SELECT t1.view        AS view,
               NEW.decider_id AS decider_id,
               NEW.offset     AS offset,
               0              AS last_offset,
               NOW()          AS locked_until,
               NEW.final      AS offset_final
        FROM views AS t1
        ON CONFLICT ON CONSTRAINT "locks_pkey" DO UPDATE SET "offset"     = NEW."offset",
                                                             offset_final = NEW.final;
        RETURN NEW;
    END;
'
    LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS t_on_insert_on_events ON events;
CREATE TRIGGER t_on_insert_on_events
    AFTER INSERT
    ON events
    FOR EACH ROW
EXECUTE FUNCTION on_insert_on_events();



-- SIDE EFFECT: after upserting a views, all the locks should be re-upserted so to keep the correct matrix of `view-deciderId` locks
CREATE OR REPLACE FUNCTION on_insert_or_update_on_views() RETURNS trigger AS
'
    BEGIN
        INSERT INTO locks
        SELECT NEW."view"    AS "view",
               t1.decider_id AS decider_id,
               t1.max_offset AS "offset",
               COALESCE(
                       (SELECT t2."offset" - 1 AS "offset"
                        FROM events AS t2
                        WHERE t2.decider_id = t1.decider_id
                          AND t2.created_at >= NEW.start_at
                        ORDER BY t2."offset" ASC
                        LIMIT 1),
                       (SELECT t2."offset" AS "offset"
                        FROM events AS t2
                        WHERE t2.decider_id = t1.decider_id
                        ORDER BY "t2"."offset" DESC
                        LIMIT 1)
               )             AS last_offset,
               NOW()         AS locked_until,
               t1.final      AS offset_final
        FROM (SELECT DISTINCT ON (decider_id) decider_id AS decider_id,
                                              "offset"   AS max_offset,
                                              final      AS final
              FROM events
              ORDER BY decider_id, "offset" DESC) AS t1
        ON CONFLICT ON CONSTRAINT "locks_pkey"
            DO UPDATE
            SET "offset"     = EXCLUDED."offset",
                last_offset  = EXCLUDED.last_offset,
                offset_final = EXCLUDED.offset_final;
        RETURN NEW;
    END;
' LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS t_on_insert_or_update_on_views ON "views";
CREATE TRIGGER t_on_insert_or_update_on_views
    AFTER INSERT OR UPDATE
    ON "views"
    FOR EACH ROW
EXECUTE FUNCTION on_insert_or_update_on_views();



-- ##### API for streaming events ######
CREATE OR REPLACE FUNCTION stream_events(v_view_name TEXT)
    RETURNS SETOF events AS
'
    DECLARE
        v_last_offset INTEGER;
        v_decider_id  TEXT;
    BEGIN
        -- Check if there are events with a greater id than the last_offset and acquire lock on views table/row for the first decider_id/stream you can find
        SELECT decider_id,
               last_offset
        INTO v_decider_id, v_last_offset
        FROM locks
        WHERE view = v_view_name
          AND locked_until < NOW() -- locked = false
          AND last_offset < "offset"
        ORDER BY "offset"
        LIMIT 1 FOR UPDATE SKIP LOCKED;

        -- Update views locked status to true
        UPDATE locks
        SET locked_until = NOW() + INTERVAL ''5m'' -- locked = true, for next 5 minutes
        WHERE view = v_view_name
          AND locked_until < NOW() -- locked = false
          AND decider_id = v_decider_id;

        -- Return the events that have not been locked yet
        RETURN QUERY SELECT *
                     FROM events
                     WHERE decider_id = v_decider_id
                       AND "offset" > v_last_offset
                     ORDER BY "offset"
                     LIMIT 1;
    END;
' LANGUAGE plpgsql;
