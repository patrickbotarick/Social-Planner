use crate::{
    editorial_domain::*, editorial_repository::EditorialRepository, media_store::checksum,
};
use std::{fs, path::Path};

fn image(directory: &Path, name: &str, red: u8) -> std::path::PathBuf {
    let path = directory.join(name);
    image::RgbImage::from_pixel(24, 16, image::Rgb([red, 100, 50]))
        .save(&path)
        .unwrap();
    path
}
fn input(
    media: &[crate::editorial_domain::MediaAsset],
    format: PublicationFormat,
) -> SavePublication {
    SavePublication {
        id: None,
        expected_version: None,
        title: "Campanha da unidade".into(),
        caption: "Linha 1 😊\nLinha 2 — ação!\n#GMAD #SãoPaulo".into(),
        format,
        media_ids: media.iter().map(|m| m.id.clone()).collect(),
    }
}
fn query() -> LibraryQuery {
    LibraryQuery {
        limit: 24,
        ..Default::default()
    }
}

#[test]
fn creation_text_files_and_preferences_persist_after_reopening() {
    let temp = tempfile::tempdir().unwrap();
    let source = image(temp.path(), "original.png", 10);
    let original = fs::read(&source).unwrap();
    let id;
    let relative;
    {
        let mut repo = EditorialRepository::open(temp.path()).unwrap();
        let staged = repo.storage.stage(&source).unwrap();
        let publication = repo
            .save(input(&[staged], PublicationFormat::Feed))
            .unwrap();
        assert_eq!(
            publication.caption,
            "Linha 1 😊\nLinha 2 — ação!\n#GMAD #SãoPaulo"
        );
        assert_eq!(publication.editorial_status, "draft");
        assert_eq!(publication.source, "manual");
        assert_eq!(publication.media[0].width, Some(24));
        assert_eq!(publication.media[0].height, Some(16));
        assert_eq!(fs::read(&source).unwrap(), original);
        assert_eq!(
            fs::read(publication.media[0].preview_path.as_ref().unwrap()).unwrap(),
            original
        );
        assert!(repo.storage.staged.is_empty());
        assert_eq!(
            repo.storage.staging_directory.read_dir().unwrap().count(),
            0
        );
        id = publication.id;
        relative = publication.media[0].storage_path.clone();
    }
    fs::remove_file(source).unwrap();
    let repo = EditorialRepository::open(temp.path()).unwrap();
    let reopened = repo.get(&id).unwrap();
    assert_eq!(reopened.title, "Campanha da unidade");
    assert!(reopened.caption.contains('😊'));
    assert_eq!(
        fs::read(repo.storage.path(&relative).unwrap()).unwrap(),
        original
    );
    assert_eq!(repo.database.status().unwrap().integrity, "ok");
    assert_eq!(
        repo.database.settings().unwrap().timezone,
        "America/Sao_Paulo"
    );
}

#[test]
fn carousel_order_edit_version_and_conflict_are_transactional() {
    let temp = tempfile::tempdir().unwrap();
    let first = image(temp.path(), "first.png", 20);
    let second = image(temp.path(), "second.png", 30);
    let mut repo = EditorialRepository::open(temp.path()).unwrap();
    let a = repo.storage.stage(&first).unwrap();
    let b = repo.storage.stage(&second).unwrap();
    let publication = repo
        .save(input(&[b, a], PublicationFormat::Carousel))
        .unwrap();
    assert_eq!(publication.media[0].filename, "second.png");
    let mut edit = input(&publication.media, PublicationFormat::Carousel);
    edit.media_ids.reverse();
    edit.id = Some(publication.id.clone());
    edit.expected_version = Some(1);
    edit.title = "Editada".into();
    edit.caption = "Nova legenda 🚀\nPreservada".into();
    let saved = repo.save(edit.clone()).unwrap();
    assert_eq!(saved.version, 2);
    assert_eq!(saved.media[0].filename, "first.png");
    assert_eq!(repo.save(edit).unwrap_err().code, "EDIT_CONFLICT");
    assert_eq!(
        repo.get(&publication.id).unwrap().caption,
        "Nova legenda 🚀\nPreservada"
    );
}

