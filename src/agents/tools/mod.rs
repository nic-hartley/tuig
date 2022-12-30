use crate::GameState;

use super::Agent;

mod completers;

/// Describes the various things that can be autocompleted. Used to indicate value types in the various autocompleters
/// and holds a little bit of common logic across all of them.
pub enum AutocompleteType {
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
    pub fn complete(&self, so_far: &str, _state: &GameState) -> String {
        match self {
            Self::Choices(opts) => {
                let mut res = String::new();
                for opt in opts {
                    if !opt.starts_with(so_far) {
                        continue;
                    }
                    let rest = &opt[so_far.len()..];
                    if res.is_empty() {
                        res = rest.into();
                        continue;
                    }
                    let eq_find = rest.chars().zip(res.chars()).enumerate().find(|(_, (o, r))| o != r);
                    let eq_idx = eq_find.map(|(i, _)| i).unwrap_or(0);
                    res.truncate(eq_idx);
                }
                res
            }
            _ => unimplemented!()
        }
    }
}

pub trait Tool {
    fn autocomplete(&self, line: &str) -> String;
    fn run(&self, line: &str) -> Box<dyn Agent>;
}
