use anyhow::Result;
use chrono::Local;
use csv::Writer;
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::time_entry::TimeEntry;

pub fn export_csv(entries: &[TimeEntry], config: &Config) -> Result<()> {
    let export_dir = shellexpand::tilde(&config.export.path).to_string();
    fs::create_dir_all(&export_dir)?;

    let date = Local::now().date_naive();
    let filename = format!("slothtime_{}.csv", date);
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
        if entry.is_complete() {
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
