CREATE TABLE export_roots (
 id TEXT PRIMARY KEY NOT NULL, path TEXT NOT NULL UNIQUE,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
CREATE TABLE assisted_settings (id INTEGER PRIMARY KEY CHECK(id=1), root_id TEXT REFERENCES export_roots(id) ON DELETE RESTRICT);
INSERT INTO assisted_settings(id) VALUES(1);
CREATE TABLE preparations (
 id TEXT PRIMARY KEY NOT NULL,
 publication_id TEXT NOT NULL REFERENCES publications(id) ON DELETE RESTRICT,
 schedule_id TEXT NOT NULL REFERENCES schedules(id) ON DELETE RESTRICT,
 publication_version INTEGER NOT NULL CHECK(publication_version>0),
 schedule_version INTEGER NOT NULL CHECK(schedule_version>0),
 fingerprint TEXT NOT NULL CHECK(length(fingerprint)=64),
 status TEXT NOT NULL CHECK(status IN ('preparing','prepared','outdated','preparation_failed')),
 root_id TEXT NOT NULL REFERENCES export_roots(id) ON DELETE RESTRICT,
 relative_path TEXT NOT NULL,
 regenerated_from TEXT REFERENCES preparations(id) ON DELETE RESTRICT,
 error_code TEXT,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 UNIQUE(root_id,relative_path)
);
CREATE INDEX preparations_schedule ON preparations(schedule_id,created_at DESC,id DESC);
CREATE UNIQUE INDEX preparations_one_current ON preparations(schedule_id) WHERE status IN ('preparing','prepared');
CREATE TABLE assisted_confirmations (
 id TEXT PRIMARY KEY NOT NULL,
 publication_id TEXT NOT NULL REFERENCES publications(id) ON DELETE RESTRICT,
 schedule_id TEXT NOT NULL REFERENCES schedules(id) ON DELETE RESTRICT,
 preparation_id TEXT NOT NULL REFERENCES preparations(id) ON DELETE RESTRICT,
 publication_version INTEGER NOT NULL CHECK(publication_version>0),
 schedule_version INTEGER NOT NULL CHECK(schedule_version>0),
 fingerprint TEXT NOT NULL CHECK(length(fingerprint)=64),
 planned_date TEXT NOT NULL, planned_time TEXT NOT NULL, planned_timezone TEXT NOT NULL, planned_utc TEXT NOT NULL,
 actual_date TEXT NOT NULL, actual_time TEXT NOT NULL, actual_timezone TEXT NOT NULL, actual_utc TEXT NOT NULL,
 platform TEXT NOT NULL CHECK(platform='instagram'),
 format TEXT NOT NULL CHECK(format IN ('feed','carousel','reels','stories')),
 reference TEXT NOT NULL DEFAULT '', note TEXT NOT NULL DEFAULT '',
 status TEXT NOT NULL CHECK(status IN ('confirmed_manually','requires_review','superseded')),
 version INTEGER NOT NULL DEFAULT 1 CHECK(version>0),
 confirmed_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
CREATE UNIQUE INDEX assisted_one_confirmation ON assisted_confirmations(schedule_id) WHERE status!='superseded';
CREATE UNIQUE INDEX assisted_one_publication_confirmation ON assisted_confirmations(publication_id) WHERE status!='superseded';
CREATE TABLE assisted_events (
 id INTEGER PRIMARY KEY,
 publication_id TEXT NOT NULL REFERENCES publications(id) ON DELETE RESTRICT,
 schedule_id TEXT NOT NULL REFERENCES schedules(id) ON DELETE RESTRICT,
 preparation_id TEXT REFERENCES preparations(id) ON DELETE RESTRICT,
 confirmation_id TEXT REFERENCES assisted_confirmations(id) ON DELETE RESTRICT,
 event TEXT NOT NULL CHECK(event IN ('preparation_started','preparation_completed','preparation_failed','package_regenerated','package_outdated','confirmation_recorded','confirmation_revised','divergence_detected')),
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
CREATE INDEX assisted_events_recent ON assisted_events(created_at DESC,id DESC);
CREATE TRIGGER assisted_events_no_update BEFORE UPDATE ON assisted_events BEGIN SELECT RAISE(ABORT,'Assisted history is immutable'); END;
CREATE TRIGGER assisted_events_no_delete BEFORE DELETE ON assisted_events BEGIN SELECT RAISE(ABORT,'Assisted history is immutable'); END;
CREATE TRIGGER preparations_no_delete BEFORE DELETE ON preparations BEGIN SELECT RAISE(ABORT,'Preparation history must be preserved'); END;
CREATE TRIGGER preparations_immutable BEFORE UPDATE ON preparations WHEN OLD.status IN ('outdated','preparation_failed') OR NEW.id!=OLD.id OR NEW.publication_id!=OLD.publication_id OR NEW.schedule_id!=OLD.schedule_id OR NEW.publication_version!=OLD.publication_version OR NEW.schedule_version!=OLD.schedule_version OR NEW.fingerprint!=OLD.fingerprint OR NEW.root_id!=OLD.root_id OR NEW.relative_path!=OLD.relative_path OR NEW.regenerated_from IS NOT OLD.regenerated_from OR NEW.created_at!=OLD.created_at OR (OLD.status='prepared' AND NEW.status!='outdated') BEGIN SELECT RAISE(ABORT,'Preserve preparation snapshot'); END;
CREATE TRIGGER assisted_confirmations_no_delete BEFORE DELETE ON assisted_confirmations BEGIN SELECT RAISE(ABORT,'Confirmation history must be preserved'); END;
CREATE TRIGGER preparations_validate BEFORE INSERT ON preparations WHEN NEW.status!='preparing' OR NOT EXISTS(
 SELECT 1 FROM schedules s JOIN publications p ON p.id=s.publication_id WHERE s.id=NEW.schedule_id AND p.id=NEW.publication_id AND s.status='planned_local' AND p.deleted_at IS NULL AND p.editorial_status='approved' AND p.version=NEW.publication_version AND s.publication_version=p.version AND s.version=NEW.schedule_version
) BEGIN SELECT RAISE(ABORT,'Current approved plan required'); END;
CREATE TRIGGER preparations_created AFTER INSERT ON preparations BEGIN
 INSERT INTO assisted_events(publication_id,schedule_id,preparation_id,event) VALUES(NEW.publication_id,NEW.schedule_id,NEW.id,'preparation_started');
END;
CREATE TRIGGER preparations_finish_validate BEFORE UPDATE OF status ON preparations WHEN NEW.status='prepared' AND NOT EXISTS(
 SELECT 1 FROM schedules s JOIN publications p ON p.id=s.publication_id WHERE s.id=NEW.schedule_id AND p.id=NEW.publication_id AND s.status='planned_local' AND p.deleted_at IS NULL AND p.editorial_status='approved' AND p.version=NEW.publication_version AND s.publication_version=p.version AND s.version=NEW.schedule_version
) BEGIN SELECT RAISE(ABORT,'Current approved plan required'); END;
CREATE TRIGGER preparations_changed AFTER UPDATE OF status ON preparations WHEN NEW.status!=OLD.status BEGIN
 INSERT INTO assisted_events(publication_id,schedule_id,preparation_id,event) VALUES(NEW.publication_id,NEW.schedule_id,NEW.id,CASE NEW.status WHEN 'prepared' THEN 'preparation_completed' WHEN 'outdated' THEN 'package_outdated' ELSE 'preparation_failed' END);
 INSERT INTO assisted_events(publication_id,schedule_id,preparation_id,event) SELECT NEW.publication_id,NEW.schedule_id,NEW.id,'package_regenerated' WHERE NEW.status='prepared' AND NEW.regenerated_from IS NOT NULL;
 UPDATE assisted_confirmations SET status='requires_review',version=version+1 WHERE preparation_id=NEW.id AND status='confirmed_manually' AND NEW.status!='prepared';
END;
CREATE TRIGGER confirmations_validate BEFORE INSERT ON assisted_confirmations WHEN NEW.status!='confirmed_manually' OR NOT EXISTS(
 SELECT 1 FROM preparations x JOIN schedules s ON s.id=x.schedule_id JOIN publications p ON p.id=s.publication_id WHERE x.id=NEW.preparation_id AND x.status='prepared' AND x.schedule_id=NEW.schedule_id AND x.publication_id=NEW.publication_id AND x.fingerprint=NEW.fingerprint AND s.version=NEW.schedule_version AND x.schedule_version=s.version AND p.version=NEW.publication_version AND x.publication_version=p.version AND s.status='planned_local' AND p.editorial_status='approved' AND p.deleted_at IS NULL
) BEGIN SELECT RAISE(ABORT,'Current prepared package required'); END;
CREATE TRIGGER confirmations_created AFTER INSERT ON assisted_confirmations BEGIN
 INSERT INTO assisted_events(publication_id,schedule_id,preparation_id,confirmation_id,event) VALUES(NEW.publication_id,NEW.schedule_id,NEW.preparation_id,NEW.id,'confirmation_recorded');
 INSERT INTO assisted_events(publication_id,schedule_id,preparation_id,confirmation_id,event) SELECT NEW.publication_id,NEW.schedule_id,NEW.preparation_id,NEW.id,'divergence_detected' WHERE NEW.planned_utc!=NEW.actual_utc;
END;
CREATE TRIGGER confirmations_changed BEFORE UPDATE ON assisted_confirmations WHEN NEW.version!=OLD.version+1 OR OLD.status='superseded' OR NEW.status NOT IN ('requires_review','superseded') OR NEW.id!=OLD.id OR NEW.preparation_id!=OLD.preparation_id OR NEW.actual_utc!=OLD.actual_utc OR NEW.note!=OLD.note OR NEW.reference!=OLD.reference BEGIN SELECT RAISE(ABORT,'Preserve confirmation snapshot'); END;
CREATE TRIGGER confirmations_snapshot BEFORE UPDATE ON assisted_confirmations WHEN NEW.publication_id!=OLD.publication_id OR NEW.schedule_id!=OLD.schedule_id OR NEW.publication_version!=OLD.publication_version OR NEW.schedule_version!=OLD.schedule_version OR NEW.fingerprint!=OLD.fingerprint OR NEW.planned_date!=OLD.planned_date OR NEW.planned_time!=OLD.planned_time OR NEW.planned_timezone!=OLD.planned_timezone OR NEW.planned_utc!=OLD.planned_utc OR NEW.actual_date!=OLD.actual_date OR NEW.actual_time!=OLD.actual_time OR NEW.actual_timezone!=OLD.actual_timezone OR NEW.platform!=OLD.platform OR NEW.format!=OLD.format OR NEW.confirmed_at!=OLD.confirmed_at BEGIN SELECT RAISE(ABORT,'Preserve confirmation snapshot'); END;
CREATE TRIGGER confirmations_review AFTER UPDATE OF status ON assisted_confirmations WHEN NEW.status='requires_review' AND OLD.status!='requires_review' BEGIN
 INSERT INTO assisted_events(publication_id,schedule_id,preparation_id,confirmation_id,event) VALUES(NEW.publication_id,NEW.schedule_id,NEW.preparation_id,NEW.id,'divergence_detected');
END;
CREATE TRIGGER assisted_schedule_changed AFTER UPDATE OF version,status ON schedules WHEN NEW.version!=OLD.version OR NEW.status!='planned_local' BEGIN
 UPDATE preparations SET status='outdated',updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE schedule_id=NEW.id AND status IN ('preparing','prepared');
 UPDATE assisted_confirmations SET status='requires_review',version=version+1 WHERE schedule_id=NEW.id AND status='confirmed_manually';
END;
CREATE TRIGGER assisted_editorial_changed AFTER UPDATE OF version,editorial_status,deleted_at ON publications WHEN NEW.version!=OLD.version OR NEW.editorial_status!='approved' OR NEW.deleted_at IS NOT NULL BEGIN
 UPDATE preparations SET status='outdated',updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE publication_id=NEW.id AND status IN ('preparing','prepared');
 UPDATE assisted_confirmations SET status='requires_review',version=version+1 WHERE publication_id=NEW.id AND status='confirmed_manually';
END;
