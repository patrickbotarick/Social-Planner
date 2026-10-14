fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "foundation_status",
            "publication_list",
            "publication_get",
            "publication_save",
            "publication_duplicate",
            "publication_delete",
            "editorial_counts",
            "media_select",
            "media_discard",
            "trello_status",
            "trello_connect",
            "trello_test",
            "trello_disconnect",
            "trello_authorization_url",
            "trello_preview",
            "trello_history",
            "trello_source",
            "trello_cancel",
            "trello_import",
        ]),
    ))
    .expect("Falha ao preparar os recursos e permissões do Tauri");
}