#[test]
fn checksum_deduplication_duplication_and_logical_delete_preserve_shared_assets() {
    let temp = tempfile::tempdir().unwrap();
    let source = image(temp.path(), "source.png", 40);
    let mut repo = EditorialRepository::open(temp.path()).unwrap();
    let a = repo.storage.stage(&source).unwrap();
    let original = repo.save(input(&[a], PublicationFormat::Feed)).unwrap();
    let b = repo.storage.stage(&source).unwrap();
    let another = repo.save(input(&[b], PublicationFormat::Stories)).unwrap();
    assert_eq!(original.media[0].id, another.media[0].id);
    let copy = repo.duplicate(&original.id).unwrap();
    assert_ne!(copy.id, original.id);
    assert_eq!(copy.title, "Campanha da unidade (cópia)");
    assert_eq!(copy.editorial_status, "draft");
    assert_eq!(copy.caption, original.caption);
    assert_eq!(copy.media[0].id, original.media[0].id);
    assert_eq!(repo.storage.directory.read_dir().unwrap().count(), 1);
    assert_eq!(repo.counts().unwrap().total, 3);
    assert_eq!(repo.counts().unwrap().drafts, 3);
    repo.delete(&original.id, 1).unwrap();
    assert_eq!(repo.get(&original.id).unwrap_err().code, "NOT_FOUND");
    assert!(Path::new(copy.media[0].preview_path.as_ref().unwrap()).exists());
    assert_eq!(repo.list(query()).unwrap().total, 2);
    assert!(repo
        .database
        .connection
        .execute("DELETE FROM media_assets", [])
        .is_err());
    assert!(repo
        .database
        .connection
        .prepare("PRAGMA foreign_key_check")
        .unwrap()
        .query([])
        .unwrap()
        .next()
        .unwrap()
        .is_none());
    repo.delete(&copy.id, 1).unwrap();
    repo.delete(&another.id, 1).unwrap();
    assert_eq!(repo.counts().unwrap().total, 0);
    assert_eq!(repo.storage.directory.read_dir().unwrap().count(), 1);
}

#[test]
fn search_filters_sort_and_pagination_use_actual_database_rows() {
    let temp = tempfile::tempdir().unwrap();
    let source = image(temp.path(), "source.png", 50);
    let mut repo = EditorialRepository::open(temp.path()).unwrap();
    let a = repo.storage.stage(&source).unwrap();
    let mut first = input(&[a], PublicationFormat::Feed);
    first.title = "Promoção ÁRVORE".into();
    let saved = repo.save(first).unwrap();
    let mut second = input(&saved.media, PublicationFormat::Stories);
    second.title = "Outro conteúdo".into();
    repo.save(second).unwrap();
    let mut q = query();
    q.search = "promocao arvore".into();
    assert_eq!(repo.list(q).unwrap().items[0].id, saved.id);
    let mut q = query();
    q.search = "#saopaulo".into();
    assert_eq!(repo.list(q).unwrap().total, 2);
    let mut q = query();
    q.format = Some(PublicationFormat::Feed);
    q.source = Some("manual".into());
    q.status = Some("draft".into());
    assert_eq!(repo.list(q).unwrap().total, 1);
    let mut q = query();
    q.source = Some("trello".into());
    assert_eq!(repo.list(q).unwrap().total, 0);
    let mut q = query();
    q.status = Some("approved".into());
    assert_eq!(repo.list(q).unwrap().total, 0);
    let mut q = query();
    q.sort = LibrarySort::Title;
    q.limit = 1;
    let page = repo.list(q).unwrap();
    assert_eq!(page.total, 2);
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].title, "Outro conteúdo");
    let mut q = query();
    q.sort = LibrarySort::Title;
    q.offset = 1;
    q.limit = 1;
    assert_eq!(repo.list(q).unwrap().items[0].id, saved.id);
}

