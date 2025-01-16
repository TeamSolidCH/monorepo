/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */

use crate::events::CalendarCommands;
use crate::GCalendar;
use log::{info, trace};
use regex::Regex;
use tokio::sync::mpsc::Receiver;

impl GCalendar {
    async fn is_calendar_id_valid_and_accessible(self: &Self, calendar_id: &str) -> bool {
        static CALENDAR_ID_REGEX: &str = r"^(\w+\.){0,3}\w+@(\w+\.){0,3}\w+$";

        let re = Regex::new(CALENDAR_ID_REGEX).unwrap();

        if !re.is_match(calendar_id) {
            return false;
        }

        trace!("Calendar ID is valid, checking if accessible");
        let result = self.hub.calendars().get(calendar_id).doit().await;
        trace!("Calendar ID is valid, checking if accessible");

        match result {
            Err(e) => {
                info!("{:?}", e);
                false
            }
            Ok(_) => true,
        }
    }

    pub(crate) fn new_worker_thread(self, mut rcv: Receiver<CalendarCommands>) -> Self {
        let mut self_clone = self.clone();
        info!("Starting worker thread");
        tokio::spawn(async move {
            while let Some(cmd) = rcv.recv().await {
                trace!("Received command: {:?}", cmd);
                match cmd {
                    CalendarCommands::VerifyCalendarId { calendar_id, resp } => {
                        let is_valid = self_clone
                            .is_calendar_id_valid_and_accessible(&calendar_id)
                            .await;
                        let _ = resp.send(Ok(is_valid));
                    }
                }
            }
        });
        self
    }
}
