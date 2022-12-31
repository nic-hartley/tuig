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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_completer_doesnt_complete_no_options() {
        let completer = BsdCompleter::new();
        let mut gs = GameState::for_player("miso".into());
        gs.files.insert("moo".into(), vec![]);
        gs.files.insert("maggot".into(), vec![]);
        gs.files.insert("abyss".into(), vec![]);
        assert_eq!(completer.complete("", &gs), "");
        assert_eq!(completer.complete("m", &gs), "");
        assert_eq!(completer.complete("f", &gs), "");
        assert_eq!(completer.complete("f m", &gs), "");
        assert_eq!(completer.complete("z", &gs), "");
        assert_eq!(completer.complete("z c", &gs), "");
        assert_eq!(completer.complete("vzf", &gs), "");
    }

    #[test]
    fn bsd_completer_completes_options() {
        let completer = BsdCompleter::new()
            .flag('v')
            .flag('q')
            .argument('z', AutocompleteType::choices(["compress", "decompress"]))
            .argument('f', AutocompleteType::LocalFile);
        let mut gs = GameState::for_player("miso".into());
        gs.files.insert("moo".into(), vec![]);
        gs.files.insert("maggot".into(), vec![]);
        gs.files.insert("abyss".into(), vec![]);
        assert_eq!(completer.complete("", &gs), "");
        assert_eq!(completer.complete("v", &gs), "");
        assert_eq!(completer.complete("vf", &gs), "");
        assert_eq!(completer.complete("vfz", &gs), "q");
        assert_eq!(completer.complete("vfq", &gs), "z");
        assert_eq!(completer.complete("qvz", &gs), "f");
    }

    #[test]
    fn bsd_completer_completes_values() {
        let completer = BsdCompleter::new()
            .flag('v')
            .flag('q')
            .argument('z', AutocompleteType::choices(["compress", "decompress"]))
            .argument('f', AutocompleteType::LocalFile);
        let mut gs = GameState::for_player("miso".into());
        gs.files.insert("moo".into(), vec![]);
        gs.files.insert("maggot".into(), vec![]);
        gs.files.insert("abyss".into(), vec![]);
        assert_eq!(completer.complete("", &gs), "");
        assert_eq!(completer.complete("f", &gs), "");
        assert_eq!(completer.complete("f ", &gs), "");
        assert_eq!(completer.complete("f a", &gs), "byss");
        assert_eq!(completer.complete("f ma", &gs), "ggot");
        assert_eq!(completer.complete("z", &gs), "");
        assert_eq!(completer.complete("z ", &gs), "");
        assert_eq!(completer.complete("z d", &gs), "ecompress");
        assert_eq!(completer.complete("z comp", &gs), "ress");
    }

    #[test]
    fn bsd_completer_skips_flag_values() {
        let completer = BsdCompleter::new()
            .flag('v')
            .flag('q')
            .argument('z', AutocompleteType::choices(["compress", "decompress"]))
            .argument('f', AutocompleteType::LocalFile);
        let mut gs = GameState::for_player("miso".into());
        gs.files.insert("moo".into(), vec![]);
        gs.files.insert("maggot".into(), vec![]);
        gs.files.insert("abyss".into(), vec![]);
        assert_eq!(completer.complete("qf", &gs), "");
        assert_eq!(completer.complete("fv ", &gs), "");
        assert_eq!(completer.complete("vf a", &gs), "byss");
        assert_eq!(completer.complete("fq ma", &gs), "ggot");
        assert_eq!(completer.complete("vqz ", &gs), "");
        assert_eq!(completer.complete("zqv d", &gs), "ecompress");
        assert_eq!(completer.complete("qvz comp", &gs), "ress");
    }
}
