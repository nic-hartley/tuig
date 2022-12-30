use std::{collections::HashMap, iter::repeat};

use crate::GameState;

use super::AutocompleteType;

/// Allows for easy completion of BSD-style commands, e.g.:
///
/// - `tar |` -> tries to complete x, f, etc. (the various option characters)
/// - `tar x|` -> the same thing
/// - `tar x |` -> doesn't try to complete anything; as far as it cares the optoins are done
/// - `tar xf |` -> tries to autocomplete a filename
/// - `tar xf abc|` -> tries to autocomplete a filename starting with `abc`
pub struct BsdCompleter {
    /// The options this completer can complete
    options: HashMap<char, Option<AutocompleteType>>,
}

impl BsdCompleter {
    pub fn new() -> Self {
        Self {
            options: Default::default(),
        }
    }

    pub fn flag(mut self, ch: char) -> Self {
        self.options.insert(ch, None);
        self
    }

    pub fn argument(mut self, ch: char, kind: AutocompleteType) -> Self {
        self.options.insert(ch, Some(kind));
        self
    }

    pub fn complete(&self, line: &str, state: &GameState) -> String {
        if let Some((opts, rest)) = line.split_once(' ') {
            // autocomplete arguments
            let mut kinds = opts
                .chars()
                .filter_map(|c| self.options.get(&c).map(|o| o.as_ref()).flatten())
                .chain(repeat(&AutocompleteType::None));
            let vals = rest.trim_start().split_whitespace();
            if rest.ends_with(char::is_whitespace) || rest.is_empty() {
                // correct for split_whitespace trimming the end
                for _ in 0..vals.count() {
                    kinds.next();
                }
                let last_kind = kinds
                    .next()
                    .expect("iter chained with infinite repeat ran out");
                last_kind.complete("", state)
            } else {
                match vals.zip(&mut kinds).last() {
                    Some((last_val, last_kind)) => last_kind.complete(last_val, state),
                    None => String::new(),
                }
            }
        } else {
            // autocomplete remaining options
            let mut remaining = self.options.keys().filter(|&&c| !line.contains(c)).fuse();
            let maybe_last = remaining.next();
            let maybe_after = remaining.next();
            match (maybe_last, maybe_after) {
                // there's exactly one remaining option
                (Some(last), None) => String::from(*last),
                // there's more than one, we can't complete
                (Some(_), Some(_)) => String::new(),
                // there's none left, we can't complete
                (None, None) => String::new(),
                // the iterator is fused, it shouldn't ever do this
                (None, Some(_)) => unreachable!(),
            }
        }
    }
}
