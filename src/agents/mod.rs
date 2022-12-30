use std::{
    fmt::{self, Debug},
    sync::{
        Arc, Mutex,
    },
};

use crate::io::clifmt::Text;

pub mod tools;

mod cf;
pub use cf::{ControlFlow, WaitHandle};

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