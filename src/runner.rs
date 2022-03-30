use crate::{db, BasicConfig, RssConfig};
use anyhow::Result;
use reqwest::header;
use std::sync::Arc;
use transmission_rpc::TransClient;

pub struct Runner {
    basic_config: Arc<BasicConfig>,
    rss_config: RssConfig,
    rss_client: reqwest::Client,
    pool: db::Pool,
    transmission_client: Arc<TransClient>,
}

impl Runner {
    pub fn new(
        basic: Arc<BasicConfig>,
        rss: RssConfig,
        pool: db::Pool,
        transmission_client: Arc<TransClient>,
    ) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CACHE_CONTROL,
            header::HeaderValue::from_static("no-cache"),
        );
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION"),
            )),
        );
        let mut client = reqwest::ClientBuilder::new().default_headers(headers);

        if let Some(proxy) = &rss.proxy {
            client = client.proxy(reqwest::Proxy::all(proxy)?);
        }

        Ok(Runner {
            basic_config: basic,
            rss_config: rss,
            rss_client: client.build().unwrap(),
            pool,
            transmission_client,
        })
    }

    fn name(&self) -> &str {
        self.rss_config.name.as_deref().unwrap_or("Unset name")
    }

    // loop forever
    pub async fn run(&self) -> ! {
        let mut tick = tokio::time::interval(self.basic_config.interval);
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tick.tick().await;
            match self.check().await {
                Ok(()) => {}
                Err(e) => {
                    debug!("{}: {:?}", self.name(), e);
                    warn!("{}: Waiting for retry...", self.name());
                }
            }
        }
    }

    async fn check(&self) -> Result<()> {
        let response = self.rss_client.get(&self.rss_config.url).send().await?;
        if !response.status().is_success() {
            error!(
                "{}: GET RSS url failed with status code {}",
                self.name(),
                response.status()
            );
            bail!("GET RSS url failed with status code {}", response.status());
        }
        let text = response.bytes().await?;
        let channel = rss::Channel::read_from(&text[..])?;
        info!(
            "{}: RSS channel parsed {} items, title = {}.",
            self.name(),
            channel.items.len(),
            channel.title
        );
        for item in channel.items {
            let guid = item.guid.unwrap_or_default().value;
            let title = item.title.unwrap_or_default();
            let site_link = item.link.unwrap_or_default();
            let enclosure = item.enclosure.unwrap_or_default();
            let torrent_size = enclosure.length().parse().unwrap_or_default();
            let torrent_url = enclosure.url;

            let torrent = db::Torrent {
                guid,
                title,
                link: site_link,
                torrent_url,
                torrent_size,
            };
            self.add_torrent(torrent).await?;
        }

        info!("{} check finished.", self.name());

        Ok(())
    }

    async fn add_torrent(&self, torrent: db::Torrent) -> Result<()> {
        if db::Torrent::exists(&torrent.guid, &self.pool).await? {
            debug!(
                "{}: Torrent {} ({}) already exists, skipping.",
                self.name(),
                torrent.title,
                torrent.guid
            );
            return Ok(());
        }
        debug!(
            "{}: Torrent {} ({}) does not exist.",
            self.name(),
            torrent.title,
            torrent.guid
        );

        let response = self
            .transmission_client
            .torrent_add(transmission_rpc::types::TorrentAddArgs {
                cookies: None,
                download_dir: Some(self.rss_config.path.display().to_string()),
                filename: Some(torrent.torrent_url.clone()),
                metainfo: None,
                paused: None,
                peer_limit: None,
                bandwidth_priority: None,
                files_wanted: None,
                files_unwanted: None,
                priority_high: None,
                priority_low: None,
                priority_normal: None,
            })
            .await;
        match response {
            Ok(response) => {
                let result = response.result;
                if response.arguments.torrent_added.is_none() && result != "success" {
                    error!(
                        "{}: Torrent {} ({}) failed to add. Reason: {}",
                        self.name(),
                        torrent.title,
                        torrent.guid,
                        result
                    );
                    bail!("Torrent failed to add: {}", result);
                }

                info!(
                    "{}: Torrent {} ({}) added. RPC response: {}",
                    self.name(),
                    torrent.title,
                    torrent.guid,
                    result
                );
            }
            Err(e) => {
                error!("{}: Transmission RPC failed: {:?}", self.name(), e);
                return Err(anyhow!(e));
            }
        }

        db::Torrent::insert(&torrent, &self.pool).await?;

        Ok(())
    }
}
