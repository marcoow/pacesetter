use anyhow::anyhow;
use {{crate_name}}_config::Environment;

/// Utilities for console UIs
pub mod ui;

/// Parses the [`my_app_config::Environment`] the CLI runs in from a string.
///
/// The environment can be passed in different forms, e.g. "dev", "development", "prod", etc. If an invalid environment is passed, this returns an error.
pub fn parse_env(s: &str) -> Result<Environment, &'static str> {
    let s = &s.to_lowercase();
    match parse_env_internal(s) {
        Ok(env) => Ok(env),
        Err(_) => Err("Cannot parse environment!"),
    }
}

fn parse_env_internal(env: &str) -> Result<Environment, anyhow::Error> {
    let env = &env.to_lowercase();
    match env.as_str() {
        "dev" => Ok(Environment::Development),
        "development" => Ok(Environment::Development),
        "test" => Ok(Environment::Test),
        "prod" => Ok(Environment::Production),
        "production" => Ok(Environment::Production),
        unknown => Err(anyhow!(r#"Unknown environment: "{}"!"#, unknown)),
    }
}
