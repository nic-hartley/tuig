//! Standardized option completers, which are used by a lot of commands.
//!
//! Demonstrationg completions is also pretty hard, because where the cursor is changes things.
//! The format used in this module's documentation is:
//!
//! - `command text|` -> explanation of what happens
//!
//! where `|` is the cursor's location when you press tab.
//! There'll also be an explanation of the command, or it'll be a common \*nix one.

mod gnu;
// pub use gnu::GnuCompleter;
mod bsd;
pub use bsd::BsdCompleter;

use crate::GameState;

fn find_autocomplete(so_far: &str, options: impl IntoIterator<Item=impl AsRef<str>>) -> String {
    let mut res: Option<String> = None;
    for opt in options.into_iter() {
        let opt = opt.as_ref();
        if !opt.starts_with(so_far) {
            continue;
        }
        let rest = &opt[so_far.len()..];
        if let Some(prev) = &mut res {
            let eq_find = rest
                .chars()
                .zip(prev.chars())
                .enumerate()
                .find(|(_, (o, r))| o != r);
            let eq_idx = eq_find.map(|(i, _)| i).unwrap_or(0);
            prev.truncate(eq_idx);
        } else {
            res = Some(rest.into());
        }
    }
    res.unwrap_or(String::new())
}

/// Describes the various things that can be autocompleted. Used to indicate value types in the various autocompleters
/// and holds a little bit of common logic across all of them.
#[derive(Debug)]
pub enum AutocompleteType {
    /// This cannot be autocompleted
    None,
    /// One of a fixed list of choices
    Choices(Vec<String>),
    /// A file on the local machine
    LocalFile,
    /// A known hostname
    Hostname,
    /// A file somewhere on a remote machine, i.e. `hostname:filepath`
    RemoteFile,
    /// The name of another tool
    Tool,
    /// A full other command, e.g. for chaining commands
    Command,
}

impl AutocompleteType {
    pub fn choices(vals: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        Self::Choices(vals.into_iter().map(|s| s.as_ref().to_owned()).collect())
    }

    pub fn complete(&self, so_far: &str, state: &GameState) -> String {
        match self {
            Self::None => String::new(),
            Self::Choices(opts) => find_autocomplete(so_far, opts),
            Self::LocalFile => find_autocomplete(so_far, state.files.keys()),
            _ => unimplemented!(),
        }
    }
}
