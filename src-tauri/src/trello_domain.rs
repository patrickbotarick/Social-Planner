use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use url::Url;

pub fn error(code: &'static str) -> AppError {
    AppError { code, message: match code {
        "TRELLO_AUTH" => "Conecte ou renove a autorização Trello em Configurações.",
        "TRELLO_ACCESS" => "Quadro indisponível ou sem permissão de leitura para esta conta.",
        "TRELLO_URL" => "Informe um link HTTPS de quadro no formato https://trello.com/b/Ab12Cd34/nome.",
        "TRELLO_NETWORK" => "Não foi possível concluir a conexão. Verifique a internet e tente novamente.",
        "TRELLO_RATE" => "Limite temporário do Trello atingido. Aguarde e tente novamente.",
        "TRELLO_CANCELLED" => "Operação cancelada. Os itens concluídos foram preservados.",
        "TRELLO_BUSY" => "Há uma operação Trello em andamento. Aguarde ou cancele a importação.",
        "TRELLO_VAULT" => "Não foi possível acessar o cofre de credenciais do Windows.",
        "TRELLO_LIMIT" => "Limite de segurança atingido. Reduza o tamanho da seleção ou do arquivo.",
        "TRELLO_REDIRECT" => "Redirecionamento de download não autorizado. O anexo foi preservado como referência.",
        "TRELLO_PREVIEW" => "Consulte o quadro novamente antes de importar. A prévia expirou ou mudou.",
        _ => "Resposta Trello incompatível. Tente consultar o quadro novamente.",
    }}
}
pub fn cancelled(flag: &AtomicBool) -> Result<(), AppError> {
    if flag.load(Ordering::Relaxed) {
        Err(error("TRELLO_CANCELLED"))
    } else {
        Ok(())
    }
}
pub fn remote_id(id: &str) -> bool {
    id.len() == 24 && id.bytes().all(|b| b.is_ascii_hexdigit())
}
pub fn board_short_link(input: &str) -> Result<String, AppError> {
    let u = Url::parse(input.trim()).map_err(|_| error("TRELLO_URL"))?;
    let parts: Vec<_> = u.path().split('/').filter(|s| !s.is_empty()).collect();
    if u.scheme() != "https"
        || !matches!(u.host_str(), Some("trello.com" | "www.trello.com"))
        || !u.username().is_empty()
        || u.password().is_some()
        || u.port().is_some()
        || parts.len() < 2
        || parts.len() > 3
        || parts[0] != "b"
        || parts[1].len() != 8
        || !parts[1].bytes().all(|b| b.is_ascii_alphanumeric())
    {
        return Err(error("TRELLO_URL"));
    }
    Ok(parts[1].into())
}
// Referências nunca conservam query/fragmento autenticado nem credenciais embutidas.
pub fn reference_url(input: &str) -> String {
    let Ok(mut u) = Url::parse(input) else {
        return String::new();
    };
    if !matches!(u.scheme(), "http" | "https") || !u.username().is_empty() || u.password().is_some()
    {
        return String::new();
    }
    u.set_query(None);
    u.set_fragment(None);
    u.to_string()
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub id: String,
    pub name: String,
    pub short_link: String,
}
impl Board {
    pub fn url(&self) -> String {
        format!("https://trello.com/b/{}", self.short_link)
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct TrelloList {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub closed: bool,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: String,
    pub name: String,
    pub url: String,
    pub mime_type: Option<String>,
    #[serde(default)]
    pub bytes: u64,
    #[serde(default)]
    pub is_upload: bool,
    #[serde(default)]
    pub pos: f64,
}
impl Attachment {
    pub fn extension(&self) -> Option<&'static str> {
        let e = self.name.rsplit('.').next()?.to_ascii_lowercase();
        match e.as_str() {
            "jpg" | "jpeg" => Some("jpg"),
            "png" => Some("png"),
            "webp" => Some("webp"),
            "mp4" => Some("mp4"),
            _ => None,
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteCard {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub id_list: String,
    pub short_link: String,
    pub closed: bool,
    pub pos: f64,
    pub date_last_activity: Option<String>,
    pub due: Option<String>,
    pub start: Option<String>,
    #[serde(default)]
    pub labels: Vec<Label>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}
impl RemoteCard {
    pub fn hash(&self) -> String {
        format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(self).expect("typed snapshot"))
        )
    }
    pub fn url(&self) -> String {
        format!("https://trello.com/c/{}", self.short_link)
    }
    pub fn sanitize(&mut self) -> Result<(), AppError> {
        if !remote_id(&self.id)
            || !remote_id(&self.id_list)
            || self.short_link.len() != 8
            || !self.short_link.bytes().all(|b| b.is_ascii_alphanumeric())
            || self.attachments.len() > 1000
        {
            return Err(error("TRELLO_RESPONSE"));
        }
        for a in &mut self.attachments {
            if !remote_id(&a.id) {
                return Err(error("TRELLO_RESPONSE"));
            }
            a.url = reference_url(&a.url);
        }
        self.attachments
            .sort_by(|a, b| a.pos.total_cmp(&b.pos).then(a.id.cmp(&b.id)));
        self.labels.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(())
    }
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewCard {
    pub card: RemoteCard,
    pub state: String,
    pub publication_id: Option<String>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardPreview {
    pub preview_id: String,
    pub board: Board,
    pub lists: Vec<TrelloList>,
    pub cards: Vec<PreviewCard>,
    pub last_synced_at: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DownloadLimits {
    pub image_mib: u64,
    pub video_mib: u64,
    pub timeout_seconds: u64,
}
impl Default for DownloadLimits {
    fn default() -> Self {
        Self {
            image_mib: 64,
            video_mib: 512,
            timeout_seconds: 120,
        }
    }
}
impl DownloadLimits {
    pub fn validate(&self) -> Result<(), AppError> {
        if !(1..=64).contains(&self.image_mib)
            || !(1..=512).contains(&self.video_mib)
            || !(10..=300).contains(&self.timeout_seconds)
        {
            Err(error("TRELLO_LIMIT"))
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub status: String,
    pub total_cards: u32,
    pub processed_cards: u32,
    pub successful_cards: u32,
    pub failed_cards: u32,
    pub new_publications: u32,
    pub changed_sources: u32,
    pub downloaded_attachments: u32,
    pub attachment_failures: u32,
    pub error_message: Option<String>,
    pub current_card: Option<String>,
    pub downloaded_bytes: u64,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardHistory {
    pub name: String,
    pub url: String,
    pub last_synced_at: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    pub url: String,
    pub original_title: String,
    pub original_description: String,
    pub source_changed: bool,
    pub source_missing: bool,
    pub needs_adjustment: bool,
    pub attachments: Vec<SourceAttachment>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceAttachment {
    pub name: String,
    pub url: String,
    pub status: String,
    pub error_message: Option<String>,
    pub removed: bool,
    pub media: Option<crate::editorial_domain::MediaAsset>,
}