#[test]
fn invalid_files_extensions_missing_paths_and_oversize_are_rejected_without_temps() {
    let temp = tempfile::tempdir().unwrap();
    let mut repo = EditorialRepository::open(temp.path()).unwrap();
    let bad = temp.path().join("bad.png");
    fs::write(&bad, b"not an image").unwrap();
    assert_eq!(repo.storage.stage(&bad).unwrap_err().code, "MEDIA_INVALID");
    assert_eq!(
        repo.storage
            .stage(&temp.path().join("missing.png"))
            .unwrap_err()
            .code,
        "MEDIA_IO"
    );
    let valid = image(temp.path(), "valid.png", 60);
    let wrong = temp.path().join("wrong.jpg");
    fs::copy(valid, &wrong).unwrap();
    assert_eq!(
        repo.storage.stage(&wrong).unwrap_err().code,
        "MEDIA_INVALID"
    );
    let oversized = temp.path().join("large.png");
    fs::File::create(&oversized)
        .unwrap()
        .set_len(64 * 1024 * 1024 + 1)
        .unwrap();
    assert!(repo.storage.stage(&oversized).is_err());
    assert!(repo.storage.staged.is_empty());
    assert_eq!(
        repo.storage.staging_directory.read_dir().unwrap().count(),
        0
    );
    assert_eq!(repo.counts().unwrap().total, 0);
    assert!(repo.storage.path("../planner.sqlite3").is_err());
}

#[test]
fn mp4_metadata_and_checksum_are_read_without_decoding_video() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("video.mp4");
    fs::write(&file, include_bytes!("../../tests/fixtures/minimal.mp4")).unwrap();
    let mut repo = EditorialRepository::open(temp.path()).unwrap();
    let staged = repo.storage.stage(&file).unwrap();
    assert_eq!(staged.mime_type, "video/mp4");
    assert!(staged.width.unwrap() > 0);
    assert!(staged.duration.is_some());
    let saved = repo
        .save(input(&[staged], PublicationFormat::Reels))
        .unwrap();
    assert_eq!(saved.media[0].checksum, checksum(&file).unwrap());
    assert_eq!(
        fs::read(saved.media[0].preview_path.as_ref().unwrap()).unwrap(),
        fs::read(file).unwrap()
    );
}

#[test]
fn format_title_quantity_and_duplicate_media_are_validated_in_backend() {
    let temp = tempfile::tempdir().unwrap();
    let source = image(temp.path(), "source.png", 70);
    let mut repo = EditorialRepository::open(temp.path()).unwrap();
    let staged = repo.storage.stage(&source).unwrap();
    assert!(repo.save(input(&[], PublicationFormat::Feed)).is_err());
    assert!(repo
        .save(input(
            std::slice::from_ref(&staged),
            PublicationFormat::Reels
        ))
        .is_err());
    assert!(repo
        .save(input(
            std::slice::from_ref(&staged),
            PublicationFormat::Carousel
        ))
        .is_err());
    let mut empty = input(std::slice::from_ref(&staged), PublicationFormat::Feed);
    empty.title = "  ".into();
    assert!(repo.save(empty).is_err());
    let mut long = input(std::slice::from_ref(&staged), PublicationFormat::Feed);
    long.caption = "x".repeat(20001);
    assert!(repo.save(long).is_err());
    let duplicate = repo.storage.stage(&source).unwrap();
    assert!(repo
        .save(input(&[staged, duplicate], PublicationFormat::Carousel))
        .is_err());
    assert_eq!(repo.counts().unwrap().total, 0);
}

