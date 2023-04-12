use std::thread;

use tuig_iosys::{ui::Region, IoSystem, Screen, Action, Key, fmt::Cell};

fn tui<'s>(mut region: Region<'s>) -> bool {
    let input = format!("{:?}", region.input);
    let nth = input.chars().nth(8).unwrap_or('x');
    let cell = Cell::of(nth);
    region.fill(cell);
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
    let (iosys, mut iorun) = tuig_iosys::load()
        .expect("failed to load any IO system(s)");
    let handle = thread::spawn(move || run(iosys));
    iorun.run();
    handle.join().expect("failed to run thread");
    println!("done!");
}
