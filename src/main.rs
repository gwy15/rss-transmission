use std::{env, sync::Arc};

use anyhow::{Context, Result};
use rss_transmission::{db, Config};

#[tokio::main]
async fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var(
            "RUST_LOG",
            "debug,sqlx=warn,rustls=info,transmission_rpc=warn",
        );
    }
    pretty_env_logger::try_init_timed()?;

    let config_path = env::var("CONFIG_FILE").unwrap_or_else(|_| "config.toml".to_string());
    let config_s = tokio::fs::read_to_string(&config_path)
        .await
        .context("Read config file failed.")?;
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
