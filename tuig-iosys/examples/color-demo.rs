use std::{iter::repeat, thread, time::Duration};

use tuig_iosys::{
    fmt::{Cell, Color, Formatted, FormattedExt},
    Action, IoSystem, Key, Screen, XY,
};

fn color_demo(mut sys: Box<dyn IoSystem>) {
    let color_width = Color::all()
        .into_iter()
        .map(|c| format!("{:?}", c).len())
        .max()
        .unwrap();

    let mut lines: Vec<Vec<Cell>> = Color::all()
        .into_iter()
        .map(|bg| {
            let mut line = vec![];
            for fg in Color::all() {
                let text = format!("{:?} on {1:<2$} ", fg, format!("{:?}", bg), color_width);
                line.extend(text.chars().map(move |c| Cell::of(c).fg(fg).bg(bg)));
            }
            line
        })
        .collect();
    let width: usize = lines.iter().map(|l| l.len()).max().unwrap() + 5;
    for line in &mut lines {
        let bg = line[0].get_fmt().bg;
        line.extend(repeat(Cell::of(' ').bg(bg)).take(width - line.len()));
    }
    let mut screen = Screen::new(XY(0, 0));
    let mut pos = 0;
    let mut moving = true;
    'main: loop {
        while let Some(action) = sys.poll_input().unwrap() {
            match action {
                Action::Closed | Action::KeyPress { key: Key::Escape } => break 'main,
                Action::KeyPress {
                    key: Key::Char(' '),
                } => moving = !moving,
                _ => (),
            }
        }
        if moving || sys.size() != screen.size() {
            screen.resize(sys.size());
            for row in 0..screen.size().y() {
                let o_row = (row + pos) % lines.len();
                for col in 0..screen.size().x() {
                    let o_col = (col + pos * 3) % width;
                    screen[row][col] = lines[o_row][o_col].clone();
                }
            }
            sys.draw(&screen).expect("failed to render screen");
        }
        if moving {
            pos += 1;
            thread::sleep(Duration::from_millis(250));
        } else {
            thread::sleep(Duration::from_millis(50));
        }
    }
    sys.stop();
}

fn main() {
    println!("loading...");
    {
        let (sys, mut run) = tuig_iosys::load().expect("failed to load any IO system");
        let main_loop = thread::spawn(move || color_demo(sys));
        run.run();
        main_loop.join().expect("failed to run iosystem!");
    }
    println!("complete!");
}
