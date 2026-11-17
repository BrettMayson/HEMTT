use std::{
    io::Read,
    path::{Path, PathBuf},
};

use clap::Parser;

use sealed_test::prelude::*;

use hemtt_common::{
    config::{ProjectConfig, PublishOptions},
    version::Version,
};

pub(super) struct Project {
    pub root: PathBuf,
}

impl Project {
    pub fn new() -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("hemtt-publish-{}-{unique}", std::process::id()));
        fs_err::create_dir_all(root.join(".hemtt")).expect("temporary project");
        fs_err::write(root.join(".hemtt/project.toml"), "name = 'Project name'\nprefix = 'publish_test'\n[version]\nmajor = 1\nminor = 2\npatch = 3\ngit_hash = 0\n").expect("project config");
        Self { root }
    }

    pub fn options(&self, source: &str) -> PublishOptions {
        fs_err::write(self.root.join(".hemtt/publish.toml"), source).expect("publish config");
        ProjectConfig::from_file(&self.root.join(".hemtt/project.toml"))
            .expect("project options")
            .hemtt()
            .publish()
            .clone()
    }

    pub fn image(&self, name: &str) -> PathBuf {
        let path = self.root.join(name);
        image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            2,
            3,
            image::Rgb([20, 100, 200]),
        ))
        .save(&path)
        .expect("write image");
        path
    }

    pub fn metadata(&self, source: &str) -> super::preflight::Metadata {
        super::preflight::Metadata::prepare(
            &self.options(source),
            "Project name",
            &Version::new(1, 2, 3, None),
            Some(steamworks::PublishedFileId(999)),
        )
        .expect("valid metadata")
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        fs_err::remove_dir_all(&self.root).expect("remove temporary project");
    }
}

#[test]
fn text_conversion_and_change_note_errors_are_recoverable() {
    let text = super::text::description("# Title\n\n**Bold** text\n\n<p>Useful paragraph</p>\n")
        .expect("description");
    assert!(text.contains("Title"));
    assert!(text.contains("[b]Bold[/b]"));
    assert!(!text.contains("<p>"));
    assert!(super::text::change_note("not a changelog", "1.2.3").is_err());
    assert!(
        super::text::change_note(
            "# Changelog\n\n## [1.0.0] - 2026-01-01\n\n- Older release",
            "1.2.3"
        )
        .is_err()
    );
    let note = super::text::change_note(
        "# Changelog\n\n## [1.2.3] - 2026-01-01\n\n- Current change",
        "1.2.3",
    )
    .expect("versioned note");
    assert!(note.contains("Current change"));
    assert!(note.contains("Version 1.2.3"));
}

#[test]
fn prepared_defaults_do_not_overwrite_existing_item_metadata() {
    let project = Project::new();
    let metadata = project.metadata("");
    assert_eq!(metadata.effective_title(false), None);
    assert_eq!(metadata.effective_title(true), Some("Project name"));
    assert!(metadata.tags.is_none());
    assert!(metadata.required_items.is_none());
    assert!(metadata.additional_previews.is_none());
}

// Keep this import available to neighboring test modules without changing cwd.
pub(super) fn version() -> Version {
    Version::new(1, 2, 3, None)
}
pub(super) fn content() -> &'static Path {
    Path::new("release")
}

#[sealed_test]
fn first_assigned_id_is_stamped_into_content_and_archives_without_rebuilding() {
    let _directory = hemtt_test::directory::TemporaryDirectory::copy(&PathBuf::from(format!(
        "{}/tests/workspace_simple",
        env!("CARGO_MANIFEST_DIR")
    )));
    let command = super::Command::parse_from(["publish", "--no-bin", "--no-sign"]);
    let (report, prepared) = super::update::prepare(&command, None).expect("build");
    assert!(!report.failed());
    let mut prepared = prepared.expect("prepared content");
    let archive = Path::new("releases/test-latest.zip");
    assert!(!archive.exists(), "archive must wait for the new ID");
    // A marker written after the build must survive release finalization.
    fs_err::write(prepared.content_path.join("build-once.txt"), "one build").expect("build marker");
    super::create::store_id(1_234_567_890).expect("assigned Workshop ID");
    let report = prepared.finish_release(true).expect("finish release");
    assert!(!report.failed());
    let meta =
        fs_err::read_to_string(prepared.content_path.join("meta.cpp")).expect("built metadata");
    assert!(meta.contains("publishedid = 1234567890;"));
    assert!(meta.contains("timestamp = "));
    let mut zip =
        zip::ZipArchive::new(fs_err::File::open(archive).expect("release archive")).expect("ZIP");
    let meta_name = zip
        .file_names()
        .map(|name| name.expect("archive filename"))
        .find(|name| name.ends_with("/meta.cpp"))
        .expect("archived metadata")
        .to_string();
    let mut archived_meta = String::new();
    zip.by_name(&meta_name)
        .expect("archived metadata")
        .read_to_string(&mut archived_meta)
        .expect("read archived metadata");
    assert_eq!(archived_meta, meta);
    assert!(
        zip.file_names()
            .map(|name| name.expect("archive filename"))
            .any(|name| name.ends_with("/build-once.txt"))
    );
    let versioned = fs_err::read("releases/test-1.0.4.zip").expect("versioned archive");
    assert_eq!(fs_err::read(archive).expect("latest archive"), versioned);
}
