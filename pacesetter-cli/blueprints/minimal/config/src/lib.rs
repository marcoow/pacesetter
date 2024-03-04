use pacesetter::config::ServerConfig;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub server: ServerConfig,
    // add your config settings here…
}
