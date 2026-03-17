use serde::Deserialize;

#[derive(Deserialize)]
pub struct NodeConfig {
    pub port: u16,
}
