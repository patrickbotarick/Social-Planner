use crate::{
    editorial_domain::MediaAsset,
    editorial_repository::{asset, normalized, with_preview, EditorialRepository},
    error::AppError,
    review::ReviewOption,
};
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, SecondsFormat, TimeZone, Utc};
use chrono_tz::Tz;
use rusqlite::{params, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn invalid(message: &'static str) -> AppError {
    AppError {
        code: "CALENDAR_INVALID",
        message,
    }
}
fn valid_id(id: &str) -> Result<(), AppError> {
    Uuid::parse_str(id)
        .map(|_| ())
        .map_err(|_| AppError::validation())
}
pub(crate) fn local_instant(
    date: &str,
    time: &str,
    timezone: &str,
) -> Result<DateTime<Utc>, AppError> {
    let date_value = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| invalid("Informe uma data válida."))?;
    let time_value = NaiveTime::parse_from_str(time, "%H:%M")
        .map_err(|_| invalid("Informe um horário válido de 24 horas."))?;
    if date_value.format("%Y-%m-%d").to_string() != date
        || time_value.format("%H:%M").to_string() != time
        || !(1900..=9998).contains(&date_value.year())
    {
        return Err(AppError::validation());
    }
    let tz: Tz = timezone
        .parse()
        .map_err(|_| invalid("Fuso IANA inválido."))?;
    tz.from_local_datetime(&date_value.and_time(time_value))
        .single()
        .map(|d| d.with_timezone(&Utc))
        .ok_or_else(|| {
            invalid("Esse horário é ambíguo ou não existe nesse fuso. Escolha outro horário.")
        })
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CalendarQuery {
    pub year: i32,
    pub month: u32,
    pub timezone: String,
    #[serde(default)]
    pub search: String,
    pub format: Option<String>,
    pub source: Option<String>,
    pub board_id: Option<String>,
    pub status: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AvailableQuery {
    #[serde(default)]
    pub search: String,
    pub format: Option<String>,
    pub source: Option<String>,
    pub board_id: Option<String>,
    pub media_type: Option<String>,
    #[serde(default)]
    pub offset: u32,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarPublication {
    pub id: String,
    pub title: String,
    pub format: String,
    pub source: String,
    pub version: u32,
    pub primary_media: Option<MediaAsset>,
    pub board_name: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    pub id: String,
    pub publication: CalendarPublication,
    pub publication_version: u32,
    pub scheduled_date: String,
    pub scheduled_time: String,
    pub timezone: String,
    pub scheduled_at_utc: String,
    pub display_date: String,
    pub display_time: String,
    pub status: String,
    pub version: u32,
    pub created_at: String,
    pub updated_at: String,
    pub cancelled_at: Option<String>,
    pub conflict: Option<&'static str>,
    pub media_unavailable: bool,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleSettings {
    pub proximity_minutes: u32,
    pub time_step_minutes: u32,
    pub version: u32,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsInput {
    pub proximity_minutes: u32,
    pub time_step_minutes: u32,
    pub expected_version: u32,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScheduleInput {
    pub id: Option<String>,
    pub expected_version: Option<u32>,
    pub publication_id: String,
    pub publication_version: u32,
    pub scheduled_date: String,
    pub scheduled_time: String,
    pub timezone: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarPage {
    pub items: Vec<Schedule>,
    pub total: u32,
    pub empty_days: u32,
    pub conflicts: u32,
    pub formats: std::collections::BTreeMap<String, u32>,
    pub boards: Vec<ReviewOption>,
    pub settings: ScheduleSettings,
}
#[derive(Serialize)]
pub struct AvailablePage {
    pub items: Vec<CalendarPublication>,
    pub total: u32,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningDashboard {
    pub available: u32,
    pub planned_this_month: u32,
    pub conflicts: u32,
    pub upcoming: Vec<Schedule>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningEvent {
    pub id: i64,
    pub schedule_id: String,
    pub publication_id: String,
    pub title: String,
    pub event: String,
    pub previous_date: Option<String>,
    pub previous_time: Option<String>,
    pub previous_timezone: Option<String>,
    pub scheduled_date: String,
    pub scheduled_time: String,
    pub timezone: String,
    pub schedule_version: u32,
    pub publication_version: u32,
    pub created_at: String,
}
#[derive(Serialize)]
pub struct PlanningHistory {
    pub items: Vec<PlanningEvent>,
    pub total: u32,
}

fn validate_filters(
    search: &str,
    format: &Option<String>,
    source: &Option<String>,
    board: &Option<String>,
) -> Result<(), AppError> {
    if search.chars().count() > 200
        || format
            .as_deref()
            .is_some_and(|s| !["feed", "carousel", "reels", "stories"].contains(&s))
        || source
            .as_deref()
            .is_some_and(|s| !["manual", "trello"].contains(&s))
    {
        return Err(AppError::validation());
    }
    if let Some(id) = board {
        valid_id(id)?;
    }
    Ok(())
}
impl EditorialRepository {
    fn calendar_publication(&self, id: &str) -> Result<CalendarPublication, AppError> {
        let mut p=self.database.connection.query_row("SELECT p.id,p.title,p.format,p.source,p.version,b.name FROM publications p LEFT JOIN imported_cards c ON c.publication_id=p.id LEFT JOIN trello_boards b ON b.id=c.board_id WHERE p.id=?1",[id],|r|Ok(CalendarPublication{id:r.get(0)?,title:r.get(1)?,format:r.get(2)?,source:r.get(3)?,version:r.get(4)?,board_name:r.get(5)?,primary_media:None}))?;
        let media:Option<String>=self.database.connection.query_row("SELECT media_asset_id FROM publication_media WHERE publication_id=?1 ORDER BY sort_order LIMIT 1",[id],|r|r.get(0)).optional()?;
        p.primary_media = media
            .map(|id| asset(&self.database.connection, &id).map(|m| with_preview(m, &self.storage)))
            .transpose()?;
        Ok(p)
    }
    pub fn schedule_settings(&self) -> Result<ScheduleSettings, AppError> {
        Ok(self.database.connection.query_row(
            "SELECT proximity_minutes,time_step_minutes,version FROM schedule_settings WHERE id=1",
            [],
            |r| {
                Ok(ScheduleSettings {
                    proximity_minutes: r.get(0)?,
                    time_step_minutes: r.get(1)?,
                    version: r.get(2)?,
                })
            },
        )?)
    }
    pub fn schedule_settings_save(&self, input: SettingsInput) -> Result<(), AppError> {
        if input.proximity_minutes > 180 || ![5, 10, 15, 30, 60].contains(&input.time_step_minutes)
        {
            return Err(AppError::validation());
        }
        if self.database.connection.execute("UPDATE schedule_settings SET proximity_minutes=?1,time_step_minutes=?2,version=version+1 WHERE id=1 AND version=?3",params![input.proximity_minutes,input.time_step_minutes,input.expected_version])?!=1 {return Err(AppError::conflict());}
        Ok(())
    }
    pub fn calendar_available(&self, q: AvailableQuery) -> Result<AvailablePage, AppError> {
        validate_filters(&q.search, &q.format, &q.source, &q.board_id)?;
        if q.offset > 100000
            || q.media_type
                .as_deref()
                .is_some_and(|s| !["image", "video"].contains(&s))
        {
            return Err(AppError::validation());
        }
        let clause="p.deleted_at IS NULL AND p.editorial_status='approved' AND NOT EXISTS(SELECT 1 FROM schedules s WHERE s.publication_id=p.id AND s.status='planned_local') AND (?1='' OR instr(p.search_text,?1)>0) AND (?2 IS NULL OR p.format=?2) AND (?3 IS NULL OR p.source=?3) AND (?4 IS NULL OR c.board_id=?4) AND (?5 IS NULL OR EXISTS(SELECT 1 FROM publication_media pm JOIN media_assets m ON m.id=pm.media_asset_id WHERE pm.publication_id=p.id AND ((?5='image' AND m.mime_type LIKE 'image/%') OR (?5='video' AND m.mime_type='video/mp4'))))";
        let search = normalized(q.search.trim());
        let args = params![search, q.format, q.source, q.board_id, q.media_type];
        let total=self.database.connection.query_row(&format!("SELECT COUNT(*) FROM publications p LEFT JOIN imported_cards c ON c.publication_id=p.id WHERE {clause}"),args,|r|r.get(0))?;
        let ids=self.database.connection.prepare(&format!("SELECT p.id FROM publications p LEFT JOIN imported_cards c ON c.publication_id=p.id WHERE {clause} ORDER BY p.created_at,p.id LIMIT 20 OFFSET ?6"))?.query_map(params![search,q.format,q.source,q.board_id,q.media_type,q.offset],|r|r.get::<_,String>(0))?.collect::<Result<Vec<_>,_>>()?;
        Ok(AvailablePage {
            items: ids
                .iter()
                .map(|id| self.calendar_publication(id))
                .collect::<Result<_, _>>()?,
            total,
        })
    }
    pub fn schedule_save(&mut self, input: ScheduleInput) -> Result<String, AppError> {
        valid_id(&input.publication_id)?;
        if input.id.is_some() != input.expected_version.is_some() {
            return Err(AppError::validation());
        }
        if let Some(id) = &input.id {
            valid_id(id)?;
        }
        let instant = local_instant(
            &input.scheduled_date,
            &input.scheduled_time,
            &input.timezone,
        )?;
        if instant <= Utc::now() {
            return Err(invalid(
                "Novos planejamentos e alterações devem usar data e horário futuros.",
            ));
        }
        let p = self.review_item(&input.publication_id, true)?;
        if p.publication.editorial_status != "approved"
            || p.publication.version != input.publication_version
        {
            return Err(invalid(
                "A publicação precisa estar aprovada na versão atual. Atualize a fila.",
            ));
        }
        if p.issues.iter().any(|i| i.severity == "error") {
            return Err(invalid("Corrija as mídias e o formato antes de planejar."));
        }
        let tx = self
            .database
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let duplicate:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM schedules WHERE publication_id=?1 AND status='planned_local' AND (?2 IS NULL OR id!=?2))",params![input.publication_id,input.id],|r|r.get(0))?;
        if duplicate {
            return Err(invalid("Esta publicação já possui planejamento ativo."));
        }
        let utc = instant.to_rfc3339_opts(SecondsFormat::Secs, true);
        let id = if let Some(id) = input.id {
            let n=tx.execute("UPDATE schedules SET scheduled_date=?1,scheduled_time=?2,timezone=?3,scheduled_at_utc=?4,version=version+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?5 AND publication_id=?6 AND publication_version=?7 AND version=?8 AND status='planned_local'",params![input.scheduled_date,input.scheduled_time,input.timezone,utc,id,input.publication_id,input.publication_version,input.expected_version])?;
            if n != 1 {
                return Err(AppError::conflict());
            }
            id
        } else {
            let id = Uuid::new_v4().to_string();
            tx.execute("INSERT INTO schedules(id,publication_id,publication_version,scheduled_date,scheduled_time,timezone,scheduled_at_utc,status) VALUES(?1,?2,?3,?4,?5,?6,?7,'planned_local')",params![id,input.publication_id,input.publication_version,input.scheduled_date,input.scheduled_time,input.timezone,utc])?;
            id
        };
        tx.commit()?;
        Ok(id)
    }
    pub fn schedule_cancel(&mut self, id: &str, version: u32) -> Result<(), AppError> {
        valid_id(id)?;
        let tx = self
            .database
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if tx.execute("UPDATE schedules SET status='cancelled_local',cancelled_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),version=version+1 WHERE id=?1 AND version=?2 AND status='planned_local'",params![id,version])?!=1{return Err(AppError::conflict());}
        tx.commit()?;
        Ok(())
    }
    pub fn calendar_list(&self, q: CalendarQuery) -> Result<CalendarPage, AppError> {
        validate_filters(&q.search, &q.format, &q.source, &q.board_id)?;
        let first = NaiveDate::from_ymd_opt(q.year, q.month, 1)
            .filter(|_| (1900..=9998).contains(&q.year))
            .ok_or_else(AppError::validation)?;
        let next = if q.month == 12 {
            NaiveDate::from_ymd_opt(q.year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(q.year, q.month + 1, 1)
        }
        .ok_or_else(AppError::validation)?;
        let start = first - Duration::days(first.weekday().num_days_from_sunday().into());
        let end = start + Duration::days(42);
        let tz: Tz = q
            .timezone
            .parse()
            .map_err(|_| invalid("Fuso IANA inválido."))?;
        if q.status.as_deref().is_some_and(|s| {
            ![
                "planned_local",
                "cancelled_local",
                "invalidated_local",
                "all",
            ]
            .contains(&s)
        }) {
            return Err(AppError::validation());
        }
        let settings = self.schedule_settings()?;
        // UTC em janela ampliada: inclui conflitos próximos fora da grade e evita meia-noite ambígua em limites de DST.
        let lower = (start - Duration::days(2))
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .to_rfc3339_opts(SecondsFormat::Secs, true);
        let upper = (end + Duration::days(2))
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .to_rfc3339_opts(SecondsFormat::Secs, true);
        let rows=self.database.connection.prepare("SELECT s.id,s.publication_id,s.publication_version,s.scheduled_date,s.scheduled_time,s.timezone,s.scheduled_at_utc,s.status,s.version,s.created_at,s.updated_at,s.cancelled_at,p.format,p.source,COALESCE(c.board_id,''),p.search_text FROM schedules s JOIN publications p ON p.id=s.publication_id LEFT JOIN imported_cards c ON c.publication_id=p.id WHERE s.scheduled_at_utc>=?1 AND s.scheduled_at_utc<?2 ORDER BY s.scheduled_at_utc,s.id LIMIT 10001")?.query_map(params![lower,upper],|r|Ok((Schedule{id:r.get(0)?,publication:self.calendar_publication(&r.get::<_,String>(1)?).map_err(|_|rusqlite::Error::InvalidQuery)?,publication_version:r.get(2)?,scheduled_date:r.get(3)?,scheduled_time:r.get(4)?,timezone:r.get(5)?,scheduled_at_utc:r.get(6)?,display_date:String::new(),display_time:String::new(),status:r.get(7)?,version:r.get(8)?,created_at:r.get(9)?,updated_at:r.get(10)?,cancelled_at:r.get(11)?,conflict:None,media_unavailable:false},r.get::<_,String>(12)?,r.get::<_,String>(13)?,r.get::<_,String>(14)?,r.get::<_,String>(15)?)))?.collect::<Result<Vec<_>,_>>()?;
        if rows.len() > 10000 {
            return Err(invalid(
                "O período excede 10.000 planejamentos. Consulte um período menor.",
            ));
        }
        let instants = rows
            .iter()
            .filter(|(s, ..)| s.status == "planned_local")
            .map(|(s, ..)| {
                Ok((
                    s.id.clone(),
                    DateTime::parse_from_rfc3339(&s.scheduled_at_utc)
                        .map_err(|_| AppError::database())?
                        .timestamp(),
                ))
            })
            .collect::<Result<Vec<_>, AppError>>()?;
        let mut items = Vec::new();
        let mut total = 0;
        let mut days = std::collections::HashSet::new();
        let mut conflicts = 0;
        let mut formats = std::collections::BTreeMap::new();
        for (mut s, format, source, board, search) in rows {
            let instant = DateTime::parse_from_rfc3339(&s.scheduled_at_utc)
                .map_err(|_| AppError::database())?;
            let local = instant.with_timezone(&tz);
            let date = local.date_naive();
            if date < start || date >= end {
                continue;
            }
            s.display_date = local.format("%Y-%m-%d").to_string();
            s.display_time = local.format("%H:%M").to_string();
            if s.status == "planned_local" {
                let closest = instants
                    .iter()
                    .filter(|(id, _)| *id != s.id)
                    .map(|(_, n)| (n - instant.timestamp()).abs())
                    .min();
                s.conflict = closest.and_then(|seconds| {
                    if seconds == 0 {
                        Some("exact")
                    } else if settings.proximity_minutes > 0
                        && seconds < i64::from(settings.proximity_minutes) * 60
                    {
                        Some("nearby")
                    } else {
                        None
                    }
                });
                s.media_unavailable = self
                    .review_item(&s.publication.id, false)
                    .map(|p| p.issues.iter().any(|i| i.severity == "error"))
                    .unwrap_or(true);
            }
            if q.format.as_ref().is_some_and(|v| *v != format)
                || q.source.as_ref().is_some_and(|v| *v != source)
                || q.board_id.as_ref().is_some_and(|v| *v != board)
                || !search.contains(&normalized(q.search.trim()))
                || !match q.status.as_deref() {
                    None => s.status != "cancelled_local",
                    Some("all") => true,
                    Some(status) => s.status == status,
                }
            {
                continue;
            }
            if date >= first && date < next && s.status == "planned_local" {
                total += 1;
                days.insert(date);
                *formats.entry(format).or_insert(0) += 1;
                if s.conflict.is_some() {
                    conflicts += 1;
                }
            }
            items.push(s);
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
        Ok(CalendarPage {
            items,
            total,
            empty_days: (next - first).num_days() as u32 - days.len() as u32,
            conflicts,
            formats,
            boards,
            settings,
        })
    }
    pub fn planning_dashboard(&self, timezone: &str) -> Result<PlanningDashboard, AppError> {
        let tz: Tz = timezone.parse().map_err(|_| AppError::validation())?;
        let now = Utc::now();
        let local = now.with_timezone(&tz);
        let month = self.calendar_list(CalendarQuery {
            year: local.year(),
            month: local.month(),
            timezone: timezone.into(),
            ..Default::default()
        })?;
        let ids=self.database.connection.prepare("SELECT id FROM schedules WHERE status='planned_local' AND scheduled_at_utc>=?1 ORDER BY scheduled_at_utc,id LIMIT 6")?.query_map([now.to_rfc3339_opts(SecondsFormat::Secs,true)],|r|r.get::<_,String>(0))?.collect::<Result<Vec<_>,_>>()?;
        // Próximos seis podem atravessar o fim do mês.
        let mut upcoming = Vec::new();
        for id in ids {
            let instant: String = self.database.connection.query_row(
                "SELECT scheduled_at_utc FROM schedules WHERE id=?1",
                [&id],
                |r| r.get(0),
            )?;
            let date = DateTime::parse_from_rfc3339(&instant)
                .map_err(|_| AppError::database())?
                .with_timezone(&tz);
            let page = self.calendar_list(CalendarQuery {
                year: date.year(),
                month: date.month(),
                timezone: timezone.into(),
                ..Default::default()
            })?;
            if let Some(s) = page.items.into_iter().find(|s| s.id == id) {
                upcoming.push(s);
            }
        }
        let available = self
            .calendar_available(AvailableQuery {
                search: String::new(),
                format: None,
                source: None,
                board_id: None,
                media_type: None,
                offset: 0,
            })?
            .total;
        Ok(PlanningDashboard {
            available,
            planned_this_month: month.total,
            conflicts: month.conflicts,
            upcoming,
        })
    }
    pub fn schedule_history(&self, offset: u32) -> Result<PlanningHistory, AppError> {
        if offset > 100000 {
            return Err(AppError::validation());
        }
        let total = self.database.connection.query_row(
            "SELECT count(*) FROM schedule_events",
            [],
            |r| r.get(0),
        )?;
        let items=self.database.connection.prepare("SELECT e.id,e.schedule_id,s.publication_id,p.title,e.event,e.previous_date,e.previous_time,e.previous_timezone,e.scheduled_date,e.scheduled_time,e.timezone,e.schedule_version,e.publication_version,e.created_at FROM schedule_events e JOIN schedules s ON s.id=e.schedule_id JOIN publications p ON p.id=s.publication_id ORDER BY e.created_at DESC,e.id DESC LIMIT 30 OFFSET ?1")?.query_map([offset],|r|Ok(PlanningEvent{id:r.get(0)?,schedule_id:r.get(1)?,publication_id:r.get(2)?,title:r.get(3)?,event:r.get(4)?,previous_date:r.get(5)?,previous_time:r.get(6)?,previous_timezone:r.get(7)?,scheduled_date:r.get(8)?,scheduled_time:r.get(9)?,timezone:r.get(10)?,schedule_version:r.get(11)?,publication_version:r.get(12)?,created_at:r.get(13)?}))?.collect::<Result<Vec<_>,_>>()?;
        Ok(PlanningHistory { items, total })
    }
}
