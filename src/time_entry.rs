use serde::{Deserialize, Serialize};
use chrono::NaiveTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    pub task_number: String,
    pub work_code: String,
    pub time_entry: String,
    pub start_time: String,
    pub end_time: String,
}

impl TimeEntry {
    pub fn new() -> Self {
        Self {
            task_number: String::new(),
            work_code: String::new(),
            time_entry: String::new(),
            start_time: String::new(),
            end_time: String::new(),
        }
    }

    pub fn is_complete(&self) -> bool {
        !self.task_number.is_empty()
            && !self.work_code.is_empty()
            && !self.time_entry.is_empty()
            && !self.start_time.is_empty()
            && !self.end_time.is_empty()
    }

    pub fn is_entirely_empty(&self) -> bool {
        self.task_number.is_empty()
            && self.work_code.is_empty()
            && self.time_entry.is_empty()
            && self.start_time.is_empty()
            && self.end_time.is_empty()
    }

    pub fn calculate_task_time(&self) -> Option<String> {
        if self.start_time.is_empty() || self.end_time.is_empty() {
            return None;
        }

        let start = Self::parse_time(&self.start_time)?;
        let end = Self::parse_time(&self.end_time)?;

        if end < start {
            return None; // invalid
        }

        let duration = end - start;
        let hours = duration.num_hours();
        let minutes = duration.num_minutes() % 60;
        Some(format!("{:02}:{:02}", hours, minutes))
    }

    fn parse_time(time_str: &str) -> Option<NaiveTime> {
        // Support HH:MM or HHMM
        let time_str = time_str.replace(":", "");
        if time_str.len() == 4 {
            let hour: u32 = time_str[0..2].parse().ok()?;
            let min: u32 = time_str[2..4].parse().ok()?;
            NaiveTime::from_hms_opt(hour, min, 0)
        } else {
            None
        }
    }
}