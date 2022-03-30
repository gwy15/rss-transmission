use anyhow::Result;
use duration_str::deserialize_duration;
use serde::Deserialize;
use std::{path::PathBuf, time::Duration};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub basic: BasicConfig,
    pub rss: Vec<RssConfig>,
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        for rss in self.rss.iter() {
            rss.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct BasicConfig {
    #[serde(deserialize_with = "deserialize_duration")]
    pub interval: Duration,
    pub sqlite_path: String,
    pub rpc_url: String,
    pub rpc_username: String,
    pub rpc_password: String,
}

#[derive(Debug, Deserialize)]
pub struct RssConfig {
    pub name: Option<String>,
    pub url: String,
    pub path: PathBuf,
    pub proxy: Option<String>,
}

impl RssConfig {
    pub fn validate(&self) -> Result<()> {
        if let Some(p) = &self.proxy {
            reqwest::Proxy::all(p)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_config() {
        let s = include_str!("../templates/example.toml");
        let config: Config = toml::from_str(s).unwrap();
        assert_eq!(config.basic.interval, Duration::from_secs(10 * 60));
    }
}
