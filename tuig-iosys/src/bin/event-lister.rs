use std::thread;

use tuig_iosys::{IoSystem, Screen, Action, text1, Key};

fn list_events(mut sys: Box<dyn IoSystem>) {
    const MAX_LEN: usize = 256;
    let mut log = vec![text1!["press escape to exit when you're done"]];
    let mut screen = Screen::new(sys.size());
    loop {
        match sys.input().expect("failed to get input") {
            Action::Closed | Action::KeyPress { key: Key::Escape } => break,
            Action::Error(e) => Err(e).expect("got an error for input"),
            other => log.push(text1!("{:?}\n"(other))),
        }
        screen.resize(sys.size());
        screen.textbox(log.clone())
            .first_indent(0)
            .indent(4)
            .scroll_bottom(true);
        sys.draw(&screen).expect("failed to render screen");
        if log.len() > MAX_LEN {
            let diff = log.len() - MAX_LEN;
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