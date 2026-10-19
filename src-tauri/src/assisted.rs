use crate::{
    assisted_files::*,
    calendar::{local_instant, CalendarQuery, Schedule, ScheduleInput},
    editorial_domain::Publication,
    editorial_repository::EditorialRepository,
    error::AppError,
    review::ReviewOption,
};
use chrono::{SecondsFormat, Utc};
use rusqlite::{params, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preparation {
    pub id: String,
    pub status: String,
    pub fingerprint: String,
    pub root_id: String,
    pub relative_path: String,
    pub publication_version: u32,
    pub schedule_version: u32,
    pub created_at: String,
    pub error_code: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualConfirmation {
    pub id: String,
    pub status: String,
    pub version: u32,
    pub preparation_id: String,
    pub planned_date: String,
    pub planned_time: String,
    pub planned_timezone: String,
    pub planned_utc: String,
    pub actual_date: String,
    pub actual_time: String,
    pub actual_timezone: String,
    pub actual_utc: String,
    pub platform: String,
    pub format: String,
    pub reference: String,
    pub note: String,
    pub confirmed_at: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssistedQuery {
    pub calendar: CalendarQuery,
    pub state: Option<String>,
    #[serde(default)]
    pub offset: u32,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistedItem {
    pub schedule: Schedule,
    pub publication: Option<Publication>,
    pub preparation: Option<Preparation>,
    pub confirmation: Option<ManualConfirmation>,
    pub eligible: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub divergent: bool,
}
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistedCounts {
    pub planned: u32,
    pub not_prepared: u32,
    pub prepared: u32,
    pub confirmed: u32,
    pub outdated: u32,
    pub divergent: u32,
    pub failed: u32,
    pub pending_transfer: u32,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistedPage {
    pub items: Vec<AssistedItem>,
    pub total: usize,
    pub counts: AssistedCounts,
    pub export_root: Option<String>,
    pub boards: Vec<ReviewOption>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreparationTarget {
    pub schedule_id: String,
    pub schedule_version: u32,
    pub publication_version: u32,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparationResult {
    pub schedule_id: String,
    pub preparation_id: Option<String>,
    pub error: Option<AppError>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfirmationInput {
    pub preparation_id: String,
    pub previous_id: Option<String>,
    pub expected_version: Option<u32>,
    pub actual_date: String,
    pub actual_time: String,
    pub timezone: String,
    #[serde(default)]
    pub reference: String,
    #[serde(default)]
    pub note: String,
    pub verified_by_operator: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReconcileInput {
    pub confirmation_id: String,
    pub expected_version: u32,
    pub schedule_version: u32,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistedEvent {
    pub id: i64,
    pub title: String,
    pub schedule_id: String,
    pub preparation_id: Option<String>,
    pub confirmation_id: Option<String>,
    pub event: String,
    pub created_at: String,
}
#[derive(Serialize)]
pub struct AssistedHistory {
    pub items: Vec<AssistedEvent>,
    pub total: u32,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Plan {
    pub id: String,
    pub publication_id: String,
    pub publication_version: u32,
    pub version: u32,
    pub date: String,
    pub time: String,
    pub timezone: String,
    pub utc: String,
    pub status: String,
}
fn id(value: &str) -> Result<(), AppError> {
    Uuid::parse_str(value).map(|_| ()).map_err(|_| {
        error(
            "ASSISTED_INVALID",
            "Identificador inválido. Atualize a lista.",
        )
    })
}
fn invalid(message: &'static str) -> AppError {
    error("ASSISTED_INVALID", message)
}
fn fingerprint(plan: &Plan, p: &Publication) -> Result<String, AppError> {
    let media: Vec<_> = p
        .media
        .iter()
        .map(|m| (&m.id, &m.checksum, &m.mime_type, m.size_bytes))
        .collect();
    Ok(digest(
        &serde_json::to_vec(&(
            plan, &p.id, p.version, &p.title, &p.caption, &p.format, media,
        ))
        .map_err(|_| AppError::database())?,
    ))
}
fn warnings(p: &Publication) -> Vec<String> {
    let mut warnings = Vec::new();
    if p.media
        .iter()
        .any(|m| m.mime_type == "image/png" || m.mime_type == "image/webp")
    {
        warnings.push("Há PNG/WebP: confira a aceitação no Business Suite. Se for exigida conversão, faça-a manualmente; este pacote conserva os originais.".into());
    }
    if p.media.iter().any(|m| m.mime_type.starts_with("video/")) {
        warnings.push("Vídeo: confira codec, áudio, duração e enquadramento na Meta. A validação básica local não garante aceitação pela plataforma.".into());
    }
    if p.caption.contains("**") {
        warnings.push("Marcadores Markdown foram preservados literalmente na legenda. Revise o texto ao transferir.".into());
    }
    warnings
}
fn compatible(p: &Publication) -> Vec<String> {
    let mut issues = Vec::new();
    if p.caption.chars().count() > 2200 {
        issues.push("Legenda com mais de 2.200 caracteres: edite e aprove a versão a transferir; não haverá truncamento.".into());
    }
    if p.format == "carousel" && p.media.len() > 10 {
        issues.push("Carrossel com mais de 10 itens: revise a composição para o fluxo assistido, sem divisão automática.".into());
    }
    issues
}
fn manifest(preparation: &Preparation, plan: &Plan, p: &Publication) -> Manifest {
    Manifest{schema:1,preparation_id:preparation.id.clone(),publication_id:p.id.clone(),publication_version:p.version,schedule_id:plan.id.clone(),schedule_version:plan.version,fingerprint:preparation.fingerprint.clone(),format:p.format.clone(),scheduled_date:plan.date.clone(),scheduled_time:plan.time.clone(),timezone:plan.timezone.clone(),scheduled_at_utc:plan.utc.clone(),caption_sha256:digest(p.caption.as_bytes()),media:p.media.iter().enumerate().map(|(n,m)|ManifestMedia{filename:format!("media_{:02}.{}",n+1,m.extension),asset_id:m.id.clone(),sha256:m.checksum.clone(),bytes:m.size_bytes,mime_type:m.mime_type.clone()}).collect(),notice:"Pacote local. Nenhum envio, agendamento ou confirmação pela API Meta. A confirmação no GMAD é somente declaração do operador.".into()}
}
impl EditorialRepository {
    pub(crate) fn assisted_plan(&self, schedule_id: &str) -> Result<Plan, AppError> {
        id(schedule_id)?;
        self.database.connection.query_row("SELECT id,publication_id,publication_version,version,scheduled_date,scheduled_time,timezone,scheduled_at_utc,status FROM schedules WHERE id=?1",[schedule_id],|r|Ok(Plan{id:r.get(0)?,publication_id:r.get(1)?,publication_version:r.get(2)?,version:r.get(3)?,date:r.get(4)?,time:r.get(5)?,timezone:r.get(6)?,utc:r.get(7)?,status:r.get(8)?})).optional()?.ok_or_else(AppError::not_found)
    }
    pub fn assisted_root(&self) -> Result<Option<String>, AppError> {
        Ok(self.database.connection.query_row("SELECT r.path FROM assisted_settings s LEFT JOIN export_roots r ON r.id=s.root_id WHERE s.id=1",[],|r|r.get(0))?)
    }
    pub(crate) fn assisted_choose_root(&self, path: &Path) -> Result<String, AppError> {
        let canonical = validate_root(path)?;
        let data = self
            .storage
            .directory
            .parent()
            .ok_or_else(AppError::file)?
            .canonicalize()
            .map_err(io_error)?;
        if canonical == data
            || canonical.starts_with(self.storage.directory.canonicalize().map_err(io_error)?)
            || canonical.starts_with(data.join("temp").canonicalize().map_err(io_error)?)
        {
            return Err(error(
                "EXPORT_PATH",
                "Escolha uma pasta separada do banco e das mídias internas.",
            ));
        }
        let text = canonical.to_str().ok_or_else(AppError::file)?;
        if text.len() > 180 {
            return Err(error(
                "EXPORT_PATH",
                "Escolha uma pasta com caminho mais curto (até 180 caracteres).",
            ));
        }
        let tx = self.database.connection.unchecked_transaction()?;
        tx.execute(
            "INSERT OR IGNORE INTO export_roots(id,path) VALUES(?1,?2)",
            params![Uuid::new_v4().to_string(), text],
        )?;
        tx.execute("UPDATE assisted_settings SET root_id=(SELECT id FROM export_roots WHERE path=?1) WHERE id=1",[text])?;
        tx.commit()?;
        Ok(text.into())
    }
    fn root_path(&self, root_id: &str) -> Result<PathBuf, AppError> {
        let text: String = self.database.connection.query_row(
            "SELECT path FROM export_roots WHERE id=?1",
            [root_id],
            |r| r.get(0),
        )?;
        let path = PathBuf::from(text);
        if validate_root(&path)? != path {
            return Err(error(
                "EXPORT_PATH",
                "A raiz de exportação mudou. Escolha novamente a pasta.",
            ));
        }
        Ok(path)
    }
    fn preparation(&self, prep_id: &str) -> Result<Preparation, AppError> {
        id(prep_id)?;
        self.database.connection.query_row("SELECT id,status,fingerprint,root_id,relative_path,publication_version,schedule_version,created_at,error_code FROM preparations WHERE id=?1",[prep_id],|r|Ok(Preparation{id:r.get(0)?,status:r.get(1)?,fingerprint:r.get(2)?,root_id:r.get(3)?,relative_path:r.get(4)?,publication_version:r.get(5)?,schedule_version:r.get(6)?,created_at:r.get(7)?,error_code:r.get(8)?})).optional()?.ok_or_else(AppError::not_found)
    }
    fn latest_preparation(&self, schedule_id: &str) -> Result<Option<Preparation>, AppError> {
        let prep: Option<String> = self
            .database
            .connection
            .query_row(
                "SELECT id FROM preparations WHERE schedule_id=?1 ORDER BY rowid DESC LIMIT 1",
                [schedule_id],
                |r| r.get(0),
            )
            .optional()?;
        prep.map(|x| self.preparation(&x)).transpose()
    }
    fn confirmation(&self, confirmation_id: &str) -> Result<ManualConfirmation, AppError> {
        id(confirmation_id)?;
        self.database.connection.query_row("SELECT id,status,version,preparation_id,planned_date,planned_time,planned_timezone,planned_utc,actual_date,actual_time,actual_timezone,actual_utc,platform,format,reference,note,confirmed_at FROM assisted_confirmations WHERE id=?1",[confirmation_id],|r|Ok(ManualConfirmation{id:r.get(0)?,status:r.get(1)?,version:r.get(2)?,preparation_id:r.get(3)?,planned_date:r.get(4)?,planned_time:r.get(5)?,planned_timezone:r.get(6)?,planned_utc:r.get(7)?,actual_date:r.get(8)?,actual_time:r.get(9)?,actual_timezone:r.get(10)?,actual_utc:r.get(11)?,platform:r.get(12)?,format:r.get(13)?,reference:r.get(14)?,note:r.get(15)?,confirmed_at:r.get(16)?})).optional()?.ok_or_else(AppError::not_found)
    }
    fn active_confirmation(
        &self,
        schedule_id: &str,
    ) -> Result<Option<ManualConfirmation>, AppError> {
        let c:Option<String>=self.database.connection.query_row("SELECT id FROM assisted_confirmations WHERE schedule_id=?1 AND status!='superseded'",[schedule_id],|r|r.get(0)).optional()?;
        c.map(|x| self.confirmation(&x)).transpose()
    }
    fn snapshot(&self, schedule_id: &str) -> Result<(Plan, Publication, String), AppError> {
        let plan = self.assisted_plan(schedule_id)?;
        let reviewed = self.review_item(&plan.publication_id, true)?;
        if plan.status != "planned_local"
            || reviewed.publication.editorial_status != "approved"
            || reviewed.publication.version != plan.publication_version
        {
            return Err(invalid(
                "A publicação precisa estar aprovada e ter planejamento ativo na versão atual.",
            ));
        }
        if reviewed.issues.iter().any(|i| i.severity == "error")
            || !compatible(&reviewed.publication).is_empty()
        {
            return Err(invalid(
                "Corrija a legenda, composição ou mídias antes de preparar.",
            ));
        }
        if local_instant(&plan.date, &plan.time, &plan.timezone)? <= Utc::now() {
            return Err(invalid(
                "O planejamento está no passado. Reagende antes de preparar.",
            ));
        }
        let signature = fingerprint(&plan, &reviewed.publication)?;
        Ok((plan, reviewed.publication, signature))
    }
    fn publication_confirmation(
        &self,
        publication_id: &str,
    ) -> Result<Option<ManualConfirmation>, AppError> {
        let value:Option<String>=self.database.connection.query_row("SELECT id FROM assisted_confirmations WHERE publication_id=?1 AND status!='superseded'",[publication_id],|r|r.get(0)).optional()?;
        value.map(|v| self.confirmation(&v)).transpose()
    }
    fn stale(&self, prep_id: &str) -> Result<(), AppError> {
        self.database.connection.execute("UPDATE preparations SET status='outdated',updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND status IN ('preparing','prepared')",[prep_id])?;
        Ok(())
    }
    fn checked_preparation(
        &self,
        prep_id: &str,
    ) -> Result<(Preparation, Plan, Publication, PathBuf), AppError> {
        let prep = self.preparation(prep_id)?;
        let schedule_id: String = self.database.connection.query_row(
            "SELECT schedule_id FROM preparations WHERE id=?1",
            [prep_id],
            |r| r.get(0),
        )?;
        let result = (|| {
            if prep.status != "prepared" {
                return Err(error(
                    "EXPORT_OUTDATED",
                    "Prepare ou regenere o pacote antes de transferir ou confirmar.",
                ));
            }
            let (plan, p, signature) = self.snapshot(&schedule_id)?;
            if signature != prep.fingerprint
                || prep.publication_version != p.version
                || prep.schedule_version != plan.version
            {
                return Err(error(
                    "EXPORT_OUTDATED",
                    "O conteúdo ou planejamento mudou. Regenere o pacote.",
                ));
            }
            let root = self.root_path(&prep.root_id)?;
            let path = inside(&root, &prep.relative_path)?;
            verify_package(&path, &manifest(&prep, &plan, &p))?;
            Ok((prep.clone(), plan, p, path))
        })();
        if result.is_err() && prep.status == "prepared" {
            self.stale(prep_id)?;
        }
        result
    }
    pub fn assisted_prepare(
        &mut self,
        targets: Vec<PreparationTarget>,
    ) -> Result<Vec<PreparationResult>, AppError> {
        self.assisted_prepare_with(targets, &LocalIo)
    }
    pub(crate) fn assisted_prepare_with(
        &mut self,
        targets: Vec<PreparationTarget>,
        io: &dyn ExportIo,
    ) -> Result<Vec<PreparationResult>, AppError> {
        if targets.is_empty() || targets.len() > 50 {
            return Err(invalid("Selecione de 1 a 50 publicações para preparar."));
        }
        let mut seen = std::collections::HashSet::new();
        for t in &targets {
            id(&t.schedule_id)?;
            if !seen.insert(t.schedule_id.clone()) {
                return Err(invalid("Seleção duplicada."));
            }
        }
        Ok(targets
            .into_iter()
            .map(|t| {
                let schedule_id = t.schedule_id.clone();
                match self.prepare_one(t, io) {
                    Ok(preparation_id) => PreparationResult {
                        schedule_id,
                        preparation_id: Some(preparation_id),
                        error: None,
                    },
                    Err(e) => PreparationResult {
                        schedule_id,
                        preparation_id: None,
                        error: Some(e),
                    },
                }
            })
            .collect())
    }
    fn prepare_one(&mut self, t: PreparationTarget, io: &dyn ExportIo) -> Result<String, AppError> {
        let (plan, p, signature) = self.snapshot(&t.schedule_id)?;
        if t.schedule_version != plan.version || t.publication_version != p.version {
            return Err(AppError::conflict());
        }
        let previous = self.latest_preparation(&plan.id)?;
        if let Some(ref old) = previous {
            if old.status == "prepared" && self.checked_preparation(&old.id).is_ok() {
                return Ok(old.id.clone());
            }
        }
        let root_id: Option<String> = self.database.connection.query_row(
            "SELECT root_id FROM assisted_settings WHERE id=1",
            [],
            |r| r.get(0),
        )?;
        let root_id = root_id
            .ok_or_else(|| error("EXPORT_PATH", "Escolha primeiro a pasta de exportação."))?;
        let root = self.root_path(&root_id)?;
        let prep_id = Uuid::new_v4().to_string();
        let relative = format!(
            "{}/{}_{}_{}_{}",
            &plan.date[..7],
            plan.date,
            plan.time.replace(':', "-"),
            slug(&p.title),
            prep_id
        );
        let tx = self
            .database
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute("INSERT INTO preparations(id,publication_id,schedule_id,publication_version,schedule_version,fingerprint,status,root_id,relative_path,regenerated_from) VALUES(?1,?2,?3,?4,?5,?6,'preparing',?7,?8,?9)",params![prep_id,p.id,plan.id,p.version,plan.version,signature,root_id,relative,previous.as_ref().map(|x|&x.id)])?;
        tx.commit()?;
        let prep = self.preparation(&prep_id)?;
        let result = write_package(
            &root,
            &relative,
            &p,
            &manifest(&prep, &plan, &p),
            |key| self.storage.path(key),
            io,
        );
        match result {
            Ok(_) => {
                if self.database.connection.execute("UPDATE preparations SET status='prepared',updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND status='preparing'",[&prep_id])? != 1 {
                    return Err(AppError::conflict());
                }
                Ok(prep_id)
            }
            Err(e) => {
                self.database.connection.execute("UPDATE preparations SET status='preparation_failed',error_code=?1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?2",params![e.code,prep_id])?;
                Err(e)
            }
        }
    }
    pub(crate) fn assisted_recover(&self) -> Result<(), AppError> {
        let rows = self
            .database
            .connection
            .prepare("SELECT id,root_id,relative_path FROM preparations WHERE status='preparing'")?
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for (prep, root_id, relative) in rows {
            if let Ok(root) = self.root_path(&root_id) {
                if let Ok(path) = inside(&root, &relative) {
                    if let Some(parent) = path.parent() {
                        cleanup(&parent.join(format!(".tmp-{prep}")));
                    }
                }
            }
            self.database.connection.execute("UPDATE preparations SET status='preparation_failed',error_code='EXPORT_INTERRUPTED',updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",[prep])?;
        }
        Ok(())
    }
    pub fn assisted_folder(&self, prep_id: &str) -> Result<String, AppError> {
        let (_, _, _, path) = self.checked_preparation(prep_id)?;
        Ok(path.to_string_lossy().into_owned())
    }
    pub fn assisted_caption(&self, prep_id: &str) -> Result<String, AppError> {
        let (_, _, p, _) = self.checked_preparation(prep_id)?;
        Ok(p.caption)
    }
    pub fn assisted_confirm(&mut self, input: ConfirmationInput) -> Result<String, AppError> {
        if !input.verified_by_operator
            || input.reference.chars().count() > 500
            || input.note.chars().count() > 2000
            || input.reference.contains('\0')
            || input.note.contains('\0')
            || input.previous_id.is_some() != input.expected_version.is_some()
        {
            return Err(invalid("Confirme que verificou o agendamento na Meta e use referência de até 500 e observação de até 2.000 caracteres."));
        }
        if input.reference.contains("://") {
            let url =
                url::Url::parse(&input.reference).map_err(|_| invalid("Referência inválida."))?;
            if url.scheme() != "https"
                || !url.username().is_empty()
                || url.password().is_some()
                || url.query().is_some()
                || url.fragment().is_some()
                || !matches!(
                    url.host_str(),
                    Some(
                        "instagram.com"
                            | "www.instagram.com"
                            | "facebook.com"
                            | "www.facebook.com"
                            | "business.facebook.com"
                    )
                )
            {
                return Err(invalid("Use referência textual ou link HTTPS da Meta sem query, fragmento ou credenciais."));
            }
        }
        let actual = local_instant(&input.actual_date, &input.actual_time, &input.timezone)?
            .to_rfc3339_opts(SecondsFormat::Secs, true);
        let (prep, plan, p, _) = self.checked_preparation(&input.preparation_id)?;
        let current = self.publication_confirmation(&plan.publication_id)?;
        match (&current, &input.previous_id) {
            (Some(c), Some(previous))
                if c.id == *previous && Some(c.version) == input.expected_version => {}
            (None, None) => {}
            _ => return Err(AppError::conflict()),
        }
        let confirmation_id = Uuid::new_v4().to_string();
        let tx = self
            .database
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(c) = &current {
            if tx.execute("UPDATE assisted_confirmations SET status='superseded',version=version+1 WHERE id=?1 AND version=?2 AND status!='superseded'",params![c.id,c.version])?!=1{return Err(AppError::conflict())}
        }
        tx.execute("INSERT INTO assisted_confirmations(id,publication_id,schedule_id,preparation_id,publication_version,schedule_version,fingerprint,planned_date,planned_time,planned_timezone,planned_utc,actual_date,actual_time,actual_timezone,actual_utc,platform,format,reference,note,status) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,'instagram',?16,?17,?18,'confirmed_manually')",params![confirmation_id,p.id,plan.id,prep.id,p.version,plan.version,prep.fingerprint,plan.date,plan.time,plan.timezone,plan.utc,input.actual_date,input.actual_time,input.timezone,actual,p.format,input.reference.trim(),input.note.trim()])?;
        if current.is_some() {
            tx.execute("INSERT INTO assisted_events(publication_id,schedule_id,preparation_id,confirmation_id,event) VALUES(?1,?2,?3,?4,'confirmation_revised')",params![p.id,plan.id,prep.id,confirmation_id])?;
        }
        tx.commit()?;
        Ok(confirmation_id)
    }
    pub fn assisted_reconcile(&mut self, input: ReconcileInput) -> Result<(), AppError> {
        let c = self.confirmation(&input.confirmation_id)?;
        if c.version != input.expected_version || c.status == "superseded" {
            return Err(AppError::conflict());
        }
        let schedule_id: String = self.database.connection.query_row(
            "SELECT schedule_id FROM assisted_confirmations WHERE id=?1",
            [&input.confirmation_id],
            |r| r.get(0),
        )?;
        let plan = self.assisted_plan(&schedule_id)?;
        if plan.version != input.schedule_version || plan.utc == c.actual_utc {
            return Err(AppError::conflict());
        }
        self.schedule_save(ScheduleInput {
            id: Some(schedule_id),
            expected_version: Some(plan.version),
            publication_id: plan.publication_id,
            publication_version: plan.publication_version,
            scheduled_date: c.actual_date,
            scheduled_time: c.actual_time,
            timezone: c.actual_timezone,
        })?;
        Ok(())
    }
    pub fn assisted_list(&self, q: AssistedQuery) -> Result<AssistedPage, AppError> {
        if q.offset > 100000
            || q.state.as_deref().is_some_and(|x| {
                ![
                    "not_prepared",
                    "prepared",
                    "outdated",
                    "preparation_failed",
                    "confirmed_manually",
                    "requires_review",
                    "pending_transfer",
                ]
                .contains(&x)
            })
        {
            return Err(invalid("Filtro inválido."));
        }
        let year = q.calendar.year;
        let month = q.calendar.month;
        let mut cal = q.calendar;
        cal.status = Some("all".into());
        let page = self.calendar_list(cal)?;
        let mut items = Vec::new();
        let mut counts = AssistedCounts::default();
        let mut seen_divergences = std::collections::HashSet::new();
        for schedule in page
            .items
            .into_iter()
            .filter(|s| s.display_date.starts_with(&format!("{year}-{month:02}")))
        {
            let mut prep = self.latest_preparation(&schedule.id)?;
            let active = schedule.status == "planned_local";
            if !active && prep.is_none() && self.active_confirmation(&schedule.id)?.is_none() {
                continue;
            }
            let mut issues = Vec::new();
            let reviewed = self.review_item(&schedule.publication.id, true);
            let p = match reviewed {
                Ok(r) => {
                    issues.extend(
                        r.issues
                            .into_iter()
                            .filter(|i| i.severity == "error")
                            .map(|i| i.message),
                    );
                    Some(r.publication)
                }
                Err(_) => {
                    issues.push("Publicação indisponível ou excluída.".into());
                    None
                }
            };
            if let Some(ref p) = p {
                issues.extend(compatible(p));
                if p.editorial_status != "approved" {
                    issues.push("A publicação precisa de nova aprovação.".into())
                }
            }
            if !active {
                issues.push(
                    "Planejamento cancelado ou invalidado; confira também a Meta manualmente."
                        .into(),
                )
            }
            if schedule.scheduled_at_utc < Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true) {
                issues.push("Planejamento no passado: reagende para preparar novamente.".into())
            }
            if let Some(ref old) = prep {
                if old.status == "prepared" && self.checked_preparation(&old.id).is_err() {
                    prep = Some(self.preparation(&old.id)?);
                }
            }
            let c = if active {
                self.publication_confirmation(&schedule.publication.id)?
            } else {
                self.active_confirmation(&schedule.id)?
            };
            let divergent = c.as_ref().is_some_and(|c| {
                c.status == "requires_review" || c.actual_utc != schedule.scheduled_at_utc
            });
            let status = prep.as_ref().map_or("not_prepared", |p| p.status.as_str());
            let confirmed = c.as_ref().is_some_and(|c| c.status == "confirmed_manually");
            let eligible = issues.is_empty();
            counts.planned += u32::from(active);
            counts.not_prepared += u32::from(active && status == "not_prepared");
            counts.prepared += u32::from(status == "prepared");
            counts.outdated += u32::from(status == "outdated");
            counts.failed += u32::from(status == "preparation_failed");
            counts.confirmed += u32::from(confirmed);
            counts.divergent += u32::from(
                divergent
                    && c.as_ref()
                        .is_some_and(|c| seen_divergences.insert(c.id.clone())),
            );
            counts.pending_transfer += u32::from(active && !confirmed);
            let matches = match q.state.as_deref() {
                None => true,
                Some("confirmed_manually") => confirmed,
                Some("requires_review") => divergent,
                Some("pending_transfer") => active && !confirmed,
                Some(state) => status == state,
            };
            if matches {
                items.push(AssistedItem {
                    warnings: p.as_ref().map_or_else(Vec::new, warnings),
                    publication: p,
                    schedule,
                    preparation: prep,
                    confirmation: c,
                    eligible,
                    issues,
                    divergent,
                });
            }
        }
        let total = items.len();
        let items = items.into_iter().skip(q.offset as usize).take(20).collect();
        Ok(AssistedPage {
            items,
            total,
            counts,
            export_root: self.assisted_root()?,
            boards: page.boards,
        })
    }
    pub fn assisted_history(&self, offset: u32) -> Result<AssistedHistory, AppError> {
        if offset > 100000 {
            return Err(invalid("Página inválida."));
        }
        let total = self.database.connection.query_row(
            "SELECT count(*) FROM assisted_events",
            [],
            |r| r.get(0),
        )?;
        let items=self.database.connection.prepare("SELECT e.id,p.title,e.schedule_id,e.preparation_id,e.confirmation_id,e.event,e.created_at FROM assisted_events e JOIN publications p ON p.id=e.publication_id ORDER BY e.created_at DESC,e.id DESC LIMIT 30 OFFSET ?1")?.query_map([offset],|r|Ok(AssistedEvent{id:r.get(0)?,title:r.get(1)?,schedule_id:r.get(2)?,preparation_id:r.get(3)?,confirmation_id:r.get(4)?,event:r.get(5)?,created_at:r.get(6)?}))?.collect::<Result<_,_>>()?;
        Ok(AssistedHistory { items, total })
    }
}
