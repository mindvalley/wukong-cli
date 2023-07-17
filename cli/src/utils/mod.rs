pub mod wukong_sdk;

use chrono::{DateTime, Duration, Local};

pub fn compare_with_current_time(time: &str) -> Duration {
    let current_time: DateTime<Local> = Local::now();
    DateTime::parse_from_rfc3339(time)
        .unwrap()
        .with_timezone(&Local)
        .signed_duration_since(current_time)
}
