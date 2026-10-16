use crate::{
    editorial_domain::*,
    editorial_repository::{normalized, EditorialRepository},
    error::AppError,
};
use rusqlite::{params, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewIssue {
    pub code: &'static str,
    pub severity: &'static str,
    pub message: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewItem {
    pub publication: Publication,
    pub board_id: Option<String>,
    pub board_name: Option<String>,
    pub source_list: Option<String>,
    pub card_url: Option<String>,
    pub issues: Vec<ReviewIssue>,
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewQuery {
    #[serde(default)]
    pub search: String,
    pub status: Option<String>,
    pub format: Option<PublicationFormat>,
    pub source: Option<String>,
    pub board_id: Option<String>,
    pub source_list: Option<String>,
    pub has_media: Option<bool>,
    #[serde(default)]
    pub issues_only: bool,
    #[serde(default)]
    pub offset: u32,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewOption {
    pub id: String,
    pub name: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewPage {
    pub items: Vec<ReviewItem>,
    pub order: Vec<String>,
    pub total: usize,
    pub counts: EditorialCounts,
    pub boards: Vec<ReviewOption>,
    pub lists: Vec<String>,
}
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewAction {
    Approve,
    Ignore,
    Reopen,
    Submit,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewTarget {
    pub id: String,
    pub expected_version: u32,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewDecision {
    pub targets: Vec<ReviewTarget>,
    pub action: ReviewAction,
    pub reason: Option<String>,
    pub note: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionResult {
    pub id: String,
    pub success: bool,
    pub error: Option<AppError>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewEvent {
    pub id: String,
    pub publication_id: String,
    pub title: String,
    pub event: String,
    pub previous_status: String,
    pub next_status: String,
    pub reviewed_version: u32,
    pub resulting_version: u32,
    pub reason: Option<String>,
    pub note: Option<String>,
    pub created_at: String,
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HistoryQuery {
    pub publication_id: Option<String>,
    #[serde(default)]
    pub search: String,
    pub event: Option<String>,
    #[serde(default)]
    pub offset: u32,
}
#[derive(Serialize)]
pub struct HistoryPage {
    pub items: Vec<ReviewEvent>,
    pub total: u32,
}

fn issue(code: &'static str, severity: &'static str, message: impl Into<String>) -> ReviewIssue {
    ReviewIssue {
        code,
        severity,
        message: message.into(),
    }
}
fn review_error(message: &'static str) -> AppError {
    AppError {
        code: "REVIEW_INVALID",
        message,
    }
}
impl EditorialRepository {
    pub fn review_item(&self, id: &str, full: bool) -> Result<ReviewItem, AppError> {
        let publication = self.get(id)?;
        let origin: Option<(String,String,String,String)> = self.database.connection.query_row(
            "SELECT b.id,b.name,c.source_list,c.metadata_json FROM imported_cards c JOIN trello_boards b ON b.id=c.board_id WHERE c.publication_id=?1", [id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
        let (board_id, board_name, source_list, card_url) =
            if let Some((id, name, list, metadata)) = origin {
                let card: crate::trello_domain::RemoteCard =
                    serde_json::from_str(&metadata).map_err(|_| AppError::database())?;
                (Some(id), Some(name), Some(list), Some(card.url()))
            } else {
                (None, None, None, None)
            };
        let mut issues = Vec::new();
        let format = match publication.format.as_str() {
            "feed" => PublicationFormat::Feed,
            "carousel" => PublicationFormat::Carousel,
            "reels" => PublicationFormat::Reels,
            _ => PublicationFormat::Stories,
        };
        if publication.media.is_empty() {
            issues.push(issue(
                "NO_MEDIA",
                "error",
                "Adicione uma mídia antes de aprovar.",
            ));
        } else if format.validate(&publication.media).is_err() {
            issues.push(issue(
                "FORMAT",
                "error",
                "Quantidade ou tipo de mídia incompatível com o formato editorial selecionado.",
            ));
        }
        for media in &publication.media {
            let valid = self.storage.path(&media.storage_path).and_then(|path| {
                let metadata = std::fs::metadata(&path).map_err(|_| AppError::file())?;
                if !metadata.is_file() || metadata.len() != media.size_bytes {
                    return Err(AppError::media());
                }
                if full {
                    crate::media_store::verify_asset(&path, media)?;
                }
                Ok(())
            });
            if valid.is_err() {
                issues.push(issue("MEDIA_UNAVAILABLE","error",format!("{}: arquivo ausente, alterado ou inválido. Reassocie uma cópia válida pelo editor.",media.filename)));
            }
        }
        if publication
            .media
            .iter()
            .any(|m| m.mime_type.starts_with("image/"))
            && publication.media.iter().any(|m| m.mime_type == "video/mp4")
        {
            issues.push(issue("MIXED","warning","Composição mista: confira a intenção editorial. Requisitos da plataforma serão validados na Fase 07."));
        }
        if publication.caption.trim().is_empty() {
            issues.push(issue(
                "EMPTY_CAPTION",
                "warning",
                "Publicação sem legenda; confira se isso atende à campanha.",
            ));
        }
        if publication.media.iter().any(|m| m.mime_type == "video/mp4") {
            issues.push(issue("PLAYBACK","warning","Confira a reprodução local do vídeo. A validação estrutural não certifica todos os codecs do WebView2."));
        }
        Ok(ReviewItem {
            publication,
            board_id,
            board_name,
            source_list,
            card_url,
            issues,
        })
    }

    pub fn review_list(&self, query: ReviewQuery) -> Result<ReviewPage, AppError> {
        if query.search.chars().count() > 200
            || query.offset > 100_000
            || query.status.as_deref().is_some_and(|s| {
                !["draft", "pending_review", "editing", "approved", "ignored"].contains(&s)
            })
            || query
                .source
                .as_deref()
                .is_some_and(|s| !["manual", "trello"].contains(&s))
            || query
                .board_id
                .as_ref()
                .is_some_and(|s| Uuid::parse_str(s).is_err())
            || query
                .source_list
                .as_ref()
                .is_some_and(|s| s.chars().count() > 500)
        {
            return Err(AppError::validation());
        }
        let mut statement=self.database.connection.prepare("SELECT p.id FROM publications p LEFT JOIN imported_cards c ON c.publication_id=p.id WHERE p.deleted_at IS NULL AND (?1='' OR instr(p.search_text,?1)>0) AND (?2 IS NULL OR p.editorial_status=?2) AND (?3 IS NULL OR p.format=?3) AND (?4 IS NULL OR p.source=?4) AND (?5 IS NULL OR c.board_id=?5) AND (?6 IS NULL OR c.source_list=?6) AND (?7 IS NULL OR EXISTS(SELECT 1 FROM publication_media pm WHERE pm.publication_id=p.id)=?7) ORDER BY p.created_at,p.id LIMIT 10001")?;
        let ids = statement
            .query_map(
                params![
                    normalized(query.search.trim()),
                    query.status,
                    query.format.map(PublicationFormat::as_str),
                    query.source,
                    query.board_id,
                    query.source_list,
                    query.has_media
                ],
                |r| r.get::<_, String>(0),
            )?
            .collect::<Result<Vec<_>, _>>()?;
        if ids.len() > 10000 {
            return Err(review_error(
                "Refine os filtros para revisar até 10.000 publicações por consulta.",
            ));
        }
        let mut items = Vec::new();
        let mut total = 0;
        let mut order = Vec::new();
        for id in ids {
            let item = self.review_item(&id, false)?;
            if query.issues_only && item.issues.is_empty() {
                continue;
            }
            order.push(item.publication.id.clone());
            if total >= query.offset as usize && items.len() < 30 {
                items.push(item);
            }
            total += 1;
        }
        let boards = self
            .database
            .connection
            .prepare("SELECT id,name FROM trello_boards ORDER BY name,id")?
            .query_map([], |r| {
                Ok(ReviewOption {
                    id: r.get(0)?,
                    name: r.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let lists=self.database.connection.prepare("SELECT DISTINCT source_list FROM imported_cards WHERE (?1 IS NULL OR board_id=?1) ORDER BY source_list")?.query_map([query.board_id],|r|r.get(0))?.collect::<Result<Vec<_>,_>>()?;
        Ok(ReviewPage {
            order,
            items,
            total,
            counts: self.counts()?,
            boards,
            lists,
        })
    }

    pub fn review_decide(
        &mut self,
        input: ReviewDecision,
    ) -> Result<Vec<DecisionResult>, AppError> {
        let mut seen = std::collections::HashSet::new();
        if input.targets.is_empty()
            || input.targets.len() > 100
            || input.targets.iter().any(|t| {
                Uuid::parse_str(&t.id).is_err() || t.expected_version == 0 || !seen.insert(&t.id)
            })
            || input.reason.as_deref().is_some_and(|r| {
                !["product", "campaign", "irrelevant", "duplicate", "other"].contains(&r)
            })
            || input
                .note
                .as_ref()
                .is_some_and(|n| n.chars().count() > 2000 || n.contains('\0'))
            || (!matches!(input.action, ReviewAction::Ignore)
                && (input.reason.is_some() || input.note.is_some()))
        {
            return Err(AppError::validation());
        }
        let mut results = Vec::new();
        for target in &input.targets {
            let result = self.review_one(
                target,
                input.action,
                input.reason.as_deref(),
                input.note.as_deref(),
            );
            results.push(DecisionResult {
                id: target.id.clone(),
                success: result.is_ok(),
                error: result.err(),
            });
        }
        Ok(results)
    }
    fn review_one(
        &mut self,
        target: &ReviewTarget,
        action: ReviewAction,
        reason: Option<&str>,
        note: Option<&str>,
    ) -> Result<(), AppError> {
        let item = self.review_item(&target.id, matches!(action, ReviewAction::Approve))?;
        let post = &item.publication;
        if post.version != target.expected_version {
            return Err(AppError::conflict());
        }
        let (next,event)=match (action,post.editorial_status.as_str()) {
            (ReviewAction::Approve,"pending_review"|"editing") => {
                if item.issues.iter().any(|i|i.severity=="error") {return Err(review_error("Aprovação bloqueada. Corrija as mídias e o formato indicados antes de aprovar."));}
                ("approved","approved")
            },
            (ReviewAction::Ignore,"pending_review"|"editing") => ("ignored","ignored"),
            (ReviewAction::Reopen,"approved"|"ignored"|"editing") => ("pending_review","returned"),
            (ReviewAction::Submit,"draft") => ("pending_review","submitted"),
            _ => return Err(review_error("Transição indisponível para o estado atual. Retorne a publicação à revisão antes de decidir novamente.")),
        };
        let tx = self
            .database
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed=tx.execute("UPDATE publications SET editorial_status=?1,version=version+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?2 AND version=?3 AND editorial_status=?4 AND deleted_at IS NULL",params![next,target.id,target.expected_version,post.editorial_status])?;
        if changed != 1 {
            return Err(AppError::conflict());
        }
        tx.execute("INSERT INTO editorial_reviews(id,publication_id,event,previous_status,next_status,reviewed_version,resulting_version,reason,note) VALUES(?1,?2,?3,?4,?5,?6,?6+1,?7,?8)",params![Uuid::new_v4().to_string(),target.id,event,post.editorial_status,next,post.version,reason,note.filter(|n|!n.trim().is_empty())])?;
        tx.commit()?;
        Ok(())
    }

    pub fn review_history(&self, query: HistoryQuery) -> Result<HistoryPage, AppError> {
        if query.search.chars().count() > 200
            || query.offset > 100_000
            || query
                .publication_id
                .as_ref()
                .is_some_and(|s| Uuid::parse_str(s).is_err())
            || query.event.as_deref().is_some_and(|s| {
                !["approved", "ignored", "returned", "submitted", "edited"].contains(&s)
            })
        {
            return Err(AppError::validation());
        }
        let clause="(?1 IS NULL OR e.publication_id=?1) AND (?2='' OR instr(p.search_text,?2)>0) AND (?3 IS NULL OR e.event=?3)";
        let search = normalized(query.search.trim());
        let total=self.database.connection.query_row(&format!("SELECT COUNT(*) FROM editorial_reviews e JOIN publications p ON p.id=e.publication_id WHERE {clause}"),params![query.publication_id,search,query.event],|r|r.get(0))?;
        let items=self.database.connection.prepare(&format!("SELECT e.id,e.publication_id,p.title,e.event,e.previous_status,e.next_status,e.reviewed_version,e.resulting_version,e.reason,e.note,e.created_at FROM editorial_reviews e JOIN publications p ON p.id=e.publication_id WHERE {clause} ORDER BY e.created_at DESC,e.rowid DESC LIMIT 30 OFFSET ?4"))?.query_map(params![query.publication_id,search,query.event,query.offset],|r|Ok(ReviewEvent{id:r.get(0)?,publication_id:r.get(1)?,title:r.get(2)?,event:r.get(3)?,previous_status:r.get(4)?,next_status:r.get(5)?,reviewed_version:r.get(6)?,resulting_version:r.get(7)?,reason:r.get(8)?,note:r.get(9)?,created_at:r.get(10)?}))?.collect::<Result<Vec<_>,_>>()?;
        Ok(HistoryPage { items, total })
    }
}
