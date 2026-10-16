CREATE TABLE editorial_reviews (
    id TEXT PRIMARY KEY NOT NULL,
    publication_id TEXT NOT NULL REFERENCES publications(id) ON DELETE RESTRICT,
    event TEXT NOT NULL CHECK (event IN ('approved','ignored','returned','submitted','edited')),
    previous_status TEXT NOT NULL CHECK (previous_status IN ('draft','pending_review','editing','approved','ignored','ready_to_schedule')),
    next_status TEXT NOT NULL CHECK (next_status IN ('draft','pending_review','editing','approved','ignored','ready_to_schedule')),
    reviewed_version INTEGER NOT NULL CHECK (reviewed_version > 0),
    resulting_version INTEGER NOT NULL CHECK (resulting_version = reviewed_version + 1),
    reason TEXT CHECK (reason IS NULL OR reason IN ('product','campaign','irrelevant','duplicate','other')),
    note TEXT CHECK (note IS NULL OR length(note) <= 2000),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    CHECK (event = 'ignored' OR (reason IS NULL AND note IS NULL))
);
CREATE INDEX editorial_reviews_publication ON editorial_reviews(publication_id, created_at DESC);
CREATE INDEX editorial_reviews_recent ON editorial_reviews(created_at DESC, id);
CREATE TRIGGER editorial_reviews_no_update BEFORE UPDATE ON editorial_reviews BEGIN
    SELECT RAISE(ABORT, 'Editorial history is immutable');
END;
CREATE TRIGGER editorial_reviews_no_delete BEFORE DELETE ON editorial_reviews BEGIN
    SELECT RAISE(ABORT, 'Editorial history is immutable');
END;
