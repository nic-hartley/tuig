//! Hastily slapped-together example binary showing off some aspect of the game or engine.

use std::{thread::sleep, time::Duration};

use redshell::{io::{Screen, Text}, text};

fn main() {
    let mut screen = <dyn Screen>::get();
    screen.textbox(text!(
        "This is a ",
        bold "textbox",
        "! Do you ",
        yellow "like {}?"("it")
    )).pos(1, 2).first_indent(2).size(15, 1000);
    screen.vertical(20);
    screen.horizontal(30);
    screen.header().tab("hello", 1).tab("there", 0).tab("notifs", 100).profile("someone").time("now!!!!");
    screen.flush();
    sleep(Duration::from_secs(5));
}
