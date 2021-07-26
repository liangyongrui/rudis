use std::time::Instant;

pub struct SnapshotStatus {
    pub create_time: Instant,
    pub base_id: u64,
    pub change_times: u64,
}

impl SnapshotStatus {
    pub fn new(base_id: u64) -> Self {
        Self {
            create_time: Instant::now(),
            change_times: 0,
            base_id,
        }
    }
}
