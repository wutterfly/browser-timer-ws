use std::time::SystemTime;

pub fn now() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Should always be later then UNIX_EPOCH")
        .as_millis()
}
