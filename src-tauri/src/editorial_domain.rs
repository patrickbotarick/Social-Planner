use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PublicationFormat {
    Feed,
    Carousel,
    Reels,
    Stories,
}
impl PublicationFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Feed => "feed",
            Self::Carousel => "carousel",
            Self::Reels => "reels",
            Self::Stories => "stories",
        }
    }
    pub fn accepts(self, mime: &str) -> bool {
        match self {
            Self::Feed => mime.starts_with("image/"),
            Self::Reels => mime == "video/mp4",
            _ => true,
        }
    }
    pub fn validate(self, media: &[MediaAsset]) -> Result<(), AppError> {
        let quantity = match self {
            Self::Carousel => (2..=20).contains(&media.len()),
            _ => media.len() == 1,
        };
        let mut hashes = std::collections::HashSet::new();
        if !quantity
            || media
                .iter()
                .any(|m| !self.accepts(&m.mime_type) || !hashes.insert(&m.checksum))
        {
            return Err(AppError::validation());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavePublication {
    pub id: Option<String>,
    pub expected_version: Option<u32>,
    pub title: String,
    pub caption: String,
    pub format: PublicationFormat,
    pub media_ids: Vec<String>,
}
impl SavePublication {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.title.trim().is_empty()
            || self.title.chars().count() > 200
            || self.caption.chars().count() > 20000
            || self.title.contains('\0')
            || self.caption.contains('\0')
            || self.media_ids.len() > 20
            || self
                .media_ids
                .iter()
                .any(|id| uuid::Uuid::parse_str(id).is_err())
            || self
                .id
                .as_ref()
                .is_some_and(|id| uuid::Uuid::parse_str(id).is_err())
            || self.id.is_some() != self.expected_version.is_some()
        {
            return Err(AppError::validation());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaAsset {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub extension: String,
    pub size_bytes: u64,
    pub storage_path: String,
    pub preview_path: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration: Option<f64>,
    pub checksum: String,
    pub created_at: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Publication {
    pub id: String,
    pub title: String,
    pub caption: String,
    pub format: String,
    pub source: String,
    pub editorial_status: String,
    pub version: u32,
    pub created_at: String,
    pub updated_at: String,
    pub media: Vec<MediaAsset>,
}
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LibraryQuery {
    #[serde(default)]
    pub search: String,
    pub format: Option<PublicationFormat>,
    pub source: Option<String>,
    pub status: Option<String>,
    #[serde(default)]
    pub sort: LibrarySort,
    #[serde(default)]
    pub offset: u32,
    #[serde(default = "page_size")]
    pub limit: u32,
}
fn page_size() -> u32 {
    24
}
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LibrarySort {
    #[default]
    Recent,
    Oldest,
    Title,
    Updated,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicationSummary {
    pub id: String,
    pub title: String,
    pub format: String,
    pub source: String,
    pub editorial_status: String,
    pub created_at: String,
    pub updated_at: String,
    pub media_count: u32,
    pub primary_media: Option<MediaAsset>,
}
#[derive(Debug, Serialize)]
pub struct LibraryPage {
    pub items: Vec<PublicationSummary>,
    pub total: u32,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorialCounts {
    pub total: u32,
    pub drafts: u32,
    pub pending_review: u32,
    pub approved: u32,
    pub ignored: u32,
}
#[derive(Debug, Serialize)]
pub struct ImportIssue {
    pub filename: String,
    pub message: &'static str,
}
#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub media: Vec<MediaAsset>,
    pub issues: Vec<ImportIssue>,
}
