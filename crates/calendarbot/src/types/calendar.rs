use anyhow::anyhow;
use chrono_tz::Tz;
use google_calendar3::api::Event;
use google_calendar3::chrono::{DateTime, Datelike, NaiveDate, TimeDelta, Utc};
use log::warn;
use poise::serenity_prelude as serenity;
use std::collections::BTreeMap;

use crate::models::GuildCalendar;

#[derive(Debug, Copy, Clone)]
pub enum CalendarEventSource {
    GoogleCalendar,
}

#[derive(Clone, Debug, Eq)]
pub struct CalendarOptions {
    pub timezone: Tz,
    pub num_of_days: i32,
    pub skip_weekend: bool,
    pub show_if_no_events: bool,
}

impl PartialEq for CalendarOptions {
    fn eq(&self, other: &Self) -> bool {
        self.timezone == other.timezone
            && self.num_of_days == other.num_of_days
            && self.skip_weekend == other.skip_weekend
            && self.show_if_no_events == other.show_if_no_events
    }
}

impl PartialOrd for CalendarOptions {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CalendarOptions {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.num_of_days
            .cmp(&other.num_of_days)
            .then_with(|| self.skip_weekend.cmp(&other.skip_weekend))
            .then_with(|| self.show_if_no_events.cmp(&other.show_if_no_events))
    }
}

impl TryFrom<GuildCalendar> for CalendarOptions {
    type Error = anyhow::Error;

    fn try_from(guild_calendar: GuildCalendar) -> std::result::Result<Self, Self::Error> {
        let timezone: Tz = match guild_calendar.timezone.parse() {
            Ok(tz) => tz,
            Err(e) => return Err(anyhow!("Failed to parse timezone: {}", e)),
        };

        Ok(Self {
            timezone,
            num_of_days: guild_calendar.nbDisplayedDays,
            show_if_no_events: !guild_calendar.skipEmptyDays,
            skip_weekend: guild_calendar.skipWeekend,
        })
    }
}

#[derive(Clone, Debug)]
pub struct CalendarEvent {
    pub id: String,
    pub summary: String,
    pub description: String,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub event_source: CalendarEventSource,
}

impl CalendarEvent {
    pub fn to_embed(events: Vec<Self>, options: CalendarOptions) -> serenity::CreateEmbed {
        let mut sorted: BTreeMap<(NaiveDate, NaiveDate), Vec<CalendarEvent>> = BTreeMap::new();
        let mut fields: Vec<(String, String, bool)> = vec![];

        for ele in events {
            let ele_clone = ele.clone();

            let start_date = ele_clone.start;
            let end_date = ele_clone.end;

            if let (None, None) = (start_date.as_ref(), end_date.as_ref()) {
                warn!(
                    "Event start date or event end date is None {:?}",
                    ele.clone()
                );
                continue;
            }

            let start_date = start_date.unwrap().with_timezone(&options.timezone);
            let end_date = end_date.unwrap().with_timezone(&options.timezone);

            if options.skip_weekend
                && (start_date.weekday().number_from_monday() > 5
                    || end_date.weekday().number_from_monday() > 5)
            {
                continue;
            }

            if start_date.date_naive() - Utc::now().date_naive()
                > TimeDelta::days(options.num_of_days.into())
            {
                continue;
            }

            sorted
                .entry((start_date.date_naive(), end_date.date_naive()))
                .or_default()
                .push(ele);
        }

        for ((start_date, end_date), events) in sorted.iter() {
            let mut field = String::new();
            for event in events {
                if event.start.is_none() {
                    warn!("Event start is None");
                    continue;
                };

                if event.end.is_none() {
                    warn!("Event end is None");
                    continue;
                };

                let start = event.start.unwrap().with_timezone(&options.timezone);
                let end = event.end.unwrap().with_timezone(&options.timezone);

                field.push_str(&format!(
                    "```{} - {} | {}```\n",
                    start.format("%H:%M"),
                    end.format("%H:%M"),
                    event.summary.clone()
                ));
            }
            let mut format = String::from("**%A** - %e %B");
            if start_date.year() != end_date.year() || start_date.year() != Utc::now().year() {
                format = String::from("%F");
            }

            let mut key = start_date.format(&format.clone()).to_string();

            if start_date.format("%F").to_string() != end_date.format("%F").to_string() {
                key.push_str(
                    &end_date
                        .format(format!(" // {}", format.clone()).as_str())
                        .to_string(),
                );
            }

            fields.push((key, field, false));
        }

        serenity::CreateEmbed::new().title("Events").fields(fields)
    }
}

impl PartialEq for CalendarEvent {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.summary == other.summary
            && self.description == other.description
            && self.start == other.start
            && self.end == other.end
    }
}

impl Eq for CalendarEvent {}

impl TryFrom<Event> for CalendarEvent {
    fn try_from(value: Event) -> Result<Self, Self::Error> {
        let id = match value.id {
            Some(id) => id,
            None => return Err(anyhow!("Event id is missing")),
        };

        let summary = value.summary.unwrap_or_default();
        let description = value.description.unwrap_or_default();

        let start = match value.start {
            Some(start) => start.date_time,
            None => None,
        };

        let end = match value.end {
            Some(end) => end.date_time,
            None => None,
        };

        Ok(Self {
            id,
            summary,
            description,
            start,
            end,
            event_source: CalendarEventSource::GoogleCalendar,
        })
    }

    type Error = anyhow::Error;
}
