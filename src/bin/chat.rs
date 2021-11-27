//! Just runs the chat window, using some simple random dialogue and random-ish inputs to demo how it works.
//! Mostly to demo how the renderer works.

use std::{thread::sleep, time::Duration};

use redshell::{GameState, app::{App, ChatApp}, io::{Action, Key, Screen}, text};

fn main() {
    let mut screen: Box<dyn Screen> = <dyn Screen>::get();
    let mut app = ChatApp::default();
    let state = GameState {
        player_name: "player".into(),
        apps: vec![],
    };
    let frames: Vec<(&[&'static str], &[Action], usize)> = vec![
        (&[
            "alice:hello there:hi,hello,sup",
        ], &[], 1000),
        (&[], &[
            Action::KeyPress { key: Key::Right, ctrl: false, alt: false, shift: false },
        ], 1000),
        (&[], &[
            Action::KeyPress { key: Key::Right, ctrl: false, alt: false, shift: false },
        ], 1000),
        (&[], &[
            Action::KeyPress { key: Key::Enter, ctrl: false, alt: false, shift: false },
        ], 1000),
        (&[
            "bob:hi friend:",
            "charlie:asdfasdfasdfadsf:",
            "charlie:adskfljalksdjasldkf:",
            "bob:u up?:yes,no",
        ], &[], 1000),
        (&[
            "alice:so:",
        ], &[
            Action::KeyPress { key: Key::Down, ctrl: false, alt: false, shift: false },
        ], 1000),
        (&[
            "alice:uh:",
            "bob:hello?:yes hello,no goodbye",
            "alice:what's the deal with airline tickets:",
        ], &[], 1000),
        (&[], &[
            Action::KeyPress { key: Key::Up, ctrl: false, alt: false, shift: false },
        ], 1000),
    ];
    for (chats, inputs, delay_ms) in frames {
        for chat in chats.into_iter() {
            app.on_event(&[chat.to_string()]);
        }
        for input in inputs.into_iter() {
            app.input(input.clone());
        }
        app.render(&state, Box::as_mut(&mut screen));
        screen.textbox(text!(
            "This is a ", bold "demo", " of the chatbox. No input necessary."
        ))
            .pos(0, 0)
            .height(1);
        screen.flush();
        sleep(Duration::from_millis(delay_ms as u64));
    }
    sleep(Duration::from_secs(1));
}
