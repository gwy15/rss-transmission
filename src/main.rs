use std::{env, sync::Arc};

use anyhow::Result;
use rss_transmission::{db, Config};

#[tokio::main]
async fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "debug");
    }
    pretty_env_logger::try_init_timed()?;

    let config_s = tokio::fs::read_to_string("config.toml").await?;
    let config: Config = toml::from_str(&config_s)?;
    config.validate()?;

    let db_url = format!("sqlite://{}", config.basic.sqlite_path);
    let pool = db::Pool::connect(&db_url).await?;
    sqlx::migrate!().run(&pool).await?;

    let transmission_client = Arc::new(transmission_rpc::TransClient::with_auth(
        &config.basic.rpc_url,
        transmission_rpc::types::BasicAuth {
            user: config.basic.rpc_username.clone(),
            password: config.basic.rpc_password.clone(),
        },
    ));
    let basic = Arc::new(config.basic);

    let mut futures = vec![];
    for rss in config.rss {
        let runner = rss_transmission::Runner::new(
            basic.clone(),
            rss,
            pool.clone(),
            transmission_client.clone(),
        )?;
        futures.push(async move {
            runner.run().await;
        });
    }
    futures::future::join_all(futures).await;

    Ok(())
}