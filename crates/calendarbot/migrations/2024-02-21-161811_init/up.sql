CREATE TABLE guilds (
    id SERIAL PRIMARY KEY NOT NULL,
    discordId varchar(64) NOT NULL
);

CREATE TABLE calendars (
    id SERIAL PRIMARY KEY NOT NULL,
    googleId varchar(90) NOT NULL,
    timezone varchar(30) DEFAULT 'Utc' NOT NULL,
    pollInterval INT DEFAULT 5 NOT NULL
);

CREATE TABLE guilds_calendars (
    guild_id INTEGER NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    calendar_id INTEGER NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
    channelId varchar(64) NOT NULL,
		messageId varchar(64),
		forceUpdate BOOLEAN NOT NULL DEFAULT TRUE,

    PRIMARY KEY(guild_id, calendar_id, channelId)
);
