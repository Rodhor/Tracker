// Timer operations — stop, pause, and resume the active entry.
// Split from app.rs to keep update() focused on message handling.
use crate::app::App;
use chrono::{DateTime, Utc};

impl App {
    pub(crate) fn stop_active_timer(&mut self, note: Option<String>) {
        // take() moves the entry out so we can mutate it while active_entry is None
        if let Some(mut entry) = self.active_entry.take() {
            let now = Utc::now();

            // If paused when stopped, fold the final pause into total_paused
            if let Some(paused_at_str) = &entry.paused_at {
                if let Ok(paused_at) = DateTime::parse_from_rfc3339(paused_at_str) {
                    let this_pause = (now - paused_at.with_timezone(&Utc)).num_minutes().max(0);
                    entry.total_paused += this_pause;
                }
            }

            let started = DateTime::parse_from_rfc3339(&entry.started_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or(now);

            let gross_minutes = (now - started).num_minutes().max(0);
            entry.ended_at = Some(now.to_rfc3339());
            entry.minutes = Some((gross_minutes - entry.total_paused).max(0));
            entry.notes = note;
            entry.paused_at = None;

            if let Some(existing) = self.entries.iter_mut().find(|e| e.id == entry.id) {
                *existing = entry;
            }
        }
    }

    pub(crate) fn pause_active_timer(&mut self) {
        if let Some(entry) = self.active_entry.as_mut() {
            if entry.paused_at.is_none() {
                entry.paused_at = Some(Utc::now().to_rfc3339());
                if let Some(existing) = self.entries.iter_mut().find(|e| e.id == entry.id) {
                    *existing = entry.clone();
                }
            }
        }
    }

    pub(crate) fn resume_active_timer(&mut self) {
        if let Some(entry) = self.active_entry.as_mut() {
            if let Some(paused_at_str) = entry.paused_at.take() {
                if let Ok(paused_at) = DateTime::parse_from_rfc3339(&paused_at_str) {
                    let this_pause = (Utc::now() - paused_at.with_timezone(&Utc))
                        .num_minutes()
                        .max(0);
                    entry.total_paused += this_pause;
                }
                if let Some(existing) = self.entries.iter_mut().find(|e| e.id == entry.id) {
                    *existing = entry.clone();
                }
            }
        }
    }
}
