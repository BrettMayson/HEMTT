use steamworks::UGC;

use crate::{Error, report::Report};

#[allow(clippy::unnecessary_wraps)]
pub fn execute(ugc: &UGC) -> Result<Report, Error> {
    ugc.create_item(
        super::APP_ID,
        steamworks::FileType::Community,
        |create_result| match create_result {
            Ok((published_id, needs_to_agree_to_terms)) => {
                store_id(published_id.0).expect("Failed to store published id");
                if needs_to_agree_to_terms {
                    warn!("You need to agree to the terms of use before you can upload any files");
                }
            }
            Err(e) => {
                error!("Error creating workshop item: {:?}", e);
            }
        },
    );

    Ok(Report::new())
}

pub fn store_id(published_id: u64) -> Result<(), Error> {
    let content = format!("protocol = 1;\npublishedid = {published_id};");
    let meta_path = std::env::current_dir()?.join("meta.cpp");
    fs_err::write(meta_path, content)?;
    Ok(())
}
