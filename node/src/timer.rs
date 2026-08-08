//! Just a helper struct for determining latency.
#![allow(dead_code)]

use std::panic::Location;

use chrono::DateTime;
use chrono::Utc;
use uuid::Uuid;

pub(crate) struct Timer {
    uuid: Uuid,
    creator: &'static Location<'static>,
    start_time: DateTime<Utc>,
    last: &'static Location<'static>,
    last_time: DateTime<Utc>,
}

impl Timer {
    #[track_caller]
    pub(crate) fn new() -> Self {
        let creator = std::panic::Location::caller();
        let now = Utc::now();
        let uuid = Uuid::new_v4();
        tracing::info!("[{:.8}][{uuid}] START --> {}", 0.0, creator);
        Self {
            uuid,
            creator,
            start_time: now,
            last: creator,
            last_time: now,
        }
    }

    #[track_caller]
    pub(crate) fn so_far(&mut self) {
        let now = Utc::now();
        let time_since = (now - self.last_time).as_seconds_f64();
        let caller = std::panic::Location::caller();
        tracing::info!(
            "[{time_since:.8}][{}] {} --> {}",
            self.uuid,
            self.last,
            caller
        );
        self.last = caller;
        self.last_time = now;
    }
}

impl Drop for Timer {
    #[track_caller]
    fn drop(&mut self) {
        let now = Utc::now();
        let time_since = (now - self.start_time).as_seconds_f64();
        tracing::info!("[{time_since:.8}][{}] {} --> DONE", self.uuid, self.creator);
    }
}
