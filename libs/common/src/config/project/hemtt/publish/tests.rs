use super::*;

fn options(source: &str) -> PublishOptions {
    toml::from_str::<PublishOptionsFile>(source)
        .expect("valid configuration")
        .resolve(Path::new("project"))
}

#[test]
fn omitted_metadata_and_empty_lists_are_distinct() {
    let omitted = options("");
    assert!(omitted.title().is_none());
    assert!(omitted.tags().is_none());
    assert!(omitted.required_items().is_none());
    assert!(omitted.additional_previews().is_none());
    let empty = options("tags = []\nrequired_items = []\nadditional_previews = []");
    assert_eq!(empty.tags(), Some([].as_slice()));
    assert_eq!(empty.required_items(), Some([].as_slice()));
    assert_eq!(empty.additional_previews(), Some([].as_slice()));
}

#[test]
fn all_metadata_and_media_types_deserialize() {
    let parsed = options(
        r#"
title = "Example"
language = "english"
visibility = "friends_only"
tags = ["Mod", "Editor Extension"]
description = "DESCRIPTION.md"
changelog = "CHANGELOG.md"
preview = "preview.png"
required_items = ["450814997", "463939057"]
additional_previews = [
    { type = "image", path = "gallery/first.png" },
    { type = "youtube", video_id = "dQw4w9WgXcQ" },
]
"#,
    );
    assert_eq!(parsed.title(), Some("Example"));
    assert_eq!(parsed.language(), Some("english"));
    assert_eq!(parsed.visibility(), Some(Visibility::FriendsOnly));
    assert_eq!(
        parsed.description(),
        Some(&PathBuf::from("project/DESCRIPTION.md"))
    );
    assert_eq!(
        parsed.changelog(),
        Some(&PathBuf::from("project/CHANGELOG.md"))
    );
    assert_eq!(parsed.preview(), Some(Path::new("project/preview.png")));
    assert_eq!(parsed.required_items().expect("dependencies").len(), 2);
    assert_eq!(
        parsed.additional_previews().expect("gallery")[0],
        AdditionalPreview::Image {
            path: "project/gallery/first.png".into()
        }
    );
    assert_eq!(
        parsed.additional_previews().expect("gallery")[1],
        AdditionalPreview::Youtube {
            video_id: "dQw4w9WgXcQ".into()
        }
    );
}

#[test]
fn rejects_unknown_fields_and_inconsistent_media_records() {
    for source in [
        "visiblity = 'public'",
        "visibility = 'hidden'",
        "key_value_tags = []",
        "[translations.german]\ndescription = 'de.md'",
        "additional_previews = [{ type = 'youtube', id = 'dQw4w9WgXcQ' }]",
        "additional_previews = [{ type = 'youtube', video_id = 'dQw4w9WgXcQ', path = 'x.png' }]",
        "additional_previews = [{ type = 'image', video_id = 'dQw4w9WgXcQ' }]",
    ] {
        assert!(
            toml::from_str::<PublishOptionsFile>(source).is_err(),
            "accepted {source}"
        );
    }
}

#[test]
fn separate_file_matches_inline_options_and_conflicts_are_rejected() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hemtt-publish-config-{}-{unique}",
        std::process::id()
    ));
    let hemtt = root.join(".hemtt");
    fs_err::create_dir_all(&hemtt).expect("temporary project");
    let source = "title = 'Example'\ndescription = 'DESCRIPTION.md'\npreview = 'preview.png'";
    let project = hemtt.join("project.toml");
    let inline = toml::from_str::<PublishOptionsFile>(source).expect("inline config");
    let expected = inline.clone().load(&project).expect("inline options");
    fs_err::write(hemtt.join("publish.toml"), source).expect("separate config");
    assert_eq!(
        PublishOptionsFile::default()
            .load(&project)
            .expect("file options"),
        expected
    );
    assert_eq!(expected.description(), Some(&root.join("DESCRIPTION.md")));
    assert!(inline.load(&project).is_err());
    fs_err::write(
        hemtt.join("publish.toml"),
        "[hemtt.publish]\ntitle = 'Wrong nesting'",
    )
    .expect("invalid file");
    assert!(PublishOptionsFile::default().load(&project).is_err());
    fs_err::remove_dir_all(root).expect("remove temporary project");
}
