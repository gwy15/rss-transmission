#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;

mod config;
pub use config::{BasicConfig, Config, RssConfig};

mod runner;
pub use runner::Runner;

pub mod db;
