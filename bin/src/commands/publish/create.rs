use crate::Error;

pub fn store_id(published_id: u64) -> Result<(), Error> {
    let content = format!("protocol = 1;\npublishedid = {published_id};");
    let meta_path = std::env::current_dir()?.join("meta.cpp");
    fs_err::write(meta_path, content)?;
    Ok(())
}
