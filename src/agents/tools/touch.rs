use crate::{app::CliState, text};

use super::{FixedOutput, Tool, AutocompleteType};

pub struct Touch;

impl Tool for Touch {
    fn name(&self) -> &'static str {
        "touch"
    }

    fn autocomplete(&self, line: &str, state: &CliState) -> String {
        if line.ends_with(char::is_whitespace) {
            AutocompleteType::LocalFile.complete("", state)
        } else if let Some(last) = line.rsplit(char::is_whitespace).next() {
            AutocompleteType::LocalFile.complete(last, state)
        } else {
            String::new()
        }
    }

    fn run(&self, line: &str, state: &CliState) -> Box<dyn crate::agents::Agent> {
        let mut lines = vec![];
        for file in line.split_whitespace() {
            if let Err(e) = state.machine.write(file, String::new()) {
                lines.push(text![bright_red "ERROR", ": failed to write {}: {}"(file, e)]);
            }
        }
        Box::new(FixedOutput(lines))
    }
}

