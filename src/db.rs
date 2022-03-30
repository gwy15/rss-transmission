use anyhow::Result;

pub type Pool = sqlx::Pool<sqlx::Sqlite>;
// pub type Tx<'c> = sqlx::Transaction<'c, sqlx::Sqlite>;

pub struct Torrent {
    pub guid: String,
    pub title: String,
    pub link: String,
    pub torrent_url: String,
    pub torrent_size: i64,
}

impl Torrent {
    pub async fn insert(&self, pool: &Pool) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO `torrents`
                (`guid`, `title`, `link`, `torrent_url`, `torrent_size`)
            VALUES
                (?, ?, ?, ?, ?)
            "#,
            self.guid,
            self.title,
            self.link,
            self.torrent_url,
            self.torrent_size,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn exists(guid: &str, pool: &Pool) -> Result<bool> {
        let exists = sqlx::query_scalar!(
            r#"
            SELECT 1
            FROM `torrents`
            WHERE `guid` = ?
            LIMIT 1;
            "#,
            guid,
        )
        .fetch_optional(pool)
        .await?
        .is_some();

        Ok(exists)
    }
}