#[test]
fn database_write_failure_rolls_back_rows_links_and_new_files() {
    let temp = tempfile::tempdir().unwrap();
    let source = image(temp.path(), "source.png", 80);
    let mut repo = EditorialRepository::open(temp.path()).unwrap();
    let staged = repo.storage.stage(&source).unwrap();
    repo.database.connection.execute_batch("CREATE TRIGGER fail_media BEFORE INSERT ON publication_media BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
    assert!(repo
        .save(input(
            std::slice::from_ref(&staged),
            PublicationFormat::Feed
        ))
        .is_err());
    assert_eq!(repo.counts().unwrap().total, 0);
    assert_eq!(
        repo.database
            .connection
            .query_row("SELECT COUNT(*) FROM media_assets", [], |r| r
                .get::<_, i32>(0))
            .unwrap(),
        0
    );
    assert_eq!(repo.storage.directory.read_dir().unwrap().count(), 0);
    assert!(repo.storage.staged.contains_key(&staged.id));
    repo.database
        .connection
        .execute_batch("DROP TRIGGER fail_media;")
        .unwrap();
    assert!(repo.save(input(&[staged], PublicationFormat::Feed)).is_ok());
}

#[test]
fn media_write_failure_and_explicit_cancel_preserve_original() {
    let temp = tempfile::tempdir().unwrap();
    let source = image(temp.path(), "source.png", 90);
    let mut repo = EditorialRepository::open(temp.path()).unwrap();
    let staged = repo.storage.stage(&source).unwrap();
    fs::remove_dir(&repo.storage.directory).unwrap();
    fs::write(&repo.storage.directory, b"blocked").unwrap();
    assert_eq!(
        repo.save(input(
            std::slice::from_ref(&staged),
            PublicationFormat::Feed
        ))
        .unwrap_err()
        .code,
        "MEDIA_IO"
    );
    assert_eq!(repo.counts().unwrap().total, 0);
    repo.storage
        .discard(std::slice::from_ref(&staged.id))
        .unwrap();
    assert!(source.exists());
    assert!(repo.storage.staged.is_empty());
    assert_eq!(
        repo.storage.staging_directory.read_dir().unwrap().count(),
        0
    );
}

#[test]
fn missing_saved_media_prevents_edit_without_losing_existing_content() {
    let temp = tempfile::tempdir().unwrap();
    let source = image(temp.path(), "source.png", 100);
    let mut repo = EditorialRepository::open(temp.path()).unwrap();
    let staged = repo.storage.stage(&source).unwrap();
    let saved = repo
        .save(input(&[staged], PublicationFormat::Feed))
        .unwrap();
    fs::remove_file(saved.media[0].preview_path.as_ref().unwrap()).unwrap();
    let reopened = repo.get(&saved.id).unwrap();
    assert!(reopened.media[0].preview_path.is_none());
    let mut edit = input(&saved.media, PublicationFormat::Feed);
    edit.id = Some(saved.id.clone());
    edit.expected_version = Some(1);
    edit.title = "Não deve salvar".into();
    assert!(repo.save(edit).is_err());
    assert_eq!(repo.get(&saved.id).unwrap().title, saved.title);
}

#[test]
fn editing_add_remove_and_format_change_do_not_invalidate_a_copy() {
    let temp = tempfile::tempdir().unwrap();
    let a = image(temp.path(), "first.png", 101);
    let b = image(temp.path(), "second.png", 102);
    let c = image(temp.path(), "new.png", 103);
    let mut repo = EditorialRepository::open(temp.path()).unwrap();
    let first = repo.storage.stage(&a).unwrap();
    let second = repo.storage.stage(&b).unwrap();
    let saved = repo
        .save(input(&[first, second], PublicationFormat::Carousel))
        .unwrap();
    let copy = repo.duplicate(&saved.id).unwrap();
    let mut edit = input(&saved.media[1..], PublicationFormat::Feed);
    edit.id = Some(saved.id.clone());
    edit.expected_version = Some(1);
    let feed = repo.save(edit).unwrap();
    assert_eq!(feed.media.len(), 1);
    let new = repo.storage.stage(&c).unwrap();
    let mut edit = input(&[new], PublicationFormat::Stories);
    edit.id = Some(saved.id.clone());
    edit.expected_version = Some(2);
    let replaced = repo.save(edit).unwrap();
    assert_eq!(replaced.media[0].filename, "new.png");
    let preserved = repo.get(&copy.id).unwrap();
    assert_eq!(preserved.media.len(), 2);
    for media in &preserved.media {
        assert!(Path::new(media.preview_path.as_ref().unwrap()).exists());
    }
    assert_eq!(repo.database.status().unwrap().integrity, "ok");
}
