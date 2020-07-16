use git2::Repository;
use handlebars::{Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext, RenderError};

pub fn helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
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
            let rev = repo.revparse_single("HEAD").map_err(|e| RenderError::new(e.to_string()))?;
            let id = rev.id().to_string();

            // Default to has length of 8 characters
            let length: usize = match params.get(1) {
                Some(len) => len.parse().unwrap_or(8),
                None => 8,
            };

            let id_sliced = &id[0..length];
            out.write(id_sliced)?;
        }
        "commitCount" => {
            // git rev-list --count HEAD
            let mut revwalk = repo.revwalk().map_err(|e| RenderError::new(e.to_string()))?;
            revwalk.push_head().map_err(|e| RenderError::new(e.to_string()))?;
            out.write(&format!("{}", revwalk.count()))?;
        }
        &_ => {}
    }
    Ok(())
}
