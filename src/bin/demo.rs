//! Hastily slapped-together example binary showing off some aspect of the game or engine.

use redshell::{output::Screen, text, output::Text};

fn main() {
    let mut screen = <dyn Screen>::get();
    screen.header().tab("hello", 1).tab("there", 0).tab("notifs", 100).profile("someone").time("now!!!!");
    screen.textbox(text!(
        "This is a",
        bold "textbox",
        "! Do you ",
        yellow "like {}?"("it")
    )).pos(1, 2).first_indent(2);
    screen.flush();
}
