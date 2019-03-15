use chrono::prelude::*;
use handlebars::{Handlebars, RenderContext, Helper, Context, JsonRender, HelperResult, Output};

use crate::error::*;

pub fn date (h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut Output) -> HelperResult {
    let param = h.param(0).unwrap().value().render();
    out.write(&Local::now().format(param.as_ref()).to_string()).unwrap_or_print();
    Ok(())
}
