use anyhow::Result;
use chrono::{Datelike, Local};
use csv::Writer;
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::time_entry::TimeEntry;

pub fn export_csv(entries: &[TimeEntry], config: &Config) -> Result<()> {
    let export_dir = shellexpand::tilde(&config.export.path).to_string();
    fs::create_dir_all(&export_dir)?;

    let now = Local::now();
    let month = now.format("%B").to_string(); // Full month name (e.g., "September")
    let day = now.day().to_string();          // Day without zero padding (e.g., "5")
    let year = now.format("%Y").to_string();  // 4-digit year (e.g., "2025")
    let filename = format!("{}_{}_{}_slothtime.csv", month, day, year);
    let filepath = Path::new(&export_dir).join(filename);

    let mut wtr = Writer::from_path(filepath)?;

    wtr.write_record(&[
        "Row",
        "Task Number",
        "Work Code",
        "Time Entry",
        "Start Time",
        "End Time",
        "Task Time",
    ])?;

    for (i, entry) in entries.iter().enumerate() {
        // Export all rows except entirely empty ones
        if !entry.is_entirely_empty() {
            let task_time = entry
                .calculate_task_time()
                .unwrap_or_else(|| "00:00".to_string());
            wtr.write_record(&[
                (i + 1).to_string(),
                entry.task_number.clone(),
                entry.work_code.clone(),
                entry.time_entry.clone(),
                entry.start_time.clone(),
                entry.end_time.clone(),
                task_time,
            ])?;
        }
    }

    wtr.flush()?;
    Ok(())
}
