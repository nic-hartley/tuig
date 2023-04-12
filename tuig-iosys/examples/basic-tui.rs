use std::thread;

use tuig_iosys::{cell, fmt::Cell, ui::Region, Action, IoSystem, Key, Screen};

fn char_for_input(action: &Option<Action>) -> Cell {
    match action {
        None => cell!(red '~'),
        Some(Action::KeyPress { .. }) => cell!(green 'K'),
        Some(Action::KeyRelease { .. }) => cell!(blue 'K'),
        Some(Action::MouseMove { .. }) => cell!(red 'm'),
        Some(Action::MousePress { .. }) => cell!(green 'M'),
        Some(Action::MouseRelease { .. }) => cell!(blue 'M'),
        _ => cell!(white on_black '~'),
    }
}

fn tui<'s>(region: Region<'s>) -> bool {
    let (mut left, mut rest) = region.split_left(10);
    left.fill(char_for_input(&left.input));
    rest.fill(char_for_input(&rest.input));
    true
}

fn run(mut iosys: Box<dyn IoSystem>) {
    let mut screen = Screen::new(iosys.size());
    let mut input = None;
    loop {
        screen.resize(iosys.size());
        let root = Region::new(&mut screen, input);
        if !tui(root) {
            break;
        }
        iosys.draw(&screen).expect("failed to render output");
        let action = iosys.input().expect("failed to get input");
        if matches!(action, Action::KeyPress { key: Key::Escape }) {
            break;
        }
        input = Some(action);
    }
    iosys.stop();
}

fn main() {
    println!("loading...");
    let (iosys, mut iorun) = tuig_iosys::load().expect("failed to load any IO system(s)");
    let handle = thread::spawn(move || run(iosys));
    iorun.run();
    handle.join().expect("failed to run thread");
    println!("done!");
}
