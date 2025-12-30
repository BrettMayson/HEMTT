pub use rhai::{EvalAltResult, Scope, packages::Package as _};

use crate::libraries::RfsPackage;

pub mod libraries;

pub fn engine(name: String) -> rhai::Engine {
    let mut engine = rhai::Engine::new();
    engine.register_static_module("hemtt_rfs", RfsPackage::new().as_shared_module());
    engine.register_fn("date", libraries::time::date);
    let inner_name = name.clone();
    engine.on_debug(move |x, _src, _pos| {
        tracing::debug!("[{inner_name}] {x}");
    });
    let inner_name = name.clone();
    engine.on_print(move |s| {
        tracing::info!("[{inner_name}] {s}");
    });
    let inner_name = name.clone();
    engine.register_fn("info", move |s: &str| {
        tracing::info!("[{inner_name}] {s}");
    });
    let inner_name = name.clone();
    engine.register_fn("warn", move |s: &str| {
        tracing::warn!("[{inner_name}] {s}");
    });
    let inner_name = name.clone();
    engine.register_fn("error", move |s: &str| {
        tracing::error!("[{inner_name}] {s}");
    });
    let inner_name = name;
    engine.register_fn("fatal", move |s: &str| -> Result<(), Box<EvalAltResult>> {
        tracing::error!("[{inner_name}] {s}");
        Err(Box::new(EvalAltResult::ErrorTerminated(
            "Script called fatal".into(),
            rhai::Position::NONE,
        )))
    });
    engine
}

/// Preprocess a Rhai script with SOURCE and PATH constants
///
/// # Errors
/// If there is an error during preprocessing
pub fn preprocess(
    mut scope: Scope,
    script_name: String,
    script_source: &str,
    target_path: String,
    target_source: String,
) -> Result<String, String> {
    scope.push_constant("SOURCE", target_source);
    scope.push_constant("PATH", target_path);

    match engine(script_name).eval_with_scope::<String>(&mut scope, script_source) {
        Ok(result) => Ok(result),
        Err(err) => Err(format!("Error during preprocessing: {err}")),
    }
}
