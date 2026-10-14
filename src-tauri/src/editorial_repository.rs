use crate::{database::Repository, editorial_domain::*, error::AppError, media_store::MediaStore};
use rusqlite::{params, Connection, OptionalExtension, Row, TransactionBehavior};
use std::path::Path;
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};
use uuid::Uuid;

pub struct EditorialRepository {
    pub database: Repository,
    pub storage: MediaStore,
}
fn normalized(text: &str) -> String {
    text.nfd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .collect()
}
fn media_row(row: &Row<'_>) -> rusqlite::Result<MediaAsset> {
    Ok(MediaAsset {
        id: row.get(0)?,
        filename: row.get(1)?,
        mime_type: row.get(2)?,
        extension: row.get(3)?,
        size_bytes: row.get::<_, u32>(4)?.into(),
        storage_path: row.get(5)?,
        width: row.get(6)?,
        height: row.get(7)?,
        duration: row.get(8)?,
        checksum: row.get(9)?,
        created_at: row.get(10)?,
        preview_path: None,
    })
}
const MEDIA_COLUMNS: &str = "m.id, m.filename, m.mime_type, m.extension, m.size_bytes, m.storage_path, m.width, m.height, m.duration, m.checksum, m.created_at";
fn asset(connection: &Connection, id: &str) -> Result<MediaAsset, AppError> {
    connection
        .query_row(
            &format!("SELECT {MEDIA_COLUMNS} FROM media_assets m WHERE m.id=?1"),
            [id],
            media_row,
        )
        .optional()?
        .ok_or_else(AppError::not_found)
}
fn with_preview(mut media: MediaAsset, storage: &MediaStore) -> MediaAsset {
    media.preview_path = storage
        .path(&media.storage_path)
        .ok()
        .map(|p| p.to_string_lossy().into_owned());
    media
}

impl EditorialRepository {
    pub fn open(data: &Path) -> Result<Self, AppError> {
        Ok(Self {
            database: Repository::open(&data.join("planner.sqlite3"))?,
            storage: MediaStore::open(data)?,
        })
    }

