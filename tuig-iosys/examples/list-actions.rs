use std::thread;

use tuig_iosys::{fmt::Cell, Action, IoSystem, Key, Screen};

fn list_events(mut sys: Box<dyn IoSystem>) {
    let mut log = vec!["press escape to exit when you're done".to_owned()];
    let mut screen = Screen::new(sys.size());
    loop {
        screen.resize(sys.size());
        for (line, row) in log.iter().rev().zip((0..screen.size().y()).rev()) {
            for (char, col) in line.chars().zip(0..screen.size().x()) {
                screen[row][col] = Cell::of(char);
            }
        }
        sys.draw(&screen).expect("failed to render screen");
        match sys.input().expect("failed to get input") {
            Action::Closed | Action::KeyPress { key: Key::Escape } => break,
            Action::Error(e) => panic!("{1}: {:?}", e, "got an error for input"),
            other => log.push(format!("{:?}", other)),
        }
        if log.len() > screen.size().y() {
            let diff = log.len() - screen.size().y();
            log.drain(..diff);
        }
    }
    sys.stop();
}

fn main() {
    println!("loading...");
    {
        let (sys, mut run) = tuig_iosys::load().expect("failed to load any IO system");
        let main_loop = thread::spawn(move || list_events(sys));
        run.run();
        main_loop.join().expect("failed to run iosystem!");
    }
    println!("complete!");
}
