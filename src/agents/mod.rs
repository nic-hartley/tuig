pub mod tools;

mod cf;
pub use cf::{ControlFlow, WaitHandle};

mod event;
pub use event::{Bundle, BundledAgent, BundledApp, BundledTool, Event};

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

/// An agent which does nothing and immediately dies.
// Big mood, buddy.
pub struct NopAgent;
impl Agent for NopAgent {
    fn start(&mut self, _replies: &mut Vec<Event>) -> ControlFlow {
        ControlFlow::Kill
    }

    fn react(&mut self, _events: &[Event], _replies: &mut Vec<Event>) -> ControlFlow {
        ControlFlow::Kill
    }
}
