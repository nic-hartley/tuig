//! Displays the chat window.

use std::{collections::BTreeSet, mem};

use crate::{
    agents::Event,
    constants::{gameplay::MAX_USERNAME, graphics::HEADER_HEIGHT},
    io::{
        clifmt::FormattedExt,
        input::{Action, Key},
        output::{Screen, Text},
    },
    text1, GameState,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Message {
    Normal { text: String, from_player: bool },
    System(String),
}

impl Message {
    fn from_player(text: String) -> Message {
        Message::Normal {
            text,
            from_player: true,
        }
    }

    fn from_npc(text: String) -> Message {
        Message::Normal {
            text,
            from_player: false,
        }
    }

    fn system(text: &str) -> Message {
        Message::System(text.into())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct DM {
    /// The name of the person we're having a conversation with
    target: String,
    /// The actual messages in this DM; (by_player, contents)
    msgs: Vec<Message>,
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

/// The direct message tab.
///
/// Allows the user to chat directly with NPCs, potentially with multiple conversations at a time.
#[derive(PartialEq, Eq, Clone, Debug, Default)]
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

    fn clear_current_unread(&mut self) {
        self.dms[self.current_dm].unread = 0;
    }
}

impl super::App for ChatApp {
    fn name(&self) -> &'static str {
        "chat"
    }

    fn input(&mut self, a: Action, events: &mut Vec<Event>) -> bool {
        let key = match a {
            Action::KeyPress { key, .. } => key,
            Action::MouseMove { .. } => return false, // TODO: Handle mouse move
            Action::MousePress { .. } => return false, // TODO: Handle mouse press
            _ => return false,
        };
        match key {
            Key::Left => {
                if self.dm().sel > 0 {
                    self.dms[self.current_dm].sel -= 1
                }
            }
            Key::Right => {
                if self.dm().sel < self.dm().options.len() - 1 {
                    self.dms[self.current_dm].sel += 1
                }
            }
            Key::Enter if !self.dm().options.is_empty() => {
                let dm = &mut self.dms[self.current_dm];
                let mut options = mem::replace(&mut dm.options, vec![]);
                let selected = options.remove(dm.sel);
                let ev = Event::player_chat(&dm.target, &selected);
                dm.msgs.push(Message::from_player(selected));
                dm.sel = 0;
                events.push(ev);
            }
            Key::Up if self.current_dm > 0 => {
                self.current_dm -= 1;
                self.clear_current_unread();
            }
            Key::Up => {}
            Key::Down if self.current_dm < self.dms.len() - 1 => {
                self.current_dm += 1;
                self.clear_current_unread();
            }
            _ => return false,
        };
        true
    }

    fn on_event(&mut self, ev: &Event) -> bool {
        let (sender, message, options) = match ev {
            Event::NPCChatMessage {
                from,
                text,
                options,
            } => (from, text, options),
            _ => return false,
        };
        if self.blocked.contains(sender) {
            return false;
        }
        match self.dms.iter_mut().find(|d| &d.target == sender) {
            Some(dm) => {
                dm.msgs.push(Message::from_npc(message.clone()));
                dm.unread += 1;
                dm.options = options.clone();
                dm.open = true;
            }
            None => self.dms.push(DM {
                msgs: vec![
                    Message::system("Chat started"),
                    Message::from_npc(message.clone()),
                ],
                options: options.clone(),
                sel: 0,
                target: sender.into(),
                unread: 1,
                open: true,
            }),
        }
        true
    }

    fn notifs(&self) -> usize {
        self.dms
            .iter()
            .filter(|dm| dm.open)
            .map(|dm| dm.unread)
            .sum()
    }

    fn render(&mut self, state: &GameState, screen: &mut Screen) {
        let size = screen.size();
        // The width of the side pane, including the vertical divider.
        let list_pane_size = (size.x() / 10).clamp(15, 30);

        if !self.dms.is_empty() {
            self.dms[self.current_dm].unread = 0;
            let dm = self.dm();
            // per message: 1 for name, 1 for colon, 1 for message contents, 1 for newline
            // plus 1 + 2 * current_dm.options.len() for the options line including spaces between
            // TODO: update messages to be Vec<Text>, and update this expression accordingly
            let chunks =
                dm.msgs.iter().map(|_| 1 + 1 + 1 + 1).sum::<usize>() + dm.options.len() * 2;
            let mut output = Vec::with_capacity(chunks);
            for msg in &dm.msgs {
                match msg {
                    Message::Normal { text, from_player } => {
                        let name = if *from_player {
                            text1!["{0:>1$}"(state.player_name, MAX_USERNAME)]
                        } else {
                            text1![cyan "{0:>1$}"(dm.target, MAX_USERNAME)]
                        };
                        output.push(name);
                        output.push(Text::plain(": "));
                        output.push(Text::plain(text));
                        output.push(Text::plain("\n"));
                    }
                    Message::System(text) => {
                        output.push(Text::of(text.clone()).bold());
                        output.push(Text::plain("\n"));
                    }
                }
            }
            for (i, opt) in dm.options.iter().enumerate() {
                if i == 0 {
                    output.push(text1!("{0:>1$}  "(">", MAX_USERNAME + 1)));
                } else {
                    output.push(text1!("   "));
                }
                if i == dm.sel {
                    output.push(text1!(underline "{}"(opt)));
                } else {
                    output.push(text1!("{}"(opt)));
                }
            }
            screen
                .textbox(output)
                .pos(0, HEADER_HEIGHT)
                .width(size.x() - list_pane_size);
        }

        screen
            .vertical(size.x() - list_pane_size)
            .start(HEADER_HEIGHT);
        let mut names = Vec::with_capacity(2 * self.dms.len());
        for (i, dm) in self.dms.iter().enumerate() {
            if !dm.open {
                continue;
            }
            if dm.unread == 0 {
                names.push(text1!("   "));
            } else if dm.unread > 9 {
                names.push(text1!(" + ").red());
            } else {
                names.push(text1!(red " {} "(dm.unread)));
            }
            if i == self.current_dm {
                names.push(text1!(underline "{}\n"(dm.target)))
            } else {
                names.push(text1!("{}\n"(dm.target)));
            }
        }
        screen
            .textbox(names)
            .pos(size.x() - list_pane_size + 1, HEADER_HEIGHT)
            .width(list_pane_size + 1)
            .scroll_bottom(true);
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

    const ENTER: Action = Action::KeyPress { key: Key::Enter };
    const LEFT: Action = Action::KeyPress { key: Key::Left };
    const RIGHT: Action = Action::KeyPress { key: Key::Right };

    macro_rules! assert_input {
        ($app:ident .input ( $($arg:expr),* $(,)? ) == $other:expr) => {
            {
                let mut evs = vec![];
                $app.input($( $arg ),* , &mut evs);
                assert_eq!(evs, $other);
            }
        };
        ($app:ident .input ( $($arg:expr),* $(,)? ) . $( $val:tt )*) => {
            {
                let mut evs = vec![];
                $app.input($( $arg ),* , &mut evs);
                assert!(evs.$($val)*);
            }
        };
    }

    #[test]
    fn test_submit_reply() {
        let mut app = app_dm(&["hello"], 0);
        assert_input!(app.input(ENTER) == &[Event::player_chat("targette", "hello")]);
    }

    #[test]
    fn test_select_submit() {
        let mut app = app_dm(&["hello", "goodbye"], 0);
        assert_input!(app.input(RIGHT).is_empty());
        assert_input!(app.input(ENTER) == &[Event::player_chat("targette", "goodbye")]);
    }

    #[test]
    fn test_select_hit_right() {
        let mut app = app_dm(&["hello", "goodbye"], 0);
        assert_input!(app.input(RIGHT).is_empty());
        assert_input!(app.input(RIGHT).is_empty());
        assert_input!(app.input(RIGHT).is_empty());
        assert_input!(app.input(ENTER) == &[Event::player_chat("targette", "goodbye")]);
    }

    #[test]
    fn test_select_hit_left() {
        let mut app = app_dm(&["hello", "goodbye"], 1);
        assert_input!(app.input(LEFT).is_empty());
        assert_input!(app.input(LEFT).is_empty());
        assert_input!(app.input(LEFT).is_empty());
        assert_input!(app.input(ENTER) == &[Event::player_chat("targette", "hello")]);
    }

    #[test]
    fn test_receive_option() {
        let mut app = app_dm(&[], 0);
        app.on_event(&Event::npc_chat("targette", "hello there", &["hi", "no"]));
        assert_input!(app.input(ENTER) == &[Event::player_chat("targette", "hi")]);
    }

    #[test]
    fn test_receive_next_option() {
        let mut app = app_dm(&[], 0);
        app.on_event(&Event::npc_chat("targette", "hello there", &["hi", "no"]));
        assert_input!(app.input(RIGHT).is_empty());
        assert_input!(app.input(ENTER) == &[Event::player_chat("targette", "no")]);
    }
}
