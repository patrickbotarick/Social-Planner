use crate::{
    editorial_domain::MediaAsset,
    editorial_repository::{asset, normalized, with_preview, EditorialRepository},
    error::AppError,
    trello_domain::*,
};
use rusqlite::{params, OptionalExtension, TransactionBehavior};
use uuid::Uuid;

pub struct AttachmentResult {
    pub attachment: Attachment,
    pub media: Option<MediaAsset>,
    pub status: String,
    pub error_message: Option<String>,
}
impl EditorialRepository {
    pub fn trello_preview(
        &self,
        board: Board,
        lists: Vec<TrelloList>,
        cards: Vec<RemoteCard>,
    ) -> Result<BoardPreview, AppError> {
        let last_synced_at = self
            .database
            .connection
            .query_row(
                "SELECT last_synced_at FROM trello_boards WHERE remote_id=?1",
                [&board.id],
                |r| r.get(0),
            )
            .optional()?
            .flatten();
        let mut result = Vec::new();
        for card in cards {
            let prior: Option<(String, String)> = self
                .database
                .connection
                .query_row(
                    "SELECT publication_id,source_hash FROM imported_cards WHERE remote_id=?1",
                    [&card.id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?;
            let state = if card.closed {
                "archived"
            } else {
                match &prior {
                    None => "new",
                    Some((_, hash)) if hash != &card.hash() => "changed",
                    _ => "imported",
                }
            };
            result.push(PreviewCard {
                card,
                state: state.into(),
                publication_id: prior.map(|p| p.0),
            });
        }
        Ok(BoardPreview {
            preview_id: Uuid::new_v4().to_string(),
            board,
            lists,
            cards: result,
            last_synced_at,
        })
    }
    pub fn trello_begin(&mut self, board: &Board, total: u32) -> Result<Job, AppError> {
        let tx = self
            .database
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute("INSERT INTO trello_boards(id,remote_id,name,url) VALUES(?1,?2,?3,?4) ON CONFLICT(remote_id) DO UPDATE SET name=excluded.name,url=excluded.url",params![Uuid::new_v4().to_string(),board.id,board.name,board.url()])?;
        let board_id: String = tx.query_row(
            "SELECT id FROM trello_boards WHERE remote_id=?1",
            [&board.id],
            |r| r.get(0),
        )?;
        let job = Job {
            id: Uuid::new_v4().to_string(),
            status: "running".into(),
            total_cards: total,
            ..Job::default()
        };
        tx.execute(
            "INSERT INTO import_jobs(id,board_id,status,total_cards) VALUES(?1,?2,'running',?3)",
            params![job.id, board_id, total],
        )?;
        tx.commit()?;
        Ok(job)
    }
    pub fn trello_job_save(&self, job: &Job) -> Result<(), AppError> {
        self.database.connection.execute("UPDATE import_jobs SET status=?2,processed_cards=?3,successful_cards=?4,failed_cards=?5,new_publications=?6,changed_sources=?7,downloaded_attachments=?8,attachment_failures=?9,error_message=?10,finished_at=CASE WHEN ?2='running' THEN NULL ELSE strftime('%Y-%m-%dT%H:%M:%fZ','now') END WHERE id=?1",params![job.id,job.status,job.processed_cards,job.successful_cards,job.failed_cards,job.new_publications,job.changed_sources,job.downloaded_attachments,job.attachment_failures,job.error_message])?;
        Ok(())
    }
    pub fn trello_recover(&self) -> Result<(), AppError> {
        self.database.connection.execute("UPDATE import_jobs SET status='interrupted',error_message='Aplicativo fechado antes da conclusão. Consulte o quadro e repita a seleção.',finished_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE status='running'",[])?;
        Ok(())
    }
    pub fn trello_history(&self) -> Result<Vec<BoardHistory>, AppError> {
        let mut q = self.database.connection.prepare(
            "SELECT name,url,last_synced_at FROM trello_boards ORDER BY created_at DESC LIMIT 100",
        )?;
        let rows = q
            .query_map([], |r| {
                Ok(BoardHistory {
                    name: r.get(0)?,
                    url: r.get(1)?,
                    last_synced_at: r.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
    pub fn trello_jobs(&self) -> Result<Vec<Job>, AppError> {
        let mut q=self.database.connection.prepare("SELECT id,status,total_cards,processed_cards,successful_cards,failed_cards,new_publications,changed_sources,downloaded_attachments,attachment_failures,error_message FROM import_jobs ORDER BY started_at DESC LIMIT 20")?;
        let rows = q
            .query_map([], |r| {
                Ok(Job {
                    id: r.get(0)?,
                    status: r.get(1)?,
                    total_cards: r.get(2)?,
                    processed_cards: r.get(3)?,
                    successful_cards: r.get(4)?,
                    failed_cards: r.get(5)?,
                    new_publications: r.get(6)?,
                    changed_sources: r.get(7)?,
                    downloaded_attachments: r.get(8)?,
                    attachment_failures: r.get(9)?,
                    error_message: r.get(10)?,
                    ..Job::default()
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
    pub fn trello_existing_media(&self, a: &Attachment) -> Result<Option<MediaAsset>, AppError> {
        let id:Option<String>=self.database.connection.query_row("SELECT media_asset_id FROM trello_attachments WHERE remote_id=?1 AND filename=?2 AND download_status='available'",params![a.id,a.name],|r|r.get(0)).optional()?.flatten();
        if let Some(id) = id {
            let media = asset(&self.database.connection, &id)?;
            if self.storage.path(&media.storage_path).is_ok() {
                return Ok(Some(media));
            }
        }
        Ok(None)
    }
    // Arquivos já validados/promovidos. A transação grava snapshot e vínculos de forma indivisível.
    pub fn trello_commit_card(
        &mut self,
        board: &Board,
        card: &RemoteCard,
        list: &str,
        results: &[AttachmentResult],
    ) -> Result<(bool, bool), AppError> {
        let tx = self
            .database
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        // Verifica novamente sob o lock transacional: uma operação editorial concorrente
        // pode ter revertido um arquivo que existia quando o download foi promovido.
        for media in results.iter().filter_map(|r| r.media.as_ref()) {
            if crate::media_store::checksum(&self.storage.path(&media.storage_path)?)?
                != media.checksum
            {
                return Err(AppError::file());
            }
        }
        let board_id: String = tx.query_row(
            "SELECT id FROM trello_boards WHERE remote_id=?1",
            [&board.id],
            |r| r.get(0),
        )?;
        let existing: Option<(String, String, String)> = tx
            .query_row(
                "SELECT id,publication_id,source_hash FROM imported_cards WHERE remote_id=?1",
                [&card.id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        let new = existing.is_none();
        let hash = card.hash();
        let changed = existing.as_ref().is_some_and(|e| e.2 != hash);
        let (id, publication_id) = existing
            .map(|e| (e.0, e.1))
            .unwrap_or_else(|| (Uuid::new_v4().to_string(), Uuid::new_v4().to_string()));
        let available: Vec<_> = results.iter().filter_map(|r| r.media.as_ref()).collect();
        let mut unique = std::collections::HashSet::new();
        let compatible = !available.is_empty()
            && available.len() <= 20
            && available.iter().all(|m| unique.insert(&m.checksum));
        let mixed = available.iter().any(|m| m.mime_type.starts_with("image/"))
            && available.iter().any(|m| m.mime_type == "video/mp4");
        let needs_adjustment = !compatible
            || mixed
            || results.iter().any(|r| r.status != "available")
            || card.name.chars().count() > 200
            || card.desc.chars().count() > 20000
            || card.name.trim().is_empty();
        if new {
            let title = if card.name.trim().is_empty() {
                "Cartão sem título".into()
            } else {
                card.name.chars().take(200).collect::<String>()
            };
            let caption = card.desc.chars().take(20000).collect::<String>();
            let format = if compatible && available.len() > 1 {
                "carousel"
            } else if compatible && available[0].mime_type == "video/mp4" {
                "reels"
            } else {
                "feed"
            };
            tx.execute("INSERT INTO publications(id,title,caption,search_text,format,source,editorial_status) VALUES(?1,?2,?3,?4,?5,'trello','pending_review')",params![publication_id,title,caption,normalized(&format!("{title} {caption}")),format])?;
        }
        let metadata = serde_json::to_string(card).map_err(|_| error("TRELLO_RESPONSE"))?;
        tx.execute("INSERT INTO imported_cards(id,remote_id,board_id,publication_id,original_title,original_description,source_list,metadata_json,source_hash,source_updated_at,needs_adjustment) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11) ON CONFLICT(remote_id) DO UPDATE SET board_id=excluded.board_id,original_title=excluded.original_title,original_description=excluded.original_description,source_list=excluded.source_list,metadata_json=excluded.metadata_json,source_changed=MAX(source_changed,source_hash<>excluded.source_hash),source_hash=excluded.source_hash,source_updated_at=excluded.source_updated_at,needs_adjustment=excluded.needs_adjustment,source_missing=0,last_synced_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')",params![id,card.id,board_id,publication_id,card.name,card.desc,list,metadata,hash,card.date_last_activity,needs_adjustment])?;
        tx.execute(
            "UPDATE trello_attachments SET removed=1 WHERE imported_card_id=?1",
            [&id],
        )?;
        let mut position = 0;
        for result in results {
            let a = &result.attachment;
            let asset_id = if let Some(m) = &result.media {
                tx.execute("INSERT INTO media_assets(id,filename,mime_type,extension,size_bytes,storage_path,width,height,duration,checksum) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10) ON CONFLICT(checksum) DO NOTHING",params![m.id,m.filename,m.mime_type,m.extension,m.size_bytes as i64,m.storage_path,m.width,m.height,m.duration,m.checksum])?;
                Some(tx.query_row(
                    "SELECT id FROM media_assets WHERE checksum=?1",
                    [&m.checksum],
                    |r| r.get::<_, String>(0),
                )?)
            } else {
                None
            };
            // Falha não apaga o asset anteriormente baixado; remoção remota não remove o arquivo.
            tx.execute("INSERT INTO trello_attachments(remote_id,imported_card_id,filename,mime_type,source_url,media_asset_id,download_status,error_message) VALUES(?1,?2,?3,?4,?5,?6,?7,?8) ON CONFLICT(remote_id) DO UPDATE SET filename=excluded.filename,mime_type=excluded.mime_type,source_url=excluded.source_url,media_asset_id=COALESCE(excluded.media_asset_id,media_asset_id),download_status=excluded.download_status,error_message=excluded.error_message,removed=0",params![a.id,id,a.name,a.mime_type,a.url,asset_id,result.status,result.error_message])?;
            if new && compatible {
                if let Some(asset_id) = asset_id {
                    tx.execute("INSERT INTO publication_media(publication_id,media_asset_id,sort_order) VALUES(?1,?2,?3)",params![publication_id,asset_id,position])?;
                    position += 1;
                }
            }
        }
        tx.commit()?;
        Ok((new, changed))
    }
    pub fn trello_finish_snapshot(
        &mut self,
        board: &Board,
        cards: &[PreviewCard],
        complete: bool,
    ) -> Result<(), AppError> {
        let tx = self
            .database
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        // Ausência só é inferida após a leitura completa, nunca após uma página/erro de rede.
        tx.execute("UPDATE imported_cards SET source_missing=1 WHERE board_id=(SELECT id FROM trello_boards WHERE remote_id=?1)",[&board.id])?;
        for c in cards {
            tx.execute("UPDATE imported_cards SET source_missing=?2 WHERE remote_id=?1 AND board_id=(SELECT id FROM trello_boards WHERE remote_id=?3)",params![c.card.id,c.card.closed,board.id])?;
        }
        if complete {
            tx.execute("UPDATE trello_boards SET last_synced_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE remote_id=?1",[&board.id])?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn trello_source(&self, publication: &str) -> Result<Option<SourceInfo>, AppError> {
        let row:Option<(String,String,String,bool,bool,bool,String)>=self.database.connection.query_row("SELECT id,original_title,original_description,source_changed,source_missing,needs_adjustment,metadata_json FROM imported_cards WHERE publication_id=?1",[publication],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?))).optional()?;
        let Some((id, title, desc, changed, missing, adjustment, metadata)) = row else {
            return Ok(None);
        };
        let card: RemoteCard =
            serde_json::from_str(&metadata).map_err(|_| error("TRELLO_RESPONSE"))?;
        let mut q=self.database.connection.prepare("SELECT filename,source_url,download_status,error_message,removed,media_asset_id FROM trello_attachments WHERE imported_card_id=?1 ORDER BY remote_id")?;
        let rows = q.query_map([id], |r| {
            Ok((
                SourceAttachment {
                    name: r.get(0)?,
                    url: r.get(1)?,
                    status: r.get(2)?,
                    error_message: r.get(3)?,
                    removed: r.get(4)?,
                    media: None,
                },
                r.get::<_, Option<String>>(5)?,
            ))
        })?;
        let mut attachments = Vec::new();
        for row in rows {
            let (mut a, media) = row?;
            if let Some(id) = media {
                a.media = Some(with_preview(
                    asset(&self.database.connection, &id)?,
                    &self.storage,
                ));
            }
            attachments.push(a);
        }
        Ok(Some(SourceInfo {
            url: card.url(),
            original_title: title,
            original_description: desc,
            source_changed: changed,
            source_missing: missing,
            needs_adjustment: adjustment,
            attachments,
        }))
    }
}
