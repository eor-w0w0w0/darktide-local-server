use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct Config {
    pub port: Option<u16>,
    pub enable_portproxy: Option<bool>,
}

#[derive(Deserialize)]
pub struct RunRequest {
    pub command: String,
}

pub const MUTEX_NAME: &str = "Global\\DarktideLocalServerMutex";
pub const DEFAULT_PORT: u16 = 41012;
pub const CONFIG_NAME: &str = "config.json";
pub const SUCCESS: &str = "success";
pub const PID: &str = "pid";
