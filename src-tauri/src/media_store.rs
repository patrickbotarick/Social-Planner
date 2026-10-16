use crate::{editorial_domain::MediaAsset, error::AppError};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{BufReader, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub struct MediaStore {
    pub directory: PathBuf,
    pub staging_directory: PathBuf,
    pub staged: HashMap<String, MediaAsset>,
}
impl MediaStore {
    pub fn open(data: &Path) -> Result<Self, AppError> {
        let directory = data.join("media");
        let staging_directory = data
            .join("temp")
            .join("imports")
            .join(Uuid::new_v4().to_string());
        fs::create_dir_all(&directory).map_err(|_| AppError::file())?;
        fs::create_dir_all(&staging_directory).map_err(|_| AppError::file())?;
        let base = data.canonicalize().map_err(|_| AppError::file())?;
        if !directory
            .canonicalize()
            .map_err(|_| AppError::file())?
            .starts_with(&base)
            || !staging_directory
                .canonicalize()
                .map_err(|_| AppError::file())?
                .starts_with(&base)
        {
            return Err(AppError::file());
        }
        Ok(Self {
            directory,
            staging_directory,
            staged: HashMap::new(),
        })
    }

    // Caminho de origem vem exclusivamente do seletor nativo Rust, nunca do IPC.
    pub fn stage(&mut self, source: &Path) -> Result<MediaAsset, AppError> {
        let extension = source
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let canonical_extension = match extension.as_str() {
            "jpg" | "jpeg" => "jpg",
            "png" => "png",
            "webp" => "webp",
            "mp4" => "mp4",
            _ => return Err(AppError::media()),
        };
        let id = Uuid::new_v4().to_string();
        let partial = self.staging_directory.join(format!("{id}.partial"));
        let finished = self
            .staging_directory
            .join(format!("{id}.{canonical_extension}"));
        let result = (|| {
            let mut input = File::open(source).map_err(|_| AppError::file())?;
            let metadata = input.metadata().map_err(|_| AppError::file())?;
            let maximum = if canonical_extension == "mp4" {
                512 * 1024 * 1024
            } else {
                64 * 1024 * 1024
            };
            if !metadata.is_file() || metadata.len() == 0 || metadata.len() > maximum {
                return Err(AppError::media());
            }
            let mut output = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&partial)
                .map_err(|_| AppError::file())?;
            let mut digest = Sha256::new();
            let mut buffer = [0u8; 64 * 1024];
            let mut size = 0u64;
            loop {
                let count = input.read(&mut buffer).map_err(|_| AppError::file())?;
                if count == 0 {
                    break;
                }
                size += count as u64;
                if size > maximum {
                    return Err(AppError::media());
                }
                output
                    .write_all(&buffer[..count])
                    .map_err(|_| AppError::file())?;
                digest.update(&buffer[..count]);
            }
            output.sync_all().map_err(|_| AppError::file())?;
            drop(output);
            if size != metadata.len() {
                return Err(AppError::file());
            }
            let (mime, width, height, duration) = inspect(&partial, canonical_extension, size)?;
            fs::rename(&partial, &finished).map_err(|_| AppError::file())?;
            Ok(MediaAsset {
                id: id.clone(),
                filename: source
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                mime_type: mime.into(),
                extension: canonical_extension.into(),
                size_bytes: size,
                storage_path: format!("{id}.{canonical_extension}"),
                preview_path: Some(finished.to_string_lossy().into_owned()),
                width,
                height,
                duration,
                checksum: format!("{:x}", digest.finalize()),
                created_at: String::new(),
            })
        })();
        match result {
            Ok(media) => {
                self.staged.insert(id, media.clone());
                Ok(media)
            }
            Err(error) => {
                let _ = fs::remove_file(&partial);
                let _ = fs::remove_file(&finished);
                Err(error)
            }
        }
    }

    pub fn discard(&mut self, ids: &[String]) -> Result<(), AppError> {
        for id in ids {
            if let Some(media) = self.staged.get(id) {
                let path = self.staging_directory.join(&media.storage_path);
                match fs::remove_file(path) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(_) => return Err(AppError::file()),
                }
                self.staged.remove(id);
            }
        }
        Ok(())
    }

    pub fn path(&self, relative: &str) -> Result<PathBuf, AppError> {
        let path = Path::new(relative);
        if path.components().count() != 1
            || !matches!(
                path.components().next(),
                Some(std::path::Component::Normal(_))
            )
        {
            return Err(AppError::file());
        }
        let full = self
            .directory
            .join(path)
            .canonicalize()
            .map_err(|_| AppError::file())?;
        if !full.starts_with(
            self.directory
                .canonicalize()
                .map_err(|_| AppError::file())?,
        ) || !full.is_file()
        {
            return Err(AppError::file());
        }
        Ok(full)
    }

    // Criação atômica sem sobrescrever. A cópia validada já está no mesmo volume.
    pub fn install(&self, media: &MediaAsset) -> Result<(String, bool), AppError> {
        let staged = self.staging_directory.join(&media.storage_path);
        if checksum(&staged)? != media.checksum {
            return Err(AppError::file());
        }
        let relative = format!("{}.{}", media.checksum, media.extension);
        let destination = self.directory.join(&relative);
        match fs::hard_link(&staged, &destination) {
            Ok(()) => Ok((relative, true)),
            Err(_) if destination.exists() => {
                if checksum(&self.path(&relative)?)? == media.checksum {
                    Ok((relative, false))
                } else {
                    Err(AppError::file())
                }
            }
            Err(_) => {
                // Fallback para volumes sem hard links. Concluir antes de promover o arquivo.
                let partial = self.directory.join(format!("{}.partial", Uuid::new_v4()));
                let result = (|| -> Result<(), AppError> {
                    let mut source = File::open(&staged).map_err(|_| AppError::file())?;
                    let mut target = OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&partial)
                        .map_err(|_| AppError::file())?;
                    std::io::copy(&mut source, &mut target).map_err(|_| AppError::file())?;
                    target.sync_all().map_err(|_| AppError::file())?;
                    drop(target);
                    if checksum(&partial)? != media.checksum {
                        return Err(AppError::file());
                    }
                    fs::rename(&partial, &destination).map_err(|_| AppError::file())?;
                    Ok(())
                })();
                if result.is_err() {
                    let _ = fs::remove_file(&partial);
                }
                result.map(|()| (relative, true))
            }
        }
    }
}
impl Drop for MediaStore {
    fn drop(&mut self) {
        let ids = self.staged.keys().cloned().collect::<Vec<_>>();
        let _ = self.discard(&ids);
        let _ = fs::remove_dir(&self.staging_directory);
    }
}
pub fn checksum(path: &Path) -> Result<String, AppError> {
    let mut file = File::open(path).map_err(|_| AppError::file())?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(|_| AppError::file())?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}
