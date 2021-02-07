use git2::Repository;
use handlebars::{
    Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext, RenderError,
};

pub fn helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = if let Some(p) = h.param(0) {
        p.value().render()
    } else {
        "id".to_string()
    };
    let params: Vec<&str> = param.split_whitespace().collect();

    let repo = Repository::open(".").map_err(|e| RenderError::new(e.to_string()))?;

    match params[0] {
        "id" => {
            // SHA-1 Commit Hash
            let rev = repo
                .revparse_single("HEAD")
                .map_err(|e| RenderError::new(e.to_string()))?;
            let id = rev.id().to_string();

            // Default to has length of 8 characters
            let length: usize = match params.get(1) {
                Some(len) => len.parse().unwrap_or(8),
                None => 8,
            };

            let id_sliced = &id[0..length];
            out.write(id_sliced)?;
        }
        "commitCount" | "commit_count" => {
            if params[0] == "commitCount" {
                warn!("commitCount is deprecated. use commit_count");
            }
            // git rev-list --count HEAD
            let mut revwalk = repo
                .revwalk()
                .map_err(|e| RenderError::new(e.to_string()))?;
            revwalk
                .push_head()
                .map_err(|e| RenderError::new(e.to_string()))?;
            out.write(&format!("{}", revwalk.count()))?;
        }
        &_ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use fs_extra::{copy_items, dir::CopyOptions};

    use crate::{Variables, render};

    struct TestFolder {
        pub root: PathBuf,
        pub dir: PathBuf,
    }
    impl TestFolder {
        pub fn new() -> Self {
            let root = tempdir::TempDir::new("hemtt_test").unwrap().path().to_path_buf();
            std::fs::create_dir_all(&root).unwrap();
            copy_items(&[PathBuf::from("tests/test-git")], &root, &CopyOptions::default()).unwrap();
            let mut dir = root.clone();
            dir.push("test-git");
            dir.push(".git");
            std::fs::rename({
                let mut root = root.clone();
                root.push("test-git");
                root.push("git");
                root
            }, &dir).unwrap();
            Self {
                root,
                dir,
            }
        }
    }
    impl Drop for TestFolder {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.root).unwrap();
        }
    }

    #[test]
    fn id() {
        let test = TestFolder::new();
        std::env::set_current_dir(&test.dir).unwrap();
        assert_eq!(render("{{git \"id\"}}", &Variables::new()).unwrap(), "1a6bce22");
        assert_eq!(render("{{git}}", &Variables::new()).unwrap(), "1a6bce22");
    }

    #[test]
    fn commit_count() {
        let test = TestFolder::new();
        std::env::set_current_dir(&test.dir).unwrap();
        assert_eq!(render("{{git \"commitCount\"}}", &Variables::new()).unwrap(), "2");
        assert_eq!(render("{{git \"commit_count\"}}", &Variables::new()).unwrap(), "2");
    }
}
