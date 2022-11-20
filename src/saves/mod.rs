//! Contains the various save systems implemented here.
//! 
//! This is separate from the [`IoSystem`][crate::io] stuff because a variety of different IO systems will use the
//! same saving and loading mechanisms.
//! 
//! This module contains the common data structures. The serialization and deserialization is left up to individual
//! modules. However, they all implement the two-phase loading and crash-resistant saving through the [`SaveSystem`]
//! trait -- see its documentation for more details.
//! 
//! The serialization and deserialization itself is mostly done by `serde` and various backends for it.

pub mod fs;

use std::fmt;

use chrono::{DateTime, Local, TimeZone};

use crate::GameState;

/// The high-level data about the save file, used for displaying in lists of available save files.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Metadata {
    /// The user-specified name of this save file
    pub name: String,
    /// When this save was created (Unix timestamp -- number of seconds since 1970.)
    pub created: i64,
    /// A game-generated string describing the current game state
    pub progress: String,
}

impl Metadata {
    pub fn created(&self) -> DateTime<Local> {
        Local.timestamp_millis_opt(self.created).unwrap()
    }
}

/// Points to a single save location and allows loading from or saving to that slot. Note this only saves/loads the
/// actual game data; metadata is handled in [`SaveSystem`].
#[async_trait::async_trait]
pub trait SaveHandle {
    type System: SaveSystem;

    /// Load a specific save, based on the handle returned from a previous [`Self::list`] called on the same object.
    /// 
    /// Using a handle from another object will probably trigger a crash, but it must not cause undefined behavior.
    async fn load(self) -> Result<GameState, <Self::System as SaveSystem>::Error>;

    /// Create a new save for the given game state.
    async fn save(self, data: &GameState) -> Result<(), <Self::System as SaveSystem>::Error>;
}

/// A standard interface to all of the saving and loading systems.
/// 
/// Saving and loading are bundled into the same trait because it makes no sense to try to save with one system, then
/// load with another.
/// 
/// This API expects your workflow to start by listing available saves, then overwriting/loading/creating one. That's
/// the workflow used by the saves menu.
/// 
/// The exception is [`quicksave`][Self::quicksave]s.
#[async_trait::async_trait]
pub trait SaveSystem {
    /// Points to one save slot, for saving to and loading from.
    type Handle: SaveHandle;

    /// Type of the errors returned if something goes wrong. Must implement `Debug` for detailed (log/dev) information
    /// and `Dispplay` for user-facing explanations.
    type Error: fmt::Debug + fmt::Display;

    /// List all of the available saves which could be loaded or overwritten.
    /// 
    /// This only returns an error if there was an error with opening a containing resource, e.g. a directory. Errors
    /// in loading any individual save file's metadata mean that particular entry is just silently omitted. Use
    /// [`Self::list_verbose`] for per-item errors.
    /// 
    /// By default this is implemented by filtering down [`Self::list_verbose`].
    async fn list(&self) -> Result<Vec<(Metadata, Self::Handle)>, Self::Error> {
        // rust is magic
        self.list_verbose().await.map(|v| v.into_iter().filter_map(Result::ok).collect())
    }

    /// List all of the available saves which could be loaded or overwritten, returning errors instead of ignoring
    /// them. Otherwise identical to [`Self::list`].
    async fn list_verbose(&self) -> Result<Vec<Result<(Metadata, Self::Handle), Self::Error>>, Self::Error>;

    /// Create a new save slot to store a save file into.
    async fn new_slot(&self, metadata: Metadata) -> Result<Self::Handle, Self::Error>;

    /// Return a handle to where the quicksave can be loaded from or saved to.
    /// 
    /// Quicksaves are ephemeral, so this should return the same value every time it's called, rather than creating a
    /// new quicksave slot.
    async fn quicksave(&self) -> Result<Self::Handle, Self::Error>;

    /// Perform any necessary cleanup, especially quicksaves.
    async fn cleanup(self) -> Result<(), Self::Error>;
}
