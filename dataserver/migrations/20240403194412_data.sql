-- Add migration script here

CREATE TABLE IF NOT EXISTS datapoints
(
    id             INTEGER PRIMARY KEY NOT NULL,
    CreationDate   DATETIME DEFAULT CURRENT_TIMESTAMP,
    ChannelId      INTEGER NOT NULL DEFAULT 0,
    DataPointValue FLOAT NOT NULL
);