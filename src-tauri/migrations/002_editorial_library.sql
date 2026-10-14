CREATE TABLE publications (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL CHECK (length(trim(title)) BETWEEN 1 AND 200),
    caption TEXT NOT NULL DEFAULT '' CHECK (length(caption) <= 20000),
    search_text TEXT NOT NULL,
    format TEXT NOT NULL CHECK (format IN ('feed', 'carousel', 'reels', 'stories')),
    source TEXT NOT NULL DEFAULT 'manual' CHECK (source IN ('manual', 'trello')),
    editorial_status TEXT NOT NULL DEFAULT 'draft'
        CHECK (editorial_status IN ('draft', 'pending_review', 'editing', 'approved', 'ignored', 'ready_to_schedule')),
    version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    deleted_at TEXT
);

CREATE TABLE media_assets (
    id TEXT PRIMARY KEY NOT NULL,
    filename TEXT NOT NULL,
    mime_type TEXT NOT NULL CHECK (mime_type IN ('image/jpeg', 'image/png', 'image/webp', 'video/mp4')),
    extension TEXT NOT NULL CHECK (extension IN ('jpg', 'png', 'webp', 'mp4')),
    size_bytes INTEGER NOT NULL CHECK (size_bytes > 0),
    storage_path TEXT NOT NULL UNIQUE CHECK (storage_path = checksum || '.' || extension),
    width INTEGER CHECK (width > 0),
    height INTEGER CHECK (height > 0),
    duration REAL CHECK (duration >= 0),
    checksum TEXT NOT NULL UNIQUE CHECK (length(checksum) = 64 AND checksum NOT GLOB '*[^0-9a-f]*'),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE publication_media (
    publication_id TEXT NOT NULL REFERENCES publications(id) ON DELETE CASCADE,
    media_asset_id TEXT NOT NULL REFERENCES media_assets(id) ON DELETE RESTRICT,
    sort_order INTEGER NOT NULL CHECK (sort_order >= 0),
    PRIMARY KEY (publication_id, media_asset_id),
    UNIQUE (publication_id, sort_order)
);

CREATE INDEX publications_active_created ON publications(created_at DESC, id) WHERE deleted_at IS NULL;
CREATE INDEX publications_active_updated ON publications(updated_at DESC, id) WHERE deleted_at IS NULL;
CREATE INDEX publications_active_filters ON publications(format, source, editorial_status) WHERE deleted_at IS NULL;
CREATE INDEX publication_media_asset ON publication_media(media_asset_id);
