CREATE TABLE guilds (
    "id" SERIAL PRIMARY KEY NOT NULL,
    "discordId" VARCHAR(64) NOT NULL
);

CREATE TABLE calendars (
    "id" SERIAL PRIMARY KEY NOT NULL,
    "googleId" VARCHAR(90) NOT NULL,
    "timezone" VARCHAR(30) DEFAULT 'Utc' NOT NULL,
    "pollInterval" INT DEFAULT 5 NOT NULL
);

CREATE TABLE guilds_calendars (
    "guild_id" INTEGER NOT NULL REFERENCES guilds (id) ON DELETE CASCADE,
    "calendar_id" INTEGER NOT NULL REFERENCES calendars (id) ON DELETE CASCADE,
    "channelId" VARCHAR(64) NOT NULL,
    "messageId" VARCHAR(64),
    "forceUpdate" BOOLEAN NOT NULL DEFAULT TRUE,

    PRIMARY KEY ("guild_id", "calendar_id", "channelId")
);
