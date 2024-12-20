ALTER TABLE calendars DROP COLUMN "pollInterval";
ALTER TABLE calendars DROP COLUMN "timezone";

ALTER TABLE guilds_calendars
ADD COLUMN "timezone" varchar(60) DEFAULT 'Etc/UTC' NOT NULL;

ALTER TABLE guilds_calendars
ADD COLUMN "pollInterval" int DEFAULT 5 NOT NULL;

ALTER TABLE guilds_calendars
ADD COLUMN "nbDisplayedDays" int DEFAULT 7 NOT NULL;

ALTER TABLE guilds_calendars
ADD COLUMN "skipWeekend" boolean NOT NULL DEFAULT FALSE;

ALTER TABLE guilds_calendars
ADD COLUMN "skipEmptyDays" boolean NOT NULL DEFAULT TRUE;
