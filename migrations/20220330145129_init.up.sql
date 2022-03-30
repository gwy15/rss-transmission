-- Add up migration script here
CREATE TABLE `torrents` (
    `guid`          BIGINT  NOT NULL UNIQUE PRIMARY KEY,
    `title`         TEXT    NOT NULL,
    `link`          TEXT    NOT NULL,
    `torrent_url`   TEXT    NOT NULL,
    `torrent_size`  BIGINT  NOT NULL
);
