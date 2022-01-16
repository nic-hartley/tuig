use std::{time::{Duration, Instant}, future::{pending, Future}, task::{Context, Poll}, pin::Pin};

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

    fn flush(&mut self) {
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
        NextFromTimedStream { ts: self }.await
    }

    fn flush(&mut self) {
        let start = self.start;
        let now = Instant::now();
        self.contents.retain(|(_, delay)| start + Duration::from_millis(*delay) > now);
    }
}

pub struct NextFromTimedStream<'a> {
    ts: &'a mut TimedStream,
}

impl<'a> Future for NextFromTimedStream<'a> {
    type Output = Action;
    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        let ts = &mut self.as_mut().ts;
        if ts.contents.is_empty() {
            return Poll::Pending;
        }
        let next_time = ts.start + Duration::from_millis(ts.contents[0].1);
        if Instant::now() < next_time {
            Poll::Pending
        } else {
            Poll::Ready(ts.contents.remove(0).0)
        }
    }
}
