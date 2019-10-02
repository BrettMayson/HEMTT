use chrono::prelude::*;
use git2::Repository;
use handlebars::{Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext};

use crate::error::*;

pub fn date(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let param = if let Some(p) = h.param(0) {
        p.value().render()
    } else {
        "%s".to_string()
    };
    out.write(&Local::now().format(param.as_ref()).to_string()).unwrap_or_print();
    Ok(())
}

pub fn git(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let param = if let Some(p) = h.param(0) {
        p.value().render()
    } else {
        "id".to_string()
    };
    let params: Vec<&str> = param.split_whitespace().collect();

    let repo = Repository::open(".").unwrap_or_print();

    if params[0] == "id" {
        // SHA-1 Commit Hash
        let rev = repo.revparse_single("HEAD").unwrap_or_print();
        let id = rev.id().to_string();

        // Default to has length of 8 characters
        let length: usize = match params.get(1) {
            Some(len) => len.parse().unwrap_or(8),
            None => 8,
        };

        let id_sliced = &id[0..length];
        out.write(id_sliced).unwrap_or_print();
    }
    Ok(())
}
