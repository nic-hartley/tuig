//! Miscellaneous helper types around controlling the timing of events.

use std::time::{Instant, Duration};

/// Keeps track of time between relatively steady pulses.
/// 
/// Ticks try to stay lined up with the original tick, but if [`Self::tick`] is called more than half a period
/// delayed, the next tick will be reset relative to the current time instead. If called early it will always advance
/// by exactly one tick.
pub struct Timer {
    next: Instant,
    period: Duration,
}

impl Timer {
    /// Create a new timer with the given period. The first tick is right now.
    pub fn new(period: f32) -> Self {
        Self { next: Instant::now(), period: Duration::from_secs_f32(period) }
    }

    /// How much time is left before the timer ticks over. Minimum 0.0.
    pub fn remaining(&self) -> Duration {
        self.next.checked_duration_since(Instant::now())
            .unwrap_or(Duration::ZERO)
    }

    /// Reset the timer
    pub fn tick(&mut self) {
        let now = Instant::now();
        if now < self.next + self.period / 2 {
            self.next += self.period;
        } else {
            self.next = now + self.period;
        }
    }

    /// Check whether we've ticked yet; if so, reset the timer. Useful for ratelimiting.
    pub fn ready(&mut self) -> bool {
        if Instant::now() > self.next {
            self.tick();
            true
        } else {
            false
        }
    }
}
