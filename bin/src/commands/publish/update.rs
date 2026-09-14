use markdown_bbcode::MdToBbcode;
use steamworks::UGC;

use crate::{Error, commands::publish::APP_ID, report::Report};

pub fn execute(cmd: &super::Command, ugc: &UGC, create: bool) -> Result<Report, Error> {
    let mut executor = crate::commands::release::executor(&cmd.release, &cmd.build)?;
    let version = executor
        .ctx()
        .config()
        .version()
        .get(executor.ctx().workspace_path().vfs())?;
    let report = executor.run()?;

    let Ok(id) = super::get_id() else {
        error!(
            "Failed to get published file ID, add it to meta.cpp or run `hemtt publish` to create a new item."
        );
        std::process::exit(1);
    };

    let Some(content) = executor.ctx().build_folder() else {
        panic!("Build folder not found");
    };

    let mut handle = ugc.start_item_update(APP_ID, id).content_path(content);
    if create {
        handle = handle
            .title(executor.ctx().config().name())
            .tags(vec!["Mod"], true)
            .add_key_value_tag("bis_platform", "-")
            .add_key_value_tag("bis_displayName", executor.ctx().config().name());
    }
    if let Some(description) = executor.ctx().config().hemtt().publish().description() {
        let content = markdown_headings_to_steam(&fs_err::read_to_string(description)?);
        let mut buf = Vec::new();
        MdToBbcode::new(&content, &mut buf)
            .serialize()
            .expect("valid markdown");
        handle = handle.description(&String::from_utf8(buf).expect("valid UTF-8"));
    }
    let changelog = if let Some(changelog) = executor.ctx().config().hemtt().publish().changelog() {
        let content = fs_err::read_to_string(changelog)?;
        let changelog = parse_changelog::parse(&content).expect("valid changelog");
        let Some(change) = changelog.get(version.to_string().as_str()) else {
            error!("No changelog entry found for version {}", version);
            std::process::exit(1);
        };

        Some(markdown_headings_to_steam(&format!(
            "# Version {}\n\n{}",
            version, change.notes
        )))
    } else {
        Some(format!("Version {version}"))
    };
    let _upload_handle = handle.submit(changelog.as_deref(), |upload_result| match upload_result {
        Ok((published_id, needs_to_agree_to_terms)) => {
            info!("Uploaded item with id {:?}", published_id);
            if needs_to_agree_to_terms {
                warn!("You need to agree to the terms of use before you can upload any files");
            }
        }
        Err(e) => {
            error!("Error uploading item: {:?}", e);
        }
    });
    if supports_hyperlinks::on(supports_hyperlinks::Stream::Stdout) {
        let text = id.0.to_string();
        let url = format!(
            "https://steamcommunity.com/sharedfiles/filedetails/?id={}",
            id.0
        );
        let link = terminal_link::Link::new(&text, &url);
        println!("Updated on Steam Workshop: {link}");
    } else {
        println!(
            "Updated on Steam Workshop: https://steamcommunity.com/sharedfiles/filedetails/?id={}",
            id.0
        );
    }
    Ok(report)
}

pub fn markdown_headings_to_steam(input: &str) -> String {
    input
        .lines()
        .map(|line| {
            let hashes = line.chars().take_while(|&c| c == '#').count();

            if (1..=usize::MAX).contains(&hashes) && line.chars().nth(hashes) == Some(' ') {
                let level = hashes.min(3);
                format!("[h{level}]{}[/h{level}]", &line[hashes + 1..])
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
