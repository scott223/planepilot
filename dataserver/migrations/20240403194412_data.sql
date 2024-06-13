-- Add migration script here

CREATE TABLE IF NOT EXISTS datapoints
(
    DataPointId    INTEGER PRIMARY KEY NOT NULL,
    CreationDate   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ChannelName        VARCHAR NOT NULL,
    DataPointValue FLOAT NOT NULL
);