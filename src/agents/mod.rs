use std::{
    fmt::{self, Debug},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

use crate::io::clifmt::Text;

/// Convenience for [`Event::SpawnAgent`], to manually implement the relevant traits without having to implement it
/// for the enture enum.
///
/// Importantly, there's only ever at most one pointed-to `dyn Agent`
#[derive(Clone)]
pub struct BundledAgent(Arc<Mutex<Option<Box<dyn Agent>>>>);

impl BundledAgent {
    pub fn new(agent: impl Agent + 'static) -> Self {
        Self(Arc::new(Mutex::new(Some(Box::new(agent)))))
    }

    pub fn take(self) -> Option<Box<dyn Agent>> {
        self.0.lock().unwrap().take()
    }
}

impl fmt::Debug for BundledAgent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BundledAgent(..)")
    }
}

impl PartialEq for BundledAgent {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for BundledAgent {}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Event {
    /// Have a new agent spawned and processing events
    SpawnAgent(BundledAgent),

    /// A line of output from a running command
    CommandOutput(Vec<Text>),
    /// The command that was running is done and the prompt can reappear.
    ///
    /// Note this doesn't kill the agent or stop more output from coming; it just tells the console to display the
    /// prompt for the next command. (This allows commands to run in the 'background'.)
    CommandDone,

    /// The player has sent a chat message to some NPC
    PlayerChatMessage { to: String, text: String },
    /// Some NPC has sent a chat message to the player
    NPCChatMessage {
        from: String,
        text: String,
        options: Vec<String>,
    },
}

impl Event {
    pub fn spawn(agent: impl Agent + 'static) -> Self {
        Self::SpawnAgent(BundledAgent::new(agent))
    }

    pub fn output(line: Vec<Text>) -> Self {
        Self::CommandOutput(line)
    }

    pub fn player_chat(to: &str, text: &str) -> Event {
        Event::PlayerChatMessage {
            to: to.into(),
            text: text.into(),
        }
    }

    pub fn npc_chat(from: &str, text: &str, options: &[&str]) -> Event {
        Event::NPCChatMessage {
            from: from.into(),
            text: text.into(),
            options: options.iter().map(|&s| s.to_owned()).collect(),
        }
    }
}

/// See [`ControlFlow::Handle`].
#[derive(Clone)]
pub struct WaitHandle(Arc<AtomicBool>);

impl WaitHandle {
    fn new() -> Self {
        WaitHandle(Arc::new(AtomicBool::new(false)))
    }

    /// Notify the waiting agent that it can wake up.
    pub fn wake(&self) {
        self.0.store(true, Ordering::Release);
    }

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

/// What should happen to an [`Agent`] after it finishes [react][Agent::react]ing to [`Event`]s.
///
/// Note that this only defines when [`Agent::react`] can start being called again -- if there are no events
/// availalbe, it may not actually be called! This should be rare in the actual game but it may happen in tests.
#[derive(PartialEq, Eq, Clone)]
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

/// An agent in the system, which can react to events.
///
/// Events are processed in 'rounds'. There's a list of 'current' events, which are fed into every actor at the same
/// time. Then all of the replies are collected, and those are the 'current' events for the next round.
///
/// As that implies, events are inherently ephemeral -- none persist more than one round.
pub trait Agent: Send + Sync {
    /// Called once on (re)start, to queue any starting events/ControlFlow as necessary. This will always be called
    /// before `react` is ever called. By default, does nothing and returns [`ControlFlow::Continue`], so that
    /// [`Self::react`] will be called on the next tick.
    fn start(&mut self, _replies: &mut Vec<Event>) -> ControlFlow {
        ControlFlow::Continue
    }

    /// React to the events of a round, indicating when the agent should be called next and optionally queueing some
    /// more events.
    ///
    /// Limitations on the [`Extend`] trait mean we just use the concrete type `Vec`. **Do not** do anything except
    /// pushing/extending/otherwise adding elements.
    fn react(&mut self, events: &[Event], replies: &mut Vec<Event>) -> ControlFlow;
}

#[cfg(test)]
mod cf_test {
    use std::{
        thread::sleep,
        time::{Duration, Instant},
    };

    use super::ControlFlow;

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
        sleep(Duration::from_millis(60));
        assert!(!cf.is_ready());
        sleep(Duration::from_millis(60));
        assert!(cf.is_ready());
    }

    #[test]
    fn sleep_for_readies_after_time() {
        let cf = ControlFlow::sleep_for(Duration::from_millis(100));
        assert!(!cf.is_ready());
        sleep(Duration::from_millis(60));
        assert!(!cf.is_ready());
        sleep(Duration::from_millis(60));
        assert!(cf.is_ready());
    }
}
