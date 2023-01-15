use std::mem;

use crate::{
    agents::{Agent, ControlFlow, Event},
    app::CliState,
    text,
};

use super::{AutocompleteType, FixedOutput, Tool};

struct CdAgent(String);
impl Agent for CdAgent {
    fn start(&mut self, replies: &mut Vec<crate::agents::Event>) -> ControlFlow {
        replies.push(Event::ChangeDir(mem::take(&mut self.0)));
        replies.push(Event::CommandDone);
        ControlFlow::Kill
    }
    fn react(&mut self, _events: &[Event], _replies: &mut Vec<Event>) -> ControlFlow {
        ControlFlow::Kill
    }
}

pub struct Cd;

impl Tool for Cd {
    fn name(&self) -> &'static str {
        "cd"
    }

    fn autocomplete(&self, line: &str, state: &CliState) -> String {
        AutocompleteType::LocalFile.complete(line, state)
    }

    fn run(&self, line: &str, state: &CliState) -> Box<dyn crate::agents::Agent> {
        let line = line.trim();
        let mut target_comps = if line.starts_with('/') {
            vec![]
        } else {
            state.cwd.split('/').filter(|s| !s.is_empty()).collect()
        };
        for component in line.split('/') {
            if component.is_empty() || component == "." {
                continue;
            } else if component == ".." {
                target_comps.pop();
            } else {
                target_comps.push(component);
            }
        }
        let res = if target_comps.is_empty() {
            "/".into()
        } else {
            format!("/{}/", target_comps.join("/"))
        };
        match state.machine.readdir(&res) {
            Ok(_) => Box::new(CdAgent(res)),
            Err(e) => Box::new(FixedOutput(vec![text![bright_red "ERROR", ": {}\n"(e)]])),
        }
    }
}
