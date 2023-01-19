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
}

impl PartialEq for WaitHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for WaitHandle {}
impl fmt::Debug for WaitHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WaitHandle(...)")
    }
}

/// What should happen to an [`Agent`][super::Agent] after it finishes [react][super::Agent::react]ing to
/// [`Event`][super::Event]s.
///
/// Note that this only defines when [`Agent::react`][super::Agent::react] can start being called again -- if there
/// are no events availalbe, it may not actually be called! This should be rare in the actual game but it may happen
/// in tests.
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
