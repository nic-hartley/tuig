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
    fn entries<'cs>(dir: &str, state: &CliState<'cs>) -> Vec<&'cs str> {
        let prefix = if dir.is_empty() {
            format!("{}", state.cwd)
        } else if dir.ends_with('/') {
            format!("{}{}", state.cwd, dir)
        } else {
            format!("{}{}/", state.cwd, dir)
        };
        let mut entries: Vec<_> = state
            .gs
            .machine
            .files
            .keys()
            .filter_map(|f| f.strip_prefix(&prefix))
            .map(|f| f.split_inclusive('/').next().unwrap_or(&f))
            .collect();
        if entries.is_empty() {
            return vec![];
        }
        entries.sort_unstable();
        entries.dedup();
        entries
    }

    fn list_short(dir: &str, state: &CliState) -> Vec<Vec<Text>> {
        let mut line: Vec<_> = Self::entries(dir, state)
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

    fn list_long(dir: &str, state: &CliState) -> Vec<Vec<Text>> {
        let entries = Self::entries(dir, state);
        vec![text!["total {}\n"(entries.len())]]
            .into_iter()
            .chain(entries.into_iter().map(|entry| {
                let mut res = vec![Text::plain(""); 3];
                res[0] = text1!["{} "(entry.len())];
                res[1] = if entry.ends_with('/') {
                    text1![cyan bold "{}"(entry)]
                } else {
                    text1!["{}"(entry)]
                };
                res[2] = text1!["\n"];
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

    fn run(&self, line: &str, state: &CliState) -> Box<dyn crate::agents::Agent> {
        let args = match COMPLETER.parse(line) {
            Ok(v) => v,
            Err(msg) => return Box::new(FixedOutput(vec![text![bright_red "ERROR", ": {}\n"(msg)]])),
        };
        let dir = args.get(&'d').unwrap_or(&Some("")).expect("None despite arg having value");
        let rows = if args.get(&'l').is_some() {
            Self::list_long(dir, state)
        } else {
            Self::list_short(dir, state)
        };
        Box::new(FixedOutput(rows))
    }
}
