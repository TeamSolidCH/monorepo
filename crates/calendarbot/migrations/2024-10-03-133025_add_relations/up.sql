ALTER TABLE guilds_calendars
ADD CONSTRAINT fk_guilds_calendars_guilds
FOREIGN KEY (guild_id) REFERENCES guilds(id);

ALTER TABLE guilds_calendars
ADD CONSTRAINT fk_guilds_calendars_calendars
FOREIGN KEY (calendar_id) REFERENCES calendars(id);
