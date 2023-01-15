use crate::{
    app::CliState,
    io::clifmt::{FormattedExt, Text},
    machine::Entry,
    text, text1,
};

use super::{AutocompleteType, BsdArgs, FixedOutput, Tool};

lazy_static::lazy_static! {
    static ref COMPLETER: BsdArgs = BsdArgs::new()
        .flag('l').argument('d', AutocompleteType::LocalFile);
}

pub struct Ls;

impl Ls {
    fn entries<'cs>(dir: &str, state: &'cs CliState) -> Result<Vec<(String, Entry)>, String> {
        let prefix = if dir.is_empty() {
            format!("{}", state.cwd)
        } else if dir.ends_with('/') {
            format!("{}{}", state.cwd, dir)
        } else {
            format!("{}{}/", state.cwd, dir)
        };
        let entries = match state.machine.readdir(&prefix) {
            Ok(e) => e,
            Err(e) => return Err(e),
        };
        let mut entries: Vec<_> = entries.collect();
        entries.sort_unstable_by(|l, r| l.0.cmp(&r.0));
        entries.dedup_by(|l, r| l.0 == r.0);
        Ok(entries)
    }

    fn list_short(dir: &str, state: &CliState) -> Vec<Vec<Text>> {
        let entries = match Self::entries(dir, state) {
            Ok(e) => e,
            Err(e) => {
                return vec![text![bright_red "ERROR", ": {}\n"(e)]];
            }
        };
        let mut line: Vec<_> = entries
            .into_iter()
            .map(|(name, entry)| {
                let text = if name.chars().any(char::is_whitespace) {
                    text1!["'{}' "(name)]
                } else {
                    text1![" {}  "(name)]
                };
                if entry.is_dir() {
                    text.cyan().bold()
                } else {
                    text
                }
            })
            .collect();
        if !line.is_empty() {
            line.push(text1!["\n"]);
        }
        vec![line]
    }

    fn list_long(dir: &str, state: &CliState) -> Vec<Vec<Text>> {
        let entries = match Self::entries(dir, state) {
            Ok(e) => e,
            Err(e) => {
                return vec![text![bright_red "ERROR", ": {}\n"(e)]];
            }
        };
        vec![text!["total {}\n"(entries.len())]]
            .into_iter()
            .chain(entries.into_iter().map(|(name, entry)| {
                let mut res = vec![Text::plain(""); 3];
                res[0] = match &entry {
                    Entry::File(f) => text1!["{} "(f.contents.len())],
                    Entry::Directory(d) => text1!["{} "(d.len())],
                };
                res[1] = if entry.is_dir() {
                    text1![cyan bold "{}"(name)]
                } else {
                    text1!["{}"(name)]
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
            Err(msg) => {
                return Box::new(FixedOutput(vec![text![bright_red "ERROR", ": {}\n"(msg)]]))
            }
        };
        let dir = args
            .get(&'d')
            .unwrap_or(&Some(""))
            .expect("None despite arg having value");
        let rows = if args.get(&'l').is_some() {
            Self::list_long(dir, state)
        } else {
            Self::list_short(dir, state)
        };
        Box::new(FixedOutput(rows))
    }
}
