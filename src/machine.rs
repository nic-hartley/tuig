use std::collections::HashMap;

/// A single machine, somewhere in cyberspace. Possibly even the player's own.
#[derive(Default, Clone)]
pub struct Machine {
    /// The files on this machine
    // TODO: Cleaner abstraction for this
    pub files: HashMap<String, String>,
}
