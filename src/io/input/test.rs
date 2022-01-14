use std::{time::{Duration, Instant}, future::pending};

use tokio::time::sleep;

use super::{Action, Input};

pub struct UntimedStream {
    contents: Vec<Action>,
}

impl UntimedStream {
    pub fn of(actions: &[Action]) -> UntimedStream {
        UntimedStream { contents: actions.to_vec() }
    }
}

#[async_trait::async_trait]
impl Input for UntimedStream {
    async fn next(&mut self) -> Action {
        if self.contents.is_empty() {
            pending().await
        } else {
            self.contents.remove(0)
        }
    }

    async fn flush(&mut self) {
        self.contents.clear();
    }
}

pub struct TimedStream {
    contents: Vec<(Action, u64)>,
    start: Instant,
}

impl TimedStream {
    #[allow(unused)]
    pub fn milliseconds(timings: &[(Action, u64)]) -> TimedStream {
        TimedStream { contents: timings.into(), start: Instant::now() }
    }
}

#[async_trait::async_trait]
impl Input for TimedStream {
    async fn next(&mut self) -> Action {
        if self.contents.is_empty() {
            return pending().await;
        }
        let next_time = self.start + Duration::from_millis(self.contents[0].1);
        while Instant::now() < next_time {
            sleep(next_time - Instant::now()).await;
        }
        return self.contents.remove(0).0;
    }

    async fn flush(&mut self) {
        let start = self.start;
        self.contents.retain(|(_, delay)| start + Duration::from_millis(*delay) > Instant::now());
    }
}
