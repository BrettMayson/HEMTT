use rhai::Scope;

pub mod libraries;

pub fn engine() -> rhai::Engine {
    let mut engine = rhai::Engine::new();
    engine.register_fn("date", libraries::time::date);
    engine
}

/// Preprocess a Rhai script with SOURCE and PATH constants
///
/// # Errors
/// If there is an error during preprocessing
pub fn preprocess(script: &str, source: String, path: String) -> Result<String, String> {
    let mut scope = Scope::new();
    scope.push_constant("SOURCE", source);
    scope.push_constant("PATH", path);

    match engine().eval_with_scope::<String>(&mut scope, script) {
        Ok(result) => Ok(result),
        Err(err) => Err(format!("Error during preprocessing: {err}")),
    }
}
