use crate::{
    app::CliState,
    io::clifmt::{FormattedExt, Text},
    text, text1,
};

use super::{AutocompleteType, BsdCompleter, FixedOutput, Tool};

lazy_static::lazy_static! {
    static ref COMPLETER: BsdCompleter = BsdCompleter::new()
        .flag('l').argument('d', AutocompleteType::LocalFile);
}

pub struct Ls;

impl Ls {
    fn entries<'cs>(state: &CliState<'cs>) -> Vec<&'cs str> {
        let mut entries: Vec<_> = state
            .gs
            .machine
            .files
            .keys()
            .filter(|f| f.starts_with(&state.cwd))
            .map(|f| f.split_inclusive('/').next().unwrap_or(&f))
            .collect();
        if entries.is_empty() {
            return vec![];
        }
        entries.sort_unstable();
        entries.dedup();
        entries
    }

    fn list_short(state: &CliState) -> Vec<Vec<Text>> {
        let mut line: Vec<_> = Self::entries(state)
            .into_iter()
            .map(|item| {
                let text = if item.chars().any(char::is_whitespace) {
                    text1!["'{}' "(item)]
                } else {
                    text1![" {}  "(item)]
                };
                if item.ends_with('/') {
                    text.cyan().bold()
                } else {
                    text
                }
            })
            .collect();
        line.push(text1!["\n"]);
        vec![line]
    }

    fn list_long(state: &CliState) -> Vec<Vec<Text>> {
        let entries = Self::entries(state);
        vec![text!["total {}"(entries.len())]]
            .into_iter()
            .chain(entries.into_iter().map(|entry| {
                let mut res = vec![Text::plain(""); 2];
                res[0] = text1!["{} "(entry.len())];
                res[1] = if entry.ends_with('/') {
                    text1![bright_blue "{}"(entry)]
                } else {
                    text1!["{}"(entry)]
                };
                res
            }))
            .collect()
    }
}

impl Tool for Ls {
    fn name(&self) -> &'static str {
        "ls"
    }

    fn autocomplete(&self, line: &str, state: &CliState) -> String {
        COMPLETER.complete(line, state)
    }

    fn run(&self, _line: &str, state: &CliState) -> Box<dyn crate::agents::Agent> {
        let rows = Self::list_short(state);
        Box::new(FixedOutput(rows))
    }
}
