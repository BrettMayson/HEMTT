use super::*;
use crate::commands::publish::tests::{Project, version};

#[test]
fn reports_all_local_metadata_errors_together() {
    let project = Project::new();
    let options = project.options(r#"
title = " "
language = "not_a_steam_language"
tags = ["bad,tag", ""]
description = "missing.md"
changelog = "missing-changelog.md"
preview = "missing.png"
required_items = ["0", "abc", "999"]
additional_previews = [{ type = "youtube", video_id = "bad" }, { type = "image", path = "missing-gallery.png" }]
"#);
    let error = Metadata::prepare(&options, "Project", &version(), Some(PublishedFileId(999)))
        .expect_err("invalid preflight")
        .to_string();
    for field in [
        "title",
        "language",
        "tag",
        "description",
        "changelog",
        "missing.png",
        "required_items",
        "itself",
        "video_id",
        "missing-gallery.png",
    ] {
        assert!(error.contains(field), "missing {field}: {error}");
    }
}

#[test]
fn checks_converted_description_size_and_utf8_sources() {
    let project = Project::new();
    let description = project.root.join("description.md");
    fs_err::write(&description, "**b** ".repeat(1_000)).expect("description");
    let options = project.options("description = 'description.md'");
    let error = Metadata::prepare(&options, "Project", &version(), None)
        .expect_err("converted description exceeds limit")
        .to_string();
    assert!(error.contains("description must be shorter than 8000"));
    fs_err::write(description, [0xff, 0xfe]).expect("invalid UTF-8");
    assert!(Metadata::prepare(&options, "Project", &version(), None).is_err());
}

#[test]
fn validates_title_bytes_nul_and_project_name_creation_defaults() {
    let project = Project::new();
    for title in [
        "a".repeat(129),
        "\u{00e9}".repeat(65),
        "nul\0title".to_string(),
    ] {
        let source = format!("title = {}", serde_json::to_string(&title).expect("string"));
        // JSON's \u0000 is also a TOML basic-string escape.
        let options = project.options(&source);
        assert!(Metadata::prepare(&options, "Project", &version(), None).is_err());
    }
    let options = project.options("");
    assert!(Metadata::prepare(&options, &"a".repeat(129), &version(), None).is_err());
    assert!(
        Metadata::prepare(
            &options,
            &"a".repeat(129),
            &version(),
            Some(PublishedFileId(999))
        )
        .is_ok()
    );
}

#[test]
fn resolves_ids_deduplicates_tags_and_preserves_empty_lists() {
    let project = Project::new();
    let metadata = project.metadata("tags = ['Mod', 'Editor Extension', 'Mod']\nrequired_items = ['450814997', '0450814997']\nadditional_previews = []");
    assert_eq!(
        metadata.tags,
        Some(vec!["Mod".to_string(), "Editor Extension".to_string()])
    );
    assert_eq!(metadata.required_items, Some(vec![450_814_997]));
    assert!(
        metadata
            .additional_previews
            .expect("explicit gallery")
            .is_empty()
    );
    let empty = project.metadata("tags = []\nrequired_items = []");
    assert_eq!(empty.tags, Some(vec![]));
    assert_eq!(empty.required_items, Some(vec![]));
}

#[test]
fn validates_all_gallery_formats_and_detects_mislabelled_or_corrupt_images() {
    let project = Project::new();
    for name in ["preview.PNG", "preview.jpeg", "preview.gif"] {
        let path = project.image(name);
        let image = validate_image(&path).expect("supported image");
        assert_eq!((image.width, image.height), (2, 3));
        assert!(image.bytes < MAX_IMAGE_BYTES);
    }
    let png = project.root.join("preview.PNG");
    let wrong = project.root.join("mislabelled.jpg");
    fs_err::copy(&png, &wrong).expect("mismatched extension");
    assert!(
        validate_image(&wrong)
            .expect_err("wrong extension")
            .contains("matching")
    );
    fs_err::write(project.root.join("corrupt.png"), b"\x89PNG\r\n\x1a\n").expect("truncated PNG");
    assert!(validate_image(&project.root.join("corrupt.png")).is_err());
    assert!(validate_image(&project.root).is_err());
}

#[test]
fn rejects_images_at_size_limit_and_validates_primary_and_gallery_sources() {
    let project = Project::new();
    let path = project.image("large.png");
    fs_err::OpenOptions::new()
        .write(true)
        .open(&path)
        .expect("image")
        .set_len(MAX_IMAGE_BYTES as u64)
        .expect("size boundary");
    assert!(
        validate_image(&path)
            .expect_err("size limit")
            .contains("smaller")
    );
    let options = project.options(
        "preview = 'large.png'\nadditional_previews = [{ type = 'image', path = 'missing.gif' }]",
    );
    let error = Metadata::prepare(&options, "Project", &version(), None)
        .expect_err("all image errors")
        .to_string();
    assert!(error.contains("large.png") && error.contains("missing.gif"));
}

#[test]
fn duplicate_gallery_entries_are_reported_but_media_order_is_preserved() {
    let project = Project::new();
    project.image("preview.png");
    let metadata = project.metadata("additional_previews = [{ type = 'youtube', video_id = 'dQw4w9WgXcQ' }, { type = 'image', path = 'preview.png' }]");
    let previews = metadata.additional_previews.expect("gallery");
    assert!(matches!(previews[0], GalleryPreview::Youtube(_)));
    assert!(matches!(previews[1], GalleryPreview::Image(_)));
    let options = project.options("additional_previews = [{ type = 'image', path = 'preview.png' }, { type = 'image', path = './preview.png' }]");
    assert!(
        Metadata::prepare(&options, "Project", &version(), None)
            .expect_err("duplicate paths")
            .to_string()
            .contains("duplicate")
    );
}

#[test]
fn bbcode_descriptions_are_uploaded_unchanged() {
    // https://github.com/BrettMayson/HEMTT/issues/1389
    let source = "[h1] Features [/h1]\n[list]\n[*] Many new tools\n[*] [url=https://example.com]Complete list[/url]\n[/list]\n";
    let project = Project::new();
    for name in ["description.txt", "description.bbcode", "DESCRIPTION"] {
        fs_err::write(project.root.join(name), source).expect("description");
        let metadata = project.metadata(&format!("description = '{name}'"));
        assert_eq!(metadata.description.as_deref(), Some(source), "{name}");
    }
    for name in ["description.md", "description.MARKDOWN"] {
        fs_err::write(project.root.join(name), "* One\n").expect("description");
        let metadata = project.metadata(&format!("description = '{name}'"));
        let text = metadata.description.expect("description");
        assert!(text.contains("[list]"), "{name}: {text}");
    }
}
