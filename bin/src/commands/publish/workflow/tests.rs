use std::cell::RefCell;

use super::*;
use crate::commands::publish::tests::{Project, content};

#[derive(Debug, PartialEq, Eq)]
enum Event {
    Inspect(bool, bool),
    Submit {
        remove_previews: u32,
        gallery: Option<usize>,
        title: Option<String>,
    },
    Dependency(u64, bool),
}

#[derive(Default)]
struct FakeSteam {
    events: RefCell<Vec<Event>>,
    existing: Vec<u64>,
    previews: u32,
    fail_inspect: bool,
    fail_submit: bool,
    fail_dependency: bool,
}

impl Workshop for FakeSteam {
    fn inspect(
        &self,
        _: PublishedFileId,
        children: bool,
        previews: bool,
    ) -> Result<RemoteState, Error> {
        self.events
            .borrow_mut()
            .push(Event::Inspect(children, previews));
        if self.fail_inspect {
            return Err(Error::Workshop("query failure".into()));
        }
        Ok(RemoteState {
            required_items: self.existing.clone(),
            additional_previews: self.previews,
        })
    }

    fn submit(&self, _: PublishedFileId, update: &ItemUpdate<'_>) -> Result<(), Error> {
        self.events.borrow_mut().push(Event::Submit {
            remove_previews: update.remove_previews,
            gallery: update.metadata.additional_previews.as_ref().map(Vec::len),
            title: update
                .metadata
                .effective_title(update.create)
                .map(str::to_string),
        });
        if self.fail_submit {
            Err(Error::Workshop("upload failure".into()))
        } else {
            Ok(())
        }
    }

    fn dependency(&self, _: PublishedFileId, child: u64, add: bool) -> Result<(), Error> {
        self.events.borrow_mut().push(Event::Dependency(child, add));
        if self.fail_dependency {
            Err(Error::Workshop("dependency failure".into()))
        } else {
            Ok(())
        }
    }
}

#[test]
fn omitted_lists_do_not_query_or_change_existing_lists() {
    let project = Project::new();
    let metadata = project.metadata("");
    let steam = FakeSteam::default();
    publish(&steam, PublishedFileId(999), content(), &metadata, false).expect("publish");
    assert_eq!(
        *steam.events.borrow(),
        vec![Event::Submit {
            remove_previews: 0,
            gallery: None,
            title: None
        }]
    );
}

#[test]
fn replaces_gallery_and_only_changes_dependency_differences_after_upload() {
    let project = Project::new();
    let metadata = project.metadata("title = 'Changed'\nrequired_items = ['20', '30']\nadditional_previews = [{ type = 'youtube', video_id = 'dQw4w9WgXcQ' }]");
    let steam = FakeSteam {
        existing: vec![10, 20],
        previews: 4,
        ..Default::default()
    };
    publish(&steam, PublishedFileId(999), content(), &metadata, false).expect("publish");
    assert_eq!(
        *steam.events.borrow(),
        vec![
            Event::Inspect(true, true),
            Event::Submit {
                remove_previews: 4,
                gallery: Some(1),
                title: Some("Changed".into())
            },
            Event::Dependency(30, true),
            Event::Dependency(10, false),
        ]
    );
}

#[test]
fn repeated_publish_does_not_duplicate_dependencies_or_gallery_entries() {
    let project = Project::new();
    let metadata = project.metadata("required_items = ['20']\nadditional_previews = [{ type = 'youtube', video_id = 'dQw4w9WgXcQ' }]");
    let steam = FakeSteam {
        existing: vec![20],
        previews: 1,
        ..Default::default()
    };
    for _ in 0..2 {
        publish(&steam, PublishedFileId(999), content(), &metadata, false).expect("repeat publish");
    }
    assert_eq!(
        *steam.events.borrow(),
        vec![
            Event::Inspect(true, true),
            Event::Submit {
                remove_previews: 1,
                gallery: Some(1),
                title: None
            },
            Event::Inspect(true, true),
            Event::Submit {
                remove_previews: 1,
                gallery: Some(1),
                title: None
            },
        ]
    );
}

#[test]
fn empty_lists_clear_existing_lists() {
    let project = Project::new();
    let metadata = project.metadata("required_items = []\nadditional_previews = []");
    let steam = FakeSteam {
        existing: vec![20],
        previews: 3,
        ..Default::default()
    };
    publish(&steam, PublishedFileId(999), content(), &metadata, false).expect("clear lists");
    assert_eq!(
        *steam.events.borrow(),
        vec![
            Event::Inspect(true, true),
            Event::Submit {
                remove_previews: 3,
                gallery: Some(0),
                title: None
            },
            Event::Dependency(20, false)
        ]
    );
}

#[test]
fn query_or_upload_failure_prevents_later_mutations() {
    let project = Project::new();
    let metadata = project.metadata("required_items = ['20']");
    let steam = FakeSteam {
        fail_inspect: true,
        ..Default::default()
    };
    assert!(publish(&steam, PublishedFileId(999), content(), &metadata, false).is_err());
    assert_eq!(*steam.events.borrow(), vec![Event::Inspect(true, false)]);
    let steam = FakeSteam {
        fail_submit: true,
        ..Default::default()
    };
    assert!(publish(&steam, PublishedFileId(999), content(), &metadata, false).is_err());
    assert_eq!(steam.events.borrow().len(), 2);
}

#[test]
fn creation_uses_defaults_and_skips_existing_state_query() {
    let project = Project::new();
    let metadata = project.metadata("required_items = ['20']");
    let steam = FakeSteam::default();
    publish(&steam, PublishedFileId(999), content(), &metadata, true).expect("create upload");
    assert_eq!(
        *steam.events.borrow(),
        vec![
            Event::Submit {
                remove_previews: 0,
                gallery: None,
                title: Some("Project name".into())
            },
            Event::Dependency(20, true)
        ]
    );
}

#[test]
fn dependency_error_reports_partial_success_and_preserves_old_requirements() {
    let project = Project::new();
    let metadata = project.metadata("required_items = ['20']");
    let steam = FakeSteam {
        existing: vec![10],
        fail_dependency: true,
        ..Default::default()
    };
    let error = publish(&steam, PublishedFileId(999), content(), &metadata, false)
        .expect_err("dependency failure")
        .to_string();
    assert!(error.contains("upload succeeded"));
    assert!(
        !steam
            .events
            .borrow()
            .contains(&Event::Dependency(10, false))
    );
}

#[test]
fn self_dependency_is_rechecked_after_interactive_id_selection() {
    let project = Project::new();
    let mut metadata = project.metadata("");
    metadata.required_items = Some(vec![999]);
    let steam = FakeSteam::default();
    assert!(publish(&steam, PublishedFileId(999), content(), &metadata, false).is_err());
    assert!(steam.events.borrow().is_empty());
}
