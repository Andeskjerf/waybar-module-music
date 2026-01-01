use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("failed to get current timestamp")
        .as_secs()
}

pub fn microseconds_to_formatted_time(microseconds: u128) -> String {
    let seconds = microseconds / 1000 / 1000;
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}
