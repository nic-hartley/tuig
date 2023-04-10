use std::thread;

use tuig_iosys::{ui::Region, IoSystem, Screen, Action, Key};

fn tui<'s>(region: Region<'s>) -> bool {
    true
}

fn run(mut iosys: Box<dyn IoSystem>) {
    let mut screen = Screen::new(iosys.size());
    let mut input = None;
    loop {
        let root = Region::new(&mut screen, input);
        if !tui(root) {
            break;
        }
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
