use std::thread;

use tuig_iosys::{
    cell,
    fmt::Cell,
    text,
    ui::{
        cols,
        elements::{TextInput, Textbox},
        rows, Region, ScreenView,
    },
    Action, IoSystem, Key, Screen,
};

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

fn run(mut iosys: Box<dyn IoSystem>) {
    let mut ti = TextInput {
        prompt: "".into(),
        line: "abcde01234ABCDE)!@#$_".into(),
        cursor: 3,
        autocomplete: "".into(),
        history: Default::default(),
        histpos: 0,
        histcap: 0,
    };
    let mut tui = |region: Region| {
        let [l, m, r] = region.split(cols!(20 "| |" * "#" 5)).unwrap();
        let [t, b] = l.split(rows!(* "=" 1)).unwrap();
        for s in [m, r] {
            s.attach(|i, mut sv: ScreenView| sv.fill(char_for_input(&i)))
        }
        t.attach(|i, sv| {
            let txt = text![
                "Hello! Your most recent ", red "action", " was: ",
                bold green "{:?}"(i),
            ];
            Textbox::new(txt).render_to(sv)
        });
        b.attach(&mut ti);
        true
    };

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
        if matches!(
            input,
            Action::Closed | Action::KeyPress { key: Key::Escape }
        ) {
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
