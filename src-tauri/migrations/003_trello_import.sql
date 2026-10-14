CREATE TABLE trello_boards (
 id TEXT PRIMARY KEY NOT NULL,
 remote_id TEXT NOT NULL UNIQUE,
 name TEXT NOT NULL,
 url TEXT NOT NULL,
 last_synced_at TEXT,
 created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
CREATE TABLE imported_cards (
 id TEXT PRIMARY KEY NOT NULL,
 remote_id TEXT NOT NULL UNIQUE,
 board_id TEXT NOT NULL REFERENCES trello_boards(id),
 publication_id TEXT NOT NULL UNIQUE REFERENCES publications(id),
 original_title TEXT NOT NULL,
 original_description TEXT NOT NULL,
 source_list TEXT NOT NULL,
 metadata_json TEXT NOT NULL,
 source_hash TEXT NOT NULL,
 source_updated_at TEXT,
 source_changed INTEGER NOT NULL DEFAULT 0 CHECK(source_changed IN (0,1)),
 source_missing INTEGER NOT NULL DEFAULT 0 CHECK(source_missing IN (0,1)),
 needs_adjustment INTEGER NOT NULL DEFAULT 0 CHECK(needs_adjustment IN (0,1)),
 last_synced_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
CREATE TABLE trello_attachments (
 remote_id TEXT PRIMARY KEY NOT NULL,
 imported_card_id TEXT NOT NULL REFERENCES imported_cards(id),
 filename TEXT NOT NULL,
 mime_type TEXT,
 source_url TEXT NOT NULL,
 media_asset_id TEXT REFERENCES media_assets(id),
 download_status TEXT NOT NULL CHECK(download_status IN ('available','failed','reference','unsupported')),
 error_message TEXT,
 removed INTEGER NOT NULL DEFAULT 0 CHECK(removed IN (0,1))
);
CREATE TABLE import_jobs (
 id TEXT PRIMARY KEY NOT NULL,
 board_id TEXT NOT NULL REFERENCES trello_boards(id),
 status TEXT NOT NULL CHECK(status IN ('running','completed','partial','cancelled','interrupted','failed')),
 total_cards INTEGER NOT NULL CHECK(total_cards >= 0),
 processed_cards INTEGER NOT NULL DEFAULT 0,
 successful_cards INTEGER NOT NULL DEFAULT 0,
 failed_cards INTEGER NOT NULL DEFAULT 0,
 new_publications INTEGER NOT NULL DEFAULT 0,
 changed_sources INTEGER NOT NULL DEFAULT 0,
 downloaded_attachments INTEGER NOT NULL DEFAULT 0,
 attachment_failures INTEGER NOT NULL DEFAULT 0,
 error_message TEXT,
 started_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
 finished_at TEXT
);
CREATE INDEX imported_cards_board ON imported_cards(board_id);
CREATE INDEX trello_attachments_card ON trello_attachments(imported_card_id);
CREATE INDEX import_jobs_recent ON import_jobs(started_at DESC);
