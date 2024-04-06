-- Add migration script here

CREATE TABLE IF NOT EXISTS datapoints
(
    DataPointId    INTEGER PRIMARY KEY NOT NULL,
    CreationDate   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ChannelId      INTEGER NOT NULL DEFAULT 0,
    DataPointValue FLOAT NOT NULL
);