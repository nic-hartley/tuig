use redshell::{output::*, text};

fn main() {
    let mut screen = <dyn Screen>::get();
    screen.textbox(text!(
        "Hello there! This is some ",
        red "random",
        " text, I hope you enjoy it.\n",
        green "This is a short paragraph, here.\n",
        "This one is a good deal longer! Really, ",
        bold "quite",
        " long, actually! Almost distressingly so. Hm. Maybe I should stop??"
    ))
        .size(40, 3)
        .scroll(2)
        .pos(4, 2)
        .first_indent(3)
        .indent(0);
    screen.flush();
}