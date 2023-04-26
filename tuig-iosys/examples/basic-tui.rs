use std::thread;

use tuig_iosys::{
    cell,
    fmt::Cell,
    ui::{cols, Region},
    Action, IoSystem, Key, Screen,
};
use tuig_pm::rows;

fn char_for_input(action: &Action) -> Cell {
    match action {
        Action::KeyPress { .. } => cell!(green 'K'),
        Action::KeyRelease { .. } => cell!(blue 'K'),
        Action::MouseMove { .. } => cell!(red 'm'),
        Action::MousePress { .. } => cell!(green 'M'),
        Action::MouseRelease { .. } => cell!(blue 'M'),
        _ => cell!(white on_black '~'),
    }
}

fn tui(region: Region) -> bool {
    let [l, m, r] = region.split(cols!(10 "| |" * "#" 5)).unwrap();
    let [t, b] = l.split(rows!(15 "x" *)).unwrap();
    for s in [m, r, t, b] {
        // TODO: s.fill(char_for_input(&s.input))
    }
    true
}

fn run(mut iosys: Box<dyn IoSystem>) {
    let mut screen = Screen::new(iosys.size());
    let mut input = Action::Redraw;
    loop {
        screen.resize(iosys.size());
        let root = Region::new(&mut screen, input);
        if !tui(root) {
            break;
        }
        iosys.draw(&screen).expect("failed to render output");
        input = iosys.input().expect("failed to get input");
        if matches!(input, Action::KeyPress { key: Key::Escape }) {
            break;
        }
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
