CREATE TABLE guilds (
    id INTEGER PRIMARY KEY,
    discordId varchar(64) NOT NULL
);

CREATE TABLE calendars (
    id INTEGER PRIMARY KEY,
    googleId varchar(90) NOT NULL,
    timezone varchar(30) DEFAULT 'Utc',
    pollInterval INT DEFAULT 5
);

CREATE TABLE guilds_calendars (
    guild_id INTEGER,
    calendar_id INTEGER,
    channelId varchar(64),

    PRIMARY KEY(guild_id, calendar_id, channelId)
);
