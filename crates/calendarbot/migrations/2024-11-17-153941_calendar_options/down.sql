ALTER TABLE calendars ADD COLUMN "pollInterval" INT DEFAULT 5 NOT NULL;
ALTER TABLE calendars ADD COLUMN "timezone" VARCHAR(30) DEFAULT 'Utc' NOT NULL;

ALTER TABLE guilds_calendars DROP COLUMN "timezone";
ALTER TABLE guilds_calendars DROP COLUMN "pollInterval";
ALTER TABLE guilds_calendars DROP COLUMN "nbDisplayedDays";
ALTER TABLE guilds_calendars DROP COLUMN "skipWeekend";
ALTER TABLE guilds_calendars DROP COLUMN "skipEmptyDays";
