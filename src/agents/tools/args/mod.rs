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

use crate::app::CliState;

pub fn autocomplete_with(
    prefix: &str,
    options: impl IntoIterator<Item = impl AsRef<str>>,
) -> String {
    let mut completed: Option<String> = None;
    for opt in options.into_iter() {
        let opt = opt.as_ref();
        println!("- {}", opt);
        if !opt.starts_with(prefix) {
            println!("  doesn't start with prefix");
            continue;
        }
        let rest = &opt[prefix.len()..];
        println!("  rest: {}", rest);
        if let Some(prev) = &mut completed {
            println!("    updating prev: {}", prev);
            let eq_find = rest
                .chars()
                .zip(prev.chars())
                .enumerate()
                .find(|(_, (o, r))| o != r);
            let eq_idx = eq_find.map(|(i, _)| i).unwrap_or(prev.len());
            prev.truncate(eq_idx);
        } else {
            println!("    setting completed");
            completed = Some(rest.into());
        }
        println!("  res: {}", completed.as_ref().unwrap());
    }
    completed.unwrap_or(String::new())
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

    pub fn complete(&self, so_far: &str, state: &CliState) -> String {
        match self {
            Self::None => String::new(),
            Self::Choices(opts) => autocomplete_with(so_far, opts),
            Self::LocalFile => {
                println!("so_far={}, cwd={}", so_far, state.cwd);
                let mut dirs: Vec<_> = so_far.split('/').collect();
                let file = dirs.pop().expect("split never returns an empty iterator");
                let cmd_dir = dirs.into_iter().map(|s| format!("{}/", s)).collect::<String>();
                let prefix = format!("{}{}", state.cwd, cmd_dir);
                println!("prefix={}, file={}", prefix, file);
                autocomplete_with(
                    file,
                    state
                        .gs
                        .machine
                        .files
                        .keys()
                        .filter_map(|f| f.strip_prefix(&prefix))
                        .map(|f| f.split_inclusive('/').next().unwrap_or(f)),
                )
            },
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::GameState;

    use super::*;

    #[test]
    fn find_autocomplete_empty_for_empty_iter() {
        let opts: &[&str] = &[];
        assert_eq!(autocomplete_with("", opts), "");
        assert_eq!(autocomplete_with("a", opts), "");
    }

    #[test]
    fn find_autocomplete_presents_option() {
        let opts: &[&str] = &["abyss"];
        assert_eq!(autocomplete_with("", opts), "abyss");
        assert_eq!(autocomplete_with("a", opts), "byss");
        assert_eq!(autocomplete_with("aby", opts), "ss");
        assert_eq!(autocomplete_with("abyss", opts), "");
    }

    #[test]
    fn find_autocomplete_presents_conflicting_options() {
        let opts: &[&str] = &["abyss", "absolute", "gorgonzola"];
        assert_eq!(autocomplete_with("", opts), "");
        assert_eq!(autocomplete_with("a", opts), "b");
        assert_eq!(autocomplete_with("ab", opts), "");
        assert_eq!(autocomplete_with("aby", opts), "ss");
        assert_eq!(autocomplete_with("abs", opts), "olute");
        assert_eq!(autocomplete_with("g", opts), "orgonzola");
    }

    #[test]
    fn none_doesnt_autocomplete() {
        let mut gs = GameState::for_player("miso".into());
        gs.machine.files.insert("moo".into(), "".into());
        gs.machine.files.insert("abyss".into(), "".into());
        let clis = CliState {
            gs: &gs,
            cwd: "".into(),
        };
        let ac = AutocompleteType::None;
        assert_eq!(ac.complete("", &clis), "");
        assert_eq!(ac.complete("m", &clis), "");
        assert_eq!(ac.complete("mi", &clis), "");
        assert_eq!(ac.complete("mo", &clis), "");
    }

    #[test]
    fn choices_autocompletes_choices() {
        let mut gs = GameState::for_player("miso".into());
        gs.machine.files.insert("moo".into(), "".into());
        gs.machine.files.insert("abyss".into(), "".into());
        let clis = CliState {
            gs: &gs,
            cwd: "".into(),
        };
        let ac = AutocompleteType::choices(&["mass", "help", "gorgonzola"]);
        assert_eq!(ac.complete("", &clis), "");
        assert_eq!(ac.complete("m", &clis), "ass");
        assert_eq!(ac.complete("ma", &clis), "ss");
        assert_eq!(ac.complete("mi", &clis), "");
        assert_eq!(ac.complete("mo", &clis), "");
        assert_eq!(ac.complete("a", &clis), "");
        assert_eq!(ac.complete("g", &clis), "orgonzola");
    }

    #[test]
    fn local_file_autocompletes_local_files() {
        let mut gs = GameState::for_player("miso".into());
        gs.machine.files.insert("moo".into(), "".into());
        gs.machine.files.insert("abyss".into(), "".into());
        let clis = CliState {
            gs: &gs,
            cwd: "".into(),
        };
        let ac = AutocompleteType::LocalFile;
        assert_eq!(ac.complete("", &clis), "");
        assert_eq!(ac.complete("m", &clis), "oo");
        assert_eq!(ac.complete("mi", &clis), "");
        assert_eq!(ac.complete("mo", &clis), "o");
        assert_eq!(ac.complete("a", &clis), "byss");
        assert_eq!(ac.complete("abyss", &clis), "");
    }

    #[test]
    fn local_file_autocompletes_local_files_in_cwd() {
        let mut gs = GameState::for_player("miso".into());
        gs.machine.files.insert("moo".into(), "".into());
        gs.machine.files.insert("abyss".into(), "".into());
        gs.machine.files.insert("stuff/bongos".into(), "".into());
        gs.machine
            .files
            .insert("stuff/michael_hill".into(), "".into());
        gs.machine.files.insert("stuff/neil_baum".into(), "".into());
        let clis = CliState {
            gs: &gs,
            cwd: "stuff/".into(),
        };
        let ac = AutocompleteType::LocalFile;
        assert_eq!(ac.complete("", &clis), "");
        assert_eq!(ac.complete("m", &clis), "ichael_hill");
        assert_eq!(ac.complete("mi", &clis), "chael_hill");
        assert_eq!(ac.complete("mo", &clis), "");
        assert_eq!(ac.complete("a", &clis), "");
        assert_eq!(ac.complete("neil_bau", &clis), "m");
        assert_eq!(ac.complete("neil_baum", &clis), "");
    }

    #[test]
    fn local_file_autocompletes_directories_nicely() {
        let mut gs = GameState::for_player("miso".into());
        gs.machine.files.insert("moo".into(), "".into());
        gs.machine.files.insert("abyss".into(), "".into());
        gs.machine.files.insert("stuff/bongos".into(), "".into());
        let clis = CliState {
            gs: &gs,
            cwd: "".into(),
        };
        let ac = AutocompleteType::LocalFile;
        assert_eq!(ac.complete("", &clis), "");
        assert_eq!(ac.complete("st", &clis), "uff/");
        assert_eq!(ac.complete("stuff", &clis), "/");
        assert_eq!(ac.complete("stuff/", &clis), "bongos");
        assert_eq!(ac.complete("stuff/bo", &clis), "ngos");
    }
}
