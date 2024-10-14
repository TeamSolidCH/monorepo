/*
Calendarbot  Copyright (C) 2023 Zbinden Yohan

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.
This is free software, and you are welcome to redistribute it
 */

use crate::events::VerifyCalendarEvent;
use crate::GCalendar;
use log::info;
use tokio::sync::mpsc::Receiver;

impl GCalendar {
    async fn is_calendar_id_valid_and_accessible(&mut self, calendar_id: &str) -> bool {
        let result = self.hub.calendars().get(&calendar_id).doit().await;

        match result {
            Err(e) => {
                info!("{:?}", e);
                false
            }
            Ok(_) => true,
        }
    }

    pub(crate) fn new_verify_calendar_id_thread(
        self,
        mut rcv: Receiver<VerifyCalendarEvent>,
    ) -> Self {
        let mut self_clone = self.clone();
        tokio::spawn(async move {
            while let Some(VerifyCalendarEvent { calendar_id }) = rcv.recv().await {
                let is_valid = self_clone
                    .is_calendar_id_valid_and_accessible(&calendar_id)
                    .await;
                info!("Calendar ID {} is valid: {}", calendar_id, is_valid);
            }
        });
        self
    }
}
