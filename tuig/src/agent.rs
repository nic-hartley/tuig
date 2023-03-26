/// The `Agent` trait and its `ControlFlow`
use core::fmt;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

#[cfg(test)]
use mock_instant::Instant;
#[cfg(not(test))]
use std::time::Instant;

use crate::{Message, Replies};

/// See [`ControlFlow::Handle`].
#[derive(Clone)]
pub struct WaitHandle(Arc<AtomicBool>);

impl WaitHandle {
    /// Create a new waiting handle
    fn new() -> Self {
        WaitHandle(Arc::new(AtomicBool::new(false)))
    }

    /// Notify the waiting agent that it can wake up.
    pub fn wake(&self) {
        self.0.store(true, Ordering::Release);
    }

    /// Check whether [`Self::wake`] has been called on this handle yet
    fn is_woken(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    /// How many threads, right at the moment of calling this, have a handle.
    ///
    /// See [Arc::strong_count][1] for important caveats about its use.
    ///
    ///  [1]: https://doc.rust-lang.org/std/sync/struct.Arc.html#method.strong_count
    pub fn references(&self) -> usize {
        Arc::strong_count(&self.0)
    }
}

impl PartialEq for WaitHandle {
    #[cfg_attr(coverage, no_coverage)]
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for WaitHandle {}
impl fmt::Debug for WaitHandle {
    #[cfg_attr(coverage, no_coverage)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WaitHandle(...)")
    }
}

/// What should happen to an [`Agent`][super::Agent] after it finishes [react][super::Agent::react]ing to
/// [`Message`][super::Message]s.
///
/// Note that this only defines when [`Agent::react`][super::Agent::react] *should* start being called again. The
/// associated agent will never skip a round it should have seen, but it may see rounds it wasn't supposed to. Treat
/// this like an optimization; if you report `ControlFlow` accurately, you can save the engine a bit of time when it
/// processes events.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ControlFlow {
    /// Continue as normal and update next time.
    Continue,
    /// Stop updating this agent and (eventually) destroy it.
    Kill,
    /// Wait until notified by someone who has the handle
    Handle(WaitHandle),
    /// Sleep, waking up at the given time
    Time(Instant),
}

impl ControlFlow {
    /// Create a new [`ControlFlow::Handle`], with its handle so something else can wake it.
    pub fn wait() -> (Self, WaitHandle) {
        let wh = WaitHandle::new();
        (Self::Handle(wh.clone()), wh)
    }

    /// Create a new [`ControlFlow::Time`], waiting until the given time.
    pub fn sleep_until(time: Instant) -> Self {
        Self::Time(time)
    }

    /// Create a new [`ControlFlow::Time`], waiting for a given duration.
    pub fn sleep_for(amt: Duration) -> Self {
        Self::Time(Instant::now() + amt)
    }

    /// Check whether an agent which returned this control flow is ready to start reacting again.
    pub fn is_ready(&self) -> bool {
        match self {
            ControlFlow::Continue => true,
            ControlFlow::Kill => false,
            ControlFlow::Handle(wh) => wh.is_woken(),
            ControlFlow::Time(when) => &Instant::now() > when,
        }
    }
}

#[cfg(test)]
mod cf_test {
    use std::time::Duration;

    use mock_instant::MockClock;

    use super::{ControlFlow, Instant};

    #[test]
    fn continue_ready() {
        assert!(ControlFlow::Continue.is_ready())
    }

    #[test]
    fn kill_unready() {
        assert!(!ControlFlow::Kill.is_ready());
    }

    #[test]
    fn wait_handle_readies_after_touch() {
        let (cf, wh) = ControlFlow::wait();
        assert!(!cf.is_ready());
        wh.wake();
        assert!(cf.is_ready());
    }

    #[test]
    fn sleep_until_readies_after_time() {
        let cf = ControlFlow::sleep_until(Instant::now() + Duration::from_millis(100));
        assert!(!cf.is_ready());
        MockClock::advance(Duration::from_millis(60));
        assert!(!cf.is_ready());
        MockClock::advance(Duration::from_millis(60));
        assert!(cf.is_ready());
    }

    #[test]
    fn sleep_for_readies_after_time() {
        let cf = ControlFlow::sleep_for(Duration::from_millis(100));
        assert!(!cf.is_ready());
        MockClock::advance(Duration::from_millis(60));
        assert!(!cf.is_ready());
        MockClock::advance(Duration::from_millis(60));
        assert!(cf.is_ready());
    }
}

/// An agent in the system, which can react to events.
///
/// Events are processed in 'rounds'. There's a list of 'current' events, which are fed into every actor at the same
/// time. Then all of the replies are collected, and those are the 'current' events for the next round.
///
/// As that implies, events are inherently ephemeral -- none persist more than one round.
pub trait Agent<M: Message>: Send + Sync {
    /// Called once on (re)start, to queue any starting events/ControlFlow as necessary. This will always be called
    /// before `react`.
    ///
    /// By default, does nothing and returns [`ControlFlow::Continue`] to allow [`Self::react`] to be called, under
    /// the assumption that your interesting code sits there.
    #[cfg_attr(coverage, no_coverage)]
    fn start(&mut self, _replies: &mut Replies<M>) -> ControlFlow {
        ControlFlow::Continue
    }

    /// React to the events of a round, indicating when the agent should be called next and optionally queueing some
    /// more events.
    ///
    /// Limitations on the [`Extend`] trait mean we just use the concrete type `Vec`. **Do not** do anything except
    /// pushing/extending/otherwise adding elements.
    ///
    /// By default, does nothing and returns [`ControlFlow::Kill`], under the assumption that you'd have implemented
    /// `react` if you wanted your agent to stay alive and do things.
    #[cfg_attr(coverage, no_coverage)]
    fn react(&mut self, _event: &M, _replies: &mut Replies<M>) -> ControlFlow {
        ControlFlow::Kill
    }
}
