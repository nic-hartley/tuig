use core::fmt;
use std::sync::{Arc, Mutex};

use crate::{app::App, io::clifmt::Text, tools::Tool};

use super::Agent;

/// Convenience for the things that pass trait objects around, but only one of them.
pub struct Bundle<T>(Arc<Mutex<Option<T>>>);

impl<T> Bundle<T> {
    pub fn of(contents: T) -> Self {
        Self(Arc::new(Mutex::new(Some(contents))))
    }

    pub fn take(&self) -> Option<T> {
        self.0.lock().unwrap().take()
    }
}

impl<T> fmt::Debug for Bundle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bundle(..)")
    }
}

impl<T> PartialEq for Bundle<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Eq for Bundle<T> {}

impl<T> Clone for Bundle<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

macro_rules! trait_bundle {
    ( $(
        $fn:ident => $enum:ident($trait:ident)
    ),* $(,)? ) => { $(
        paste::paste! {
            pub type [< Bundled $trait >] = Bundle<Box<dyn $trait>>;
            impl [< Bundled $trait >] {
                pub fn new(contents: impl $trait + 'static) -> Self {
                    Bundle::of(Box::new(contents))
                }
            }
            impl Event {
                pub fn $fn(item: impl $trait + 'static) -> Self {
                    Self::$enum([< Bundled $trait >]::new(item))
                }
            }
        }
    )* };
}
trait_bundle! {
    spawn => SpawnAgent(Agent),
    install => InstallTool(Tool),
    add_tab => AddTab(App),
}

/// A single thing which has happened, which an [`Agent`] may or may not want to respond to.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Event {
    /// Have a new agent spawned and processing events
    SpawnAgent(BundledAgent),
    /// Create a new tab on the player's UI
    AddTab(BundledApp),

    /// Something added a command to the player's CLI
    InstallTool(BundledTool),
    /// A line of output from a running command
    CommandOutput(Vec<Text>),
    /// Command has changed the CLI's directory to the given (absolute) one
    ChangeDir(String),
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

    pub fn cd(to: &str) -> Event {
        Event::ChangeDir(to.into())
    }
}
