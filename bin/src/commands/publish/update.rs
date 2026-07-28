use steamworks::UGC;

use crate::{Error, commands::publish::APP_ID, report::Report};

pub fn execute(cmd: &super::Command, ugc: &UGC, create: bool) -> Result<Report, Error> {
    let mut executor = crate::commands::release::executor(&cmd.release, &cmd.build)?;
    let version = format!(
        "Version {}",
        executor
            .ctx()
            .config()
            .version()
            .get(executor.ctx().workspace_path().vfs())?
    );
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
    let _upload_handle = handle.submit(Some(&version), |upload_result| match upload_result {
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
