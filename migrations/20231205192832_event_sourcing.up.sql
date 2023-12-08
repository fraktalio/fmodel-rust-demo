--      ########################
--      ######## TABLES ########
--      ########################

-- Registered deciders and the respectful events that these deciders can publish (decider can publish and/or source its own state from these event types only)
CREATE TABLE IF NOT EXISTS deciders
(
    -- decider name/type
    "decider" TEXT NOT NULL,
    -- event name/type that this decider can publish
    "event"   TEXT NOT NULL,
    PRIMARY KEY ("decider", "event")
);

-- Events
CREATE TABLE IF NOT EXISTS events
(
    -- event name/type. Part of a composite foreign key to `deciders`
    "event"       TEXT    NOT NULL,
    -- event ID. This value is used by the next event as it's `previous_id` value to guard against a Lost-EventModel problem / optimistic locking.
    "event_id"    UUID    NOT NULL UNIQUE,
    -- decider name/type. Part of a composite foreign key to `deciders`
    "decider"     TEXT    NOT NULL,
    -- business identifier for the decider
    "decider_id"  TEXT    NOT NULL,
    -- event data in JSON format
    "data"        JSONB   NOT NULL,
    -- command ID causing this event
    "command_id"  UUID    NULL,
    -- previous event uuid; null for first event; null does not trigger UNIQUE constraint; we defined a function `check_first_event_for_decider`
    "previous_id" UUID UNIQUE,
    -- indicator if the event stream for the `decider_id` is final
    "final"       BOOLEAN NOT NULL         DEFAULT FALSE,
    -- The timestamp of the event insertion. AUTOPOPULATES—DO NOT INSERT
    "created_at"  TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    -- ordering sequence/offset for all events in all deciders. AUTOPOPULATES—DO NOT INSERT
    "offset"      BIGSERIAL PRIMARY KEY,
    FOREIGN KEY ("decider", "event") REFERENCES deciders ("decider", "event")
);

CREATE INDEX IF NOT EXISTS decider_index ON events ("decider_id", "offset");

--      ########################
--      ##### SIDE EFFECTS #####
--      ########################

-- Many things that can be done using triggers can also be implemented using the Postgres rule system.
-- What currently cannot be implemented by rules are some kinds of constraints.
-- It is possible, to place a qualified rule that rewrites a query to NOTHING if the value of a column does not appear in another table.
-- But then the data is silently thrown away, and that's not a good idea.
-- If checks for valid values are required, and in the case of an invalid value an error message should be generated, it must be done by a trigger for now.

-- SIDE EFFECT (rule): immutable decider - ignore deleting already registered events
CREATE OR REPLACE RULE ignore_delete_decider_events AS ON DELETE TO deciders
    DO INSTEAD NOTHING;

-- SIDE EFFECT (rule): immutable decider - ignore updating already registered events
CREATE OR REPLACE RULE ignore_update_decider_events AS ON UPDATE TO deciders
    DO INSTEAD NOTHING;

-- SIDE EFFECT (rule): immutable events - ignore delete
CREATE OR REPLACE RULE ignore_delete_events AS ON DELETE TO events
    DO INSTEAD NOTHING;

-- SIDE EFFECT (rule): immutable events - ignore update
CREATE OR REPLACE RULE ignore_update_events AS ON UPDATE TO events
    DO INSTEAD NOTHING;

-- SIDE EFFECT (trigger): can only append events if the decider_id stream is not finalized already
CREATE OR REPLACE FUNCTION check_final_event_for_decider() RETURNS trigger AS
'
    BEGIN
        IF EXISTS(SELECT 1
                  FROM events
                  WHERE NEW.decider_id = decider_id
                    AND TRUE = final
                    AND NEW.decider = decider)
        THEN
            RAISE EXCEPTION ''last event for this decider stream is already final. the stream is closed, you can not append events to it.'';
        END IF;
        RETURN NEW;
    END;
'
    LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS t_check_final_event_for_decider ON events;
CREATE TRIGGER t_check_final_event_for_decider
    BEFORE INSERT
    ON events
    FOR EACH ROW
EXECUTE FUNCTION check_final_event_for_decider();


-- SIDE EFFECT (trigger): Can only use null previousId for first event in an decider
CREATE OR REPLACE FUNCTION check_first_event_for_decider() RETURNS trigger AS
'
    BEGIN
        IF (NEW.previous_id IS NULL
            AND EXISTS(SELECT 1
                       FROM events
                       WHERE NEW.decider_id = decider_id
                         AND NEW.decider = decider))
        THEN
            RAISE EXCEPTION ''previous_id can only be null for first decider event'';
        END IF;
        RETURN NEW;
    END;
'
    LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS t_check_first_event_for_decider ON events;
CREATE TRIGGER t_check_first_event_for_decider
    BEFORE INSERT
    ON events
    FOR EACH ROW
EXECUTE FUNCTION check_first_event_for_decider();


-- SIDE EFFECT (trigger): previousId must be in the same decider as the event
CREATE OR REPLACE FUNCTION check_previous_id_in_same_decider() RETURNS trigger AS
'
    BEGIN
        IF (NEW.previous_id IS NOT NULL
            AND NOT EXISTS(SELECT 1
                           FROM events
                           WHERE NEW.previous_id = event_id
                             AND NEW.decider_id = decider_id
                             AND NEW.decider = decider))
        THEN
            RAISE EXCEPTION ''previous_id must be in the same decider'';
        END IF;
        RETURN NEW;
    END;
'
    LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS t_check_previous_id_in_same_decider ON events;
CREATE TRIGGER t_check_previous_id_in_same_decider
    BEFORE INSERT
    ON events
    FOR EACH ROW
EXECUTE FUNCTION check_previous_id_in_same_decider();