pub(crate) fn verify_asset(path: &Path, media: &MediaAsset) -> Result<(), AppError> {
    let maximum = if media.extension == "mp4" {
        512 * 1024 * 1024
    } else {
        64 * 1024 * 1024
    };
    if media.size_bytes == 0 || media.size_bytes > maximum || checksum(path)? != media.checksum {
        return Err(AppError::media());
    }
    let (mime, width, height, duration) = inspect(path, &media.extension, media.size_bytes)?;
    if mime != media.mime_type
        || width != media.width
        || height != media.height
        || duration != media.duration
    {
        return Err(AppError::media());
    }
    Ok(())
}
type Inspection = (&'static str, Option<u32>, Option<u32>, Option<f64>);
fn inspect(path: &Path, extension: &str, size: u64) -> Result<Inspection, AppError> {
    if extension == "mp4" {
        validate_mp4_boxes(path, size)?;
        let reader = mp4::Mp4Reader::read_header(
            BufReader::new(File::open(path).map_err(|_| AppError::file())?),
            size,
        )
        .map_err(|_| AppError::media())?;
        let track = reader
            .tracks()
            .values()
            .find(|t| t.track_type().ok() == Some(mp4::TrackType::Video))
            .ok_or_else(AppError::media)?;
        if track.width() == 0 || track.height() == 0 {
            return Err(AppError::media());
        }
        return Ok((
            "video/mp4",
            Some(track.width().into()),
            Some(track.height().into()),
            Some(reader.duration().as_secs_f64()),
        ));
    }
    let format = match extension {
        "jpg" => image::ImageFormat::Jpeg,
        "png" => image::ImageFormat::Png,
        _ => image::ImageFormat::WebP,
    };
    let mut reader = image::ImageReader::new(BufReader::new(
        File::open(path).map_err(|_| AppError::file())?,
    ))
    .with_guessed_format()
    .map_err(|_| AppError::file())?;
    if reader.format() != Some(format) {
        return Err(AppError::media());
    }
    let dimensions = image::ImageReader::new(BufReader::new(
        File::open(path).map_err(|_| AppError::file())?,
    ));
    let (width, height) = image::ImageReader::with_format(dimensions.into_inner(), format)
        .into_dimensions()
        .map_err(|_| AppError::media())?;
    if width == 0
        || height == 0
        || width > 8192
        || height > 8192
        || u64::from(width) * u64::from(height) > 40_000_000
    {
        return Err(AppError::media());
    }
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(192 * 1024 * 1024);
    reader.limits(limits);
    reader.decode().map_err(|_| AppError::media())?;
    Ok((
        match extension {
            "jpg" => "image/jpeg",
            "png" => "image/png",
            _ => "image/webp",
        },
        Some(width),
        Some(height),
        None,
    ))
}

// Rejeita contêineres truncados e metadados excessivos antes do parser MP4.
fn validate_mp4_boxes(path: &Path, length: u64) -> Result<(), AppError> {
    let mut file = File::open(path).map_err(|_| AppError::file())?;
    let mut position = 0;
    let mut boxes = 0;
    while position < length {
        boxes += 1;
        if boxes > 10000 || length - position < 8 {
            return Err(AppError::media());
        }
        file.seek(SeekFrom::Start(position))
            .map_err(|_| AppError::file())?;
        let mut header = [0u8; 8];
        file.read_exact(&mut header)
            .map_err(|_| AppError::media())?;
        let short = u32::from_be_bytes(header[..4].try_into().map_err(|_| AppError::media())?);
        let (size, minimum) = if short == 1 {
            let mut large = [0u8; 8];
            file.read_exact(&mut large).map_err(|_| AppError::media())?;
            (u64::from_be_bytes(large), 16)
        } else if short == 0 {
            (length - position, 8)
        } else {
            (u64::from(short), 8)
        };
        if size < minimum
            || size > length - position
            || &header[4..] == b"moov" && size > 16 * 1024 * 1024
            || &header[4..] == b"ftyp" && size > 64 * 1024
        {
            return Err(AppError::media());
        }
        position += size;
    }
    Ok(())
}
