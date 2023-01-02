use crate::{app::CliState, io::clifmt::Text, text, text1};

use super::{AutocompleteType, BsdCompleter, FixedOutput, Tool};

lazy_static::lazy_static! {
    static ref COMPLETER: BsdCompleter = BsdCompleter::new()
        .flag('l').argument('d', AutocompleteType::LocalFile);
}

pub struct Ls;

impl Ls {
    fn entries<'cs>(state: &CliState<'cs>) -> Vec<&'cs str> {
        // TODO: directory-specific
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

    fn list_short(width: usize, state: &CliState) -> Vec<Vec<Text>> {
        let entries = Self::entries(state);
        let (chunks, num_rows) = (0..)
            .filter_map(|rows| {
                let chunks = entries.chunks(rows);
                let widths = chunks
                    .clone()
                    .map(|items| items.iter().map(|s| s.len()).max().unwrap());
                let total = (widths.len() - 1) * 3 + widths.sum::<usize>();
                if total < width {
                    Some((chunks, rows))
                } else {
                    None
                }
            })
            .next()
            .expect("couldn't find a fitting row length");
        let mut rows = vec![text![]; num_rows];
        for chunk in chunks {
            let width = chunk.iter().map(|s| s.len()).sum();
            for (row, item) in chunk.iter().enumerate() {
                let text = if item.contains(char::is_whitespace) {
                    text1!("'{:0>1$}' "(item, width))
                } else {
                    text1!(" {:0>1$}  "(item, width))
                };
                rows[row].push(text);
            }
        }
        rows
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
        let rows = Self::list_short(80, state); // TODO: accommodate actual screen size
        Box::new(FixedOutput(rows))
    }
}
