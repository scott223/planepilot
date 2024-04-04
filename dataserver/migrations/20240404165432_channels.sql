-- Add migration script here
CREATE TABLE IF NOT EXISTS channels
(
    ChannelId      INTEGER PRIMARY KEY NOT NULL,
    ChannelName    CHAR(255) NOT NULL
);