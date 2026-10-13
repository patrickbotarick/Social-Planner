fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&["foundation_status"])),
    )
    .expect("Falha ao preparar os recursos e permissões do Tauri");
}
