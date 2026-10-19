//! Somente pacotes locais sob raízes escolhidas pelo seletor nativo. Nenhuma rede.
use crate::{editorial_domain::Publication, error::AppError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

pub(crate) fn error(code: &'static str, message: &'static str) -> AppError {
    AppError { code, message }
}
pub(crate) fn io_error(e: std::io::Error) -> AppError {
    if matches!(e.raw_os_error(), Some(28 | 39 | 112)) {
        error(
            "EXPORT_SPACE",
            "Não há espaço suficiente para o pacote. Libere espaço e tente novamente.",
        )
    } else {
        error("EXPORT_IO","Não foi possível exportar. Confira a pasta, permissões e espaço livre; os originais foram preservados.")
    }
}
pub(crate) fn linked(path: &Path) -> bool {
    let Ok(m) = fs::symlink_metadata(path) else {
        return false;
    };
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        m.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        m.file_type().is_symlink()
    }
}
pub(crate) fn validate_root(path: &Path) -> Result<PathBuf, AppError> {
    if !path.is_absolute() || !path.is_dir() || linked(path) {
        return Err(error(
            "EXPORT_PATH",
            "Escolha uma pasta local existente, sem links ou atalhos de diretório.",
        ));
    }
    #[cfg(windows)]
    {
        use std::path::Prefix;
        if !matches!(path.components().next(),Some(Component::Prefix(p)) if matches!(p.kind(),Prefix::Disk(_)|Prefix::VerbatimDisk(_)))
        {
            return Err(error(
                "EXPORT_PATH",
                "Use uma pasta em uma unidade local do Windows.",
            ));
        }
    }
    path.canonicalize().map_err(io_error)
}
pub(crate) fn inside(root: &Path, relative: &str) -> Result<PathBuf, AppError> {
    let canonical = validate_root(root)?;
    if canonical != root {
        return Err(error(
            "EXPORT_PATH",
            "A pasta autorizada mudou. Escolha novamente o diretório.",
        ));
    }
    let rel = Path::new(relative);
    if rel.components().count() != 2
        || rel.components().any(|c| !matches!(c, Component::Normal(_)))
        || relative.contains('\\')
        || relative.contains(':')
    {
        return Err(error("EXPORT_PATH", "Caminho de pacote inválido."));
    }
    let mut p = root.to_path_buf();
    for c in rel.components() {
        p.push(c);
        if linked(&p) {
            return Err(error(
                "EXPORT_PATH",
                "Links de diretório não são aceitos em pacotes.",
            ));
        }
        if p.exists() && !p.canonicalize().map_err(io_error)?.starts_with(root) {
            return Err(error("EXPORT_PATH", "Pacote fora da pasta autorizada."));
        }
    }
    Ok(p)
}
pub(crate) fn slug(title: &str) -> String {
    let value: String = title
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let value = value
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let value = value.chars().take(40).collect::<String>();
    if value.is_empty() {
        "publicacao".into()
    } else {
        format!("post-{value}")
    }
}
pub(crate) fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub(crate) fn file_digest(path: &Path) -> Result<String, AppError> {
    if linked(path) {
        return Err(error(
            "EXPORT_PATH",
            "O pacote contém um link de arquivo não permitido.",
        ));
    }
    let mut f = File::open(path).map_err(io_error)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    let mut bytes = 0u64;
    loop {
        let n = f.read(&mut buffer).map_err(io_error)?;
        if n == 0 {
            break;
        }
        bytes += n as u64;
        if bytes > 512 * 1024 * 1024 {
            return Err(AppError::media());
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ManifestMedia {
    pub filename: String,
    pub asset_id: String,
    pub sha256: String,
    pub bytes: u64,
    pub mime_type: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Manifest {
    pub schema: u32,
    pub preparation_id: String,
    pub publication_id: String,
    pub publication_version: u32,
    pub schedule_id: String,
    pub schedule_version: u32,
    pub fingerprint: String,
    pub format: String,
    pub scheduled_date: String,
    pub scheduled_time: String,
    pub timezone: String,
    pub scheduled_at_utc: String,
    pub caption_sha256: String,
    pub media: Vec<ManifestMedia>,
    pub notice: String,
}
pub(crate) trait ExportIo {
    fn available(&self, root: &Path) -> Result<u64, AppError>;
    fn write_guard(&self) -> Result<(), AppError> {
        Ok(())
    }
}
pub(crate) struct LocalIo;
impl ExportIo for LocalIo {
    fn available(&self, root: &Path) -> Result<u64, AppError> {
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            #[link(name = "kernel32")]
            extern "system" {
                fn GetDiskFreeSpaceExW(
                    directory: *const u16,
                    available: *mut u64,
                    total: *mut u64,
                    free: *mut u64,
                ) -> i32;
            }
            let wide: Vec<u16> = root.as_os_str().encode_wide().chain(Some(0)).collect();
            let mut free = 0u64;
            // Ponteiros válidos para UTF-16 e u64; valores de 64 bits preservados.
            if unsafe {
                GetDiskFreeSpaceExW(
                    wide.as_ptr(),
                    &mut free,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            } == 0
            {
                return Err(io_error(std::io::Error::last_os_error()));
            }
            Ok(free)
        }
        #[cfg(not(windows))]
        {
            let _ = root;
            Ok(u64::MAX)
        }
    }
}
fn write_new(path: &Path, bytes: &[u8], io: &dyn ExportIo) -> Result<(), AppError> {
    io.write_guard()?;
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(io_error)?;
    f.write_all(bytes).map_err(io_error)?;
    f.sync_all().map_err(io_error)
}
pub(crate) fn cleanup(stage: &Path) {
    // Remove apenas arquivos regulares do staging conhecido; nunca percorre links/diretórios.
    if linked(stage) || stage.parent().is_none_or(linked) {
        return;
    }
    if let Ok(entries) = fs::read_dir(stage) {
        for entry in entries.flatten() {
            let p = entry.path();
            if !linked(&p) && p.is_file() {
                let _ = fs::remove_file(p);
            }
        }
        let _ = fs::remove_dir(stage);
    }
}
pub(crate) fn write_package(
    root: &Path,
    relative: &str,
    p: &Publication,
    manifest: &Manifest,
    source: impl Fn(&str) -> Result<PathBuf, AppError>,
    io: &dyn ExportIo,
) -> Result<PathBuf, AppError> {
    let final_path = inside(root, relative)?;
    let parent = final_path.parent().ok_or_else(AppError::file)?;
    let required =
        p.media.iter().map(|m| m.size_bytes).sum::<u64>() + p.caption.len() as u64 + 1024 * 1024;
    if io.available(root)? < required {
        return Err(error(
            "EXPORT_SPACE",
            "Não há espaço suficiente para o pacote. Libere espaço e tente novamente.",
        ));
    }
    if !parent.exists() {
        fs::create_dir(parent).map_err(io_error)?;
    }
    inside(root, relative)?;
    if final_path.exists() {
        return Err(error(
            "EXPORT_COLLISION",
            "Já existe um pacote nesse caminho. Nenhum arquivo foi sobrescrito.",
        ));
    }
    let stage = parent.join(format!(".tmp-{}", manifest.preparation_id));
    fs::create_dir(&stage).map_err(io_error)?;
    let result = (|| {
        for (m, export) in p.media.iter().zip(&manifest.media) {
            io.write_guard()?;
            let path = source(&m.storage_path)?;
            let mut input = File::open(path).map_err(io_error)?;
            let mut out = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(stage.join(&export.filename))
                .map_err(io_error)?;
            let mut hash = Sha256::new();
            let mut bytes = 0;
            let mut buffer = [0; 65536];
            loop {
                let n = input.read(&mut buffer).map_err(io_error)?;
                if n == 0 {
                    break;
                }
                bytes += n as u64;
                if bytes > m.size_bytes {
                    return Err(AppError::media());
                }
                hash.update(&buffer[..n]);
                out.write_all(&buffer[..n]).map_err(io_error)?;
            }
            if bytes != m.size_bytes || format!("{:x}", hash.finalize()) != m.checksum {
                return Err(AppError::media());
            }
            out.sync_all().map_err(io_error)?;
        }
        write_new(&stage.join("legenda.txt"), p.caption.as_bytes(), io)?;
        write_new(
            &stage.join("informacoes.json"),
            &serde_json::to_vec_pretty(manifest).map_err(|_| AppError::file())?,
            io,
        )?;
        inside(root, relative)?;
        if linked(&stage) || final_path.exists() {
            return Err(error(
                "EXPORT_COLLISION",
                "O destino mudou durante a preparação. Nenhum pacote existente foi alterado.",
            ));
        }
        fs::rename(&stage, &final_path).map_err(io_error)?;
        Ok(final_path)
    })();
    if result.is_err() && inside(root, relative).is_ok() {
        cleanup(&stage)
    }
    result
}
pub(crate) fn verify_package(path: &Path, manifest: &Manifest) -> Result<(), AppError> {
    let expected = serde_json::to_vec_pretty(manifest).map_err(|_| AppError::file())?;
    let manifest_path = path.join("informacoes.json");
    if linked(&manifest_path)
        || fs::metadata(&manifest_path).map_err(io_error)?.len() != expected.len() as u64
        || fs::read_dir(path).map_err(io_error)?.count() != manifest.media.len() + 2
    {
        return Err(error(
            "EXPORT_OUTDATED",
            "O pacote foi alterado ou está incompleto. Gere uma nova preparação.",
        ));
    }
    let bytes = fs::read(manifest_path).map_err(io_error)?;
    if bytes != expected || file_digest(&path.join("legenda.txt"))? != manifest.caption_sha256 {
        return Err(error(
            "EXPORT_OUTDATED",
            "O pacote foi alterado ou está incompleto. Gere uma nova preparação.",
        ));
    }
    for m in &manifest.media {
        let p = path.join(&m.filename);
        if fs::metadata(&p).map_err(io_error)?.len() != m.bytes || file_digest(&p)? != m.sha256 {
            return Err(error(
                "EXPORT_OUTDATED",
                "O pacote foi alterado ou está incompleto. Gere uma nova preparação.",
            ));
        }
    }
    Ok(())
}

pub(crate) fn system_open(target: &str) -> Result<(), AppError> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        #[link(name = "shell32")]
        extern "system" {
            fn ShellExecuteW(
                hwnd: *mut std::ffi::c_void,
                operation: *const u16,
                file: *const u16,
                parameters: *const u16,
                directory: *const u16,
                show: i32,
            ) -> isize;
        }
        if target.contains('\0') {
            return Err(AppError::file());
        }
        let file: Vec<u16> = std::ffi::OsStr::new(target)
            .encode_wide()
            .chain(Some(0))
            .collect();
        let verb: Vec<u16> = "open".encode_utf16().chain(Some(0)).collect();
        if unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                verb.as_ptr(),
                file.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                1,
            )
        } <= 32
        {
            return Err(error(
                "EXPORT_OPEN",
                "Não foi possível abrir. Verifique o aplicativo padrão do Windows.",
            ));
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = target;
        Err(error(
            "EXPORT_OPEN",
            "A abertura pelo sistema requer Windows.",
        ))
    }
}
