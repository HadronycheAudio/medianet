//! Datagram pacing: hold a target rate by scheduling send k at
//! `start + k / rate`, sleeping while the wait is long and spinning for the
//! last stretch. A plain sleep per datagram overshoots by more than one
//! interval at the rates this probe runs (tens of microseconds); the hybrid
//! keeps the drift bounded and catches up in a burst if it ever falls back.

use std::thread;
use std::time::{Duration, Instant};

pub struct Pacer {
    start: Instant,
    interval: Duration,
    slot: u64,
}

impl Pacer {
    /// A pacer for `rate` sends per second; slot 0 is due immediately.
    pub fn new(rate: f64) -> Pacer {
        Pacer {
            start: Instant::now(),
            interval: Duration::from_secs_f64(1.0 / rate),
            slot: 0,
        }
    }

    /// Block until the current slot is due, then advance to the next one.
    pub fn wait(&mut self) {
        let target = self.start + self.interval * (self.slot.min(u64::from(u32::MAX)) as u32);
        while Instant::now() < target {
            let left = target - Instant::now();
            if left > Duration::from_micros(250) {
                thread::sleep(left - Duration::from_micros(150));
            } else {
                std::hint::spin_loop();
            }
        }
        self.slot += 1;
    }
}