    pub fn get(&self, id: &str) -> Result<Publication, AppError> {
        let connection = &self.database.connection;
        let mut publication = connection.query_row("SELECT id,title,caption,format,source,editorial_status,version,created_at,updated_at FROM publications WHERE id=?1 AND deleted_at IS NULL", [id], |row| Ok(Publication { id: row.get(0)?, title: row.get(1)?, caption: row.get(2)?, format: row.get(3)?, source: row.get(4)?, editorial_status: row.get(5)?, version: row.get(6)?, created_at: row.get(7)?, updated_at: row.get(8)?, media: Vec::new() })).optional()?.ok_or_else(AppError::not_found)?;
        let mut statement = connection.prepare(&format!("SELECT {MEDIA_COLUMNS} FROM publication_media pm JOIN media_assets m ON m.id=pm.media_asset_id WHERE pm.publication_id=?1 ORDER BY pm.sort_order"))?;
        let rows = statement.query_map([id], media_row)?;
        publication.media = rows
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|m| with_preview(m, &self.storage))
            .collect();
        Ok(publication)
    }

    pub fn save(&mut self, input: SavePublication) -> Result<Publication, AppError> {
        input.validate()?;
        let id = input
            .id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let media = input
            .media_ids
            .iter()
            .map(|id| {
                self.storage
                    .staged
                    .get(id)
                    .cloned()
                    .map(Ok)
                    .unwrap_or_else(|| asset(&self.database.connection, id))
            })
            .collect::<Result<Vec<_>, _>>()?;
        input.format.validate(&media)?;
        for m in &media {
            if !self.storage.staged.contains_key(&m.id) {
                self.storage.path(&m.storage_path)?;
            }
        }
        let transaction = self
            .database
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut installed = Vec::new();
        let result = (|| -> Result<(), AppError> {
            if let Some(version) = input.expected_version {
                let changed = transaction.execute("UPDATE publications SET title=?1,caption=?2,search_text=?3,format=?4,version=version+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?5 AND version=?6 AND source='manual' AND editorial_status='draft' AND deleted_at IS NULL", params![input.title.trim(), input.caption, normalized(&format!("{} {}", input.title, input.caption)), input.format.as_str(), id, version])?;
                if changed != 1 {
                    return Err(AppError::conflict());
                }
                transaction.execute(
                    "DELETE FROM publication_media WHERE publication_id=?1",
                    [&id],
                )?;
            } else {
                transaction.execute("INSERT INTO publications (id,title,caption,search_text,format) VALUES (?1,?2,?3,?4,?5)", params![id, input.title.trim(), input.caption, normalized(&format!("{} {}", input.title, input.caption)), input.format.as_str()])?;
            }
            for (position, m) in media.iter().enumerate() {
                let asset_id = if self.storage.staged.contains_key(&m.id) {
                    let existing: Option<String> = transaction
                        .query_row(
                            "SELECT id FROM media_assets WHERE checksum=?1",
                            [&m.checksum],
                            |r| r.get(0),
                        )
                        .optional()?;
                    if let Some(existing) = existing {
                        let saved = asset(&transaction, &existing)?;
                        if crate::media_store::checksum(&self.storage.path(&saved.storage_path)?)?
                            != saved.checksum
                        {
                            return Err(AppError::file());
                        }
                        existing
                    } else {
                        let (path, is_new) = self.storage.install(m)?;
                        if is_new {
                            installed.push(self.storage.directory.join(&path));
                        }
                        transaction.execute("INSERT INTO media_assets (id,filename,mime_type,extension,size_bytes,storage_path,width,height,duration,checksum) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)", params![m.id,m.filename,m.mime_type,m.extension,m.size_bytes as i64,path,m.width,m.height,m.duration,m.checksum])?;
                        m.id.clone()
                    }
                } else {
                    m.id.clone()
                };
                transaction.execute("INSERT INTO publication_media (publication_id,media_asset_id,sort_order) VALUES (?1,?2,?3)", params![id,asset_id,position as u32])?;
            }
            Ok(())
        })();
        if let Err(error) = result {
            // Somente arquivos criados por esta operação; o lock SQLite ainda está ativo.
            for path in &installed {
                let _ = std::fs::remove_file(path);
            }
            return Err(error);
        }
        // Uma falha de commit mantém os arquivos: evitar perder dados se o resultado for incerto.
        transaction.commit()?;
        if self.storage.discard(&input.media_ids).is_err() {
            eprintln!("[editorial] TEMP_CLEANUP_PENDING");
        }
        self.get(&id)
    }

    pub fn duplicate(&mut self, id: &str) -> Result<Publication, AppError> {
        let original = self.get(id)?;
        let title: String = original.title.chars().take(190).collect();
        let format = match original.format.as_str() {
            "feed" => PublicationFormat::Feed,
            "carousel" => PublicationFormat::Carousel,
            "reels" => PublicationFormat::Reels,
            _ => PublicationFormat::Stories,
        };
        self.save(SavePublication {
            id: None,
            expected_version: None,
            title: format!("{title} (cópia)"),
            caption: original.caption,
            format,
            media_ids: original.media.into_iter().map(|m| m.id).collect(),
        })
    }

    pub fn delete(&mut self, id: &str, version: u32) -> Result<(), AppError> {
        let transaction = self
            .database
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed = transaction.execute("UPDATE publications SET deleted_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),version=version+1 WHERE id=?1 AND version=?2 AND deleted_at IS NULL", params![id,version])?;
        if changed != 1 {
            return Err(AppError::conflict());
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn list(&self, query: LibraryQuery) -> Result<LibraryPage, AppError> {
        if query.search.chars().count() > 200
            || query
                .source
                .as_deref()
                .is_some_and(|s| !["manual", "trello"].contains(&s))
            || query.status.as_deref().is_some_and(|s| {
                ![
                    "draft",
                    "pending_review",
                    "editing",
                    "approved",
                    "ignored",
                    "ready_to_schedule",
                ]
                .contains(&s)
            })
            || query.offset > 1_000_000
        {
            return Err(AppError::validation());
        }
        let search = normalized(query.search.trim());
        let format = query.format.map(PublicationFormat::as_str);
        let clause = "deleted_at IS NULL AND (?1='' OR instr(search_text,?1)>0) AND (?2 IS NULL OR format=?2) AND (?3 IS NULL OR source=?3) AND (?4 IS NULL OR editorial_status=?4)";
        let total = self.database.connection.query_row(
            &format!("SELECT COUNT(*) FROM publications WHERE {clause}"),
            params![search, format, query.source, query.status],
            |r| r.get(0),
        )?;
        let order = match query.sort {
            LibrarySort::Recent => "created_at DESC,id",
            LibrarySort::Oldest => "created_at ASC,id",
            LibrarySort::Title => "title COLLATE NOCASE ASC,id",
            LibrarySort::Updated => "updated_at DESC,id",
        };
        let mut statement = self.database.connection.prepare(&format!("SELECT id,title,format,source,editorial_status,created_at,updated_at,(SELECT COUNT(*) FROM publication_media WHERE publication_id=p.id),(SELECT media_asset_id FROM publication_media WHERE publication_id=p.id ORDER BY sort_order LIMIT 1) FROM publications p WHERE {clause} ORDER BY {order} LIMIT ?5 OFFSET ?6"))?;
        let rows = statement.query_map(
            params![
                search,
                format,
                query.source,
                query.status,
                query.limit.clamp(1, 100),
                query.offset
            ],
            |r| {
                Ok((
                    PublicationSummary {
                        id: r.get(0)?,
                        title: r.get(1)?,
                        format: r.get(2)?,
                        source: r.get(3)?,
                        editorial_status: r.get(4)?,
                        created_at: r.get(5)?,
                        updated_at: r.get(6)?,
                        media_count: r.get(7)?,
                        primary_media: None,
                    },
                    r.get::<_, Option<String>>(8)?,
                ))
            },
        )?;
        let mut items = Vec::new();
        for row in rows {
            let (mut item, primary) = row?;
            if let Some(id) = primary {
                item.primary_media = Some(with_preview(
                    asset(&self.database.connection, &id)?,
                    &self.storage,
                ));
            }
            items.push(item);
        }
        Ok(LibraryPage { items, total })
    }
    pub fn counts(&self) -> Result<EditorialCounts, AppError> {
        Ok(self.database.connection.query_row("SELECT COUNT(*),COALESCE(SUM(editorial_status='draft'),0),COALESCE(SUM(editorial_status='pending_review'),0),COALESCE(SUM(editorial_status IN ('approved','ready_to_schedule')),0) FROM publications WHERE deleted_at IS NULL", [], |r| Ok(EditorialCounts { total: r.get(0)?, drafts: r.get(1)?, pending_review: r.get(2)?, approved: r.get(3)? }))?)
    }
}
