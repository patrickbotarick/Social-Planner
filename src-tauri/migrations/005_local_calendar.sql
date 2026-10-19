CREATE TABLE schedules (
    id TEXT PRIMARY KEY NOT NULL,
    publication_id TEXT NOT NULL REFERENCES publications(id) ON DELETE RESTRICT,
    publication_version INTEGER NOT NULL CHECK (publication_version > 0),
    scheduled_date TEXT NOT NULL CHECK (length(scheduled_date) = 10),
    scheduled_time TEXT NOT NULL CHECK (length(scheduled_time) = 5),
    timezone TEXT NOT NULL CHECK (length(timezone) BETWEEN 1 AND 100),
    scheduled_at_utc TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('planned_local','cancelled_local','invalidated_local')),
    version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    cancelled_at TEXT,
    CHECK ((status = 'cancelled_local') = (cancelled_at IS NOT NULL))
);
CREATE UNIQUE INDEX schedules_one_active ON schedules(publication_id) WHERE status = 'planned_local';
CREATE INDEX schedules_instant ON schedules(scheduled_at_utc, status);
CREATE TABLE schedule_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    proximity_minutes INTEGER NOT NULL DEFAULT 30 CHECK (proximity_minutes BETWEEN 0 AND 180),
    time_step_minutes INTEGER NOT NULL DEFAULT 30 CHECK (time_step_minutes IN (5,10,15,30,60)),
    version INTEGER NOT NULL DEFAULT 1 CHECK (version > 0)
);
INSERT INTO schedule_settings(id) VALUES(1);
CREATE TABLE schedule_events (
    id INTEGER PRIMARY KEY,
    schedule_id TEXT NOT NULL REFERENCES schedules(id) ON DELETE RESTRICT,
    event TEXT NOT NULL CHECK (event IN ('created_local','rescheduled_local','cancelled_local','invalidated_local')),
    previous_date TEXT, previous_time TEXT, previous_timezone TEXT, previous_utc TEXT,
    scheduled_date TEXT NOT NULL, scheduled_time TEXT NOT NULL, timezone TEXT NOT NULL, scheduled_at_utc TEXT NOT NULL,
    schedule_version INTEGER NOT NULL,
    publication_version INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
CREATE INDEX schedule_events_recent ON schedule_events(created_at DESC, id DESC);
CREATE TRIGGER schedule_events_no_update BEFORE UPDATE ON schedule_events BEGIN
    SELECT RAISE(ABORT,'Local planning history is immutable');
END;
CREATE TRIGGER schedule_events_no_delete BEFORE DELETE ON schedule_events BEGIN
    SELECT RAISE(ABORT,'Local planning history is immutable');
END;
CREATE TRIGGER schedules_no_delete BEFORE DELETE ON schedules BEGIN
    SELECT RAISE(ABORT,'Cancel local planning instead of deleting');
END;
CREATE TRIGGER schedules_validate_insert BEFORE INSERT ON schedules WHEN NEW.status != 'planned_local' OR NOT EXISTS (
    SELECT 1 FROM publications WHERE id=NEW.publication_id AND version=NEW.publication_version AND editorial_status='approved' AND deleted_at IS NULL
) BEGIN SELECT RAISE(ABORT,'Approved publication version required'); END;
CREATE TRIGGER schedules_validate_update BEFORE UPDATE ON schedules WHEN
    OLD.status != 'planned_local' OR NEW.version != OLD.version+1 OR NEW.publication_id != OLD.publication_id OR NEW.publication_version != OLD.publication_version OR
    (NEW.status='planned_local' AND NOT EXISTS(SELECT 1 FROM publications WHERE id=NEW.publication_id AND version=NEW.publication_version AND editorial_status='approved' AND deleted_at IS NULL))
BEGIN SELECT RAISE(ABORT,'Invalid local planning transition'); END;
CREATE TRIGGER schedules_created AFTER INSERT ON schedules BEGIN
    INSERT INTO schedule_events(schedule_id,event,scheduled_date,scheduled_time,timezone,scheduled_at_utc,schedule_version,publication_version)
    VALUES(NEW.id,'created_local',NEW.scheduled_date,NEW.scheduled_time,NEW.timezone,NEW.scheduled_at_utc,NEW.version,NEW.publication_version);
END;
CREATE TRIGGER schedules_changed AFTER UPDATE ON schedules BEGIN
    INSERT INTO schedule_events(schedule_id,event,previous_date,previous_time,previous_timezone,previous_utc,scheduled_date,scheduled_time,timezone,scheduled_at_utc,schedule_version,publication_version)
    VALUES(NEW.id,CASE WHEN NEW.status='planned_local' THEN 'rescheduled_local' ELSE NEW.status END,OLD.scheduled_date,OLD.scheduled_time,OLD.timezone,OLD.scheduled_at_utc,NEW.scheduled_date,NEW.scheduled_time,NEW.timezone,NEW.scheduled_at_utc,NEW.version,NEW.publication_version);
END;
CREATE TRIGGER schedules_editorial_change AFTER UPDATE OF version,editorial_status,deleted_at ON publications WHEN NEW.version != OLD.version OR NEW.editorial_status != 'approved' OR NEW.deleted_at IS NOT NULL BEGIN
    UPDATE schedules SET status='invalidated_local',version=version+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE publication_id=NEW.id AND status='planned_local';
END;
