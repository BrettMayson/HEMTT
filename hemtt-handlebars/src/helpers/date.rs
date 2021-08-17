use chrono::prelude::*;
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
    let param = h
        .param(0)
        .map_or_else(|| "%s".to_string(), |p| p.value().render());

    let now = if let Some(p) = h.param(1) {
        let p = p.render();
        match p.to_lowercase().as_ref() {
            "utc" => Utc::now().naive_utc(),
            "\"local\"" => Local::now().naive_local(),
            _ => {
                return Err(RenderError::new(format!("Unknown offset: {}", p)));
            }
        }
    } else {
        Local::now().naive_local()
    };
    out.write(&now.format(param.as_ref()).to_string())
        .map_err(|e| RenderError::new(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::prelude::*;
    use handlebars::Handlebars;
    use serde_json::value::Value as Json;
    use std::collections::BTreeMap;

    #[test]
    fn date_year() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("date", Box::new(super::helper));
        handlebars.set_strict_mode(true);
        let data: BTreeMap<&'static str, Json> = BTreeMap::new();
        assert_eq!(
            format!("The year is `{}`", Local::now().year()),
            handlebars
                .render_template("The year is `{{date \"%Y\"}}`", &data)
                .unwrap()
        );
    }

    #[test]
    fn hour_local() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("date", Box::new(super::helper));
        handlebars.set_strict_mode(true);
        let data: BTreeMap<&'static str, Json> = BTreeMap::new();
        assert_eq!(
            format!("The hour is `{}` locally", Local::now().format("%H")),
            handlebars
                .render_template("The hour is `{{date \"%H\"}}` locally", &data)
                .unwrap()
        );
    }

    #[test]
    fn hour_utc() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("date", Box::new(super::helper));
        handlebars.set_strict_mode(true);
        let data: BTreeMap<&'static str, Json> = BTreeMap::new();
        assert_eq!(
            format!("The hour is `{}` utc", Utc::now().format("%H")),
            handlebars
                .render_template("The hour is `{{date \"%H\" \"utc\"}}` utc", &data)
                .unwrap()
        );
    }

    #[test]
    fn hour_invalid() {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("date", Box::new(super::helper));
        handlebars.set_strict_mode(true);
        let data: BTreeMap<&'static str, Json> = BTreeMap::new();
        assert!(handlebars
            .render_template("The hour is `{{date \"%H\" \"nyc\"}}` utc", &data)
            .is_err());
    }
}
