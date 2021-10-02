//! Displays the chat window.

use std::{collections::BTreeSet, mem};

use crate::{GameState, io::{Action, Key, Screen}};

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct DM {
    /// The name of the person we're having a conversation with
    target: String,
    /// The actual messages in this DM; (by_player, contents)
    msgs: Vec<(bool, String)>,
    /// How many messages have piled up since the player last looked at this chat
    unread: usize,
    /// The current options available in this DM
    // TODO: Replace this with something real
    options: Vec<String>,
    /// Which option is selected (will be zero if there's no options)
    sel: usize,
    /// Whether the DM is still open or whether they've closed it
    open: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChatApp {
    /// The DMs the player has going
    dms: Vec<DM>,
    /// Which DM they're currently looking at
    current_dm: usize,
    /// The other people this one has blocked
    blocked: BTreeSet<String>,
}

impl ChatApp {
    fn dm(&self) -> &DM {
        &self.dms[self.current_dm]
    }
}

impl super::App for ChatApp {
    fn name(&self) -> &'static str {
        "chat"
    }

    fn input(&mut self, a: Action) -> Vec<String> {
        let key = match a {
            Action::KeyPress { key, .. } => key,
            _ => return vec![],
        };
        match key {
            Key::Left => {
                if self.dm().sel > 0 {
                    self.dms[self.current_dm].sel -= 1
                }
                vec![]
            }
            Key::Right => {
                if self.dm().sel < self.dm().options.len() - 1 {
                    self.dms[self.current_dm].sel += 1
                }
                vec![]
            }
            Key::Enter => {
                if self.dm().options.is_empty() {
                    vec![]
                } else {
                    let dm = &mut self.dms[self.current_dm];
                    let mut options = mem::replace(&mut dm.options, vec![]);
                    let selected = options.remove(dm.sel);
                    let res = format!("{}:{}", dm.target, selected);
                    dm.msgs.push((true, selected));
                    vec![res]
                }
            }
            _ => vec![],
        }
    }

    fn on_event(&mut self, evs: &[String]) {
        for ev in evs {
            let (sender, rest) = ev.split_once(':').unwrap();
            if self.blocked.contains(sender) {
                continue;
            }
            let (message, options) = rest.split_once(':').unwrap();
            let options = options.split(',').map(|s| s.to_string()).collect();
            match self.dms.iter_mut().find(|d| d.target == sender) {
                Some(dm) => {
                    dm.msgs.push((false, message.into()));
                    dm.options = options;
                    dm.open = true;
                }
                None => {
                    self.dms.push(DM {
                        msgs: vec![(false, message.into())],
                        options,
                        sel: 0,
                        target: sender.into(),
                        unread: 1,
                        open: true,
                    })
                }
            }
        }
    }

    fn notifs(&self) -> usize {
        self.dms.iter().filter(|dm| dm.open).map(|dm| dm.unread).sum()
    }

    fn render(&self, state: &GameState, screen: &dyn Screen) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::app::App;

    #[allow(unused_imports)]
    use super::*;

    #[allow(unused)]
    fn app() -> ChatApp {
        ChatApp {
            dms: vec![],
            current_dm: 0,
            blocked: BTreeSet::new(),
        }
    }

    #[allow(unused)]
    fn dm() -> DM {
        DM {
            msgs: vec![],
            target: "targette".into(),
            unread: 0,
            options: vec![],
            sel: 0,
            open: true,
        }
    }

    #[allow(unused)]
    fn app_dm(options: &[&str], sel: usize) -> ChatApp {
        ChatApp {
            dms: vec![DM {
                options: options.iter().map(|s| s.to_string()).collect(),
                ..dm()
            }],
            ..app()
        }
    }

    const ENTER: Action = Action::KeyPress {
        key: Key::Enter,
        alt: false,
        ctrl: false,
        shift: false,
    };

    const LEFT: Action = Action::KeyPress {
        key: Key::Left,
        alt: false,
        ctrl: false,
        shift: false,
    };

    const RIGHT: Action = Action::KeyPress {
        key: Key::Right,
        alt: false,
        ctrl: false,
        shift: false,
    };

    #[test]
    fn test_submit_reply() {
        let mut app = app_dm(&["hello"], 0);
        assert_eq!(app.input(ENTER), &["targette:hello"]);
    }

    #[test]
    fn test_select_submit() {
        let mut app = app_dm(&["hello", "goodbye"], 0);
        assert!(app.input(RIGHT).is_empty());
        assert_eq!(app.input(ENTER), &["targette:goodbye"]);
    }

    #[test]
    fn test_select_hit_right() {
        let mut app = app_dm(&["hello", "goodbye"], 0);
        assert!(app.input(RIGHT).is_empty());
        assert!(app.input(RIGHT).is_empty());
        assert!(app.input(RIGHT).is_empty());
        assert_eq!(app.input(ENTER), &["targette:goodbye"]);
    }

    #[test]
    fn test_select_hit_left() {
        let mut app = app_dm(&["hello", "goodbye"], 1);
        assert!(app.input(LEFT).is_empty());
        assert!(app.input(LEFT).is_empty());
        assert!(app.input(LEFT).is_empty());
        assert_eq!(app.input(ENTER), &["targette:hello"]);
    }

    #[test]
    fn test_receive_option() {
        let mut app = app_dm(&[], 0);
        app.on_event(&["targette:hello there:hi,no".into()]);
        assert_eq!(app.input(ENTER), &["targette:hi"]);
    }

    #[test]
    fn test_receive_next_option() {
        let mut app = app_dm(&[], 0);
        app.on_event(&["targette:hello there:hi,no".into()]);
        assert!(app.input(RIGHT).is_empty());
        assert_eq!(app.input(ENTER), &["targette:no"]);
    }
}
