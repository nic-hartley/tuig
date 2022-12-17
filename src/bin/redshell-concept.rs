use std::{
    env,
    time::Duration, process, thread,
};

use redshell::{
    app::{App, ChatApp},
    event::Event,
    io::{
        input::{Action, Key},
        output::{Color, FormattedExt, Screen, Text},
        sys::{self, IoSystem, IoRunner},
        XY,
    },
    text, GameState,
};
use tokio::time::sleep;

pub fn load_or_die() -> (Box<dyn IoSystem>, Box<dyn IoRunner>) {
    let errs = match sys::load() {
        Ok(pair) => return pair,
        Err(e) => e,
    };

    if errs.is_empty() {
        println!("No renderers enabled! Someone compiled this wrong.")
    } else {
        println!("{} renderers tried to load:", errs.len());
        for (name, err) in errs {
            println!("- {}: {:?}", name, err);
        }
    }

    std::process::exit(1);
}

async fn render_demo(io: &mut dyn IoSystem) {
    let mut s = Screen::new(io.size());
    s.horizontal(1);
    s.vertical(0);
    let mut texts = Vec::new();
    for fg in Color::all() {
        texts.push(Text::of(format!("{} on:\n", fg.name())));
        let amt = Color::all().len();
        const LINES: usize = 2;
        for (i, bg) in IntoIterator::into_iter(Color::all()).enumerate() {
            let text = Text::of(format!("{}", bg.name())).fg(fg).bg(bg);
            texts.push(text);
            if i % (amt / LINES) == amt / LINES - 1 {
                texts.push(Text::plain("\n"));
            } else if i < amt - 1 {
                texts.push(Text::plain(" "));
            }
        }
    }

    texts.extend(text!("\n", underline "underline", " ", bold "bold", " ", invert "invert", " "));

    s.textbox(texts).pos(1, 2);
    s.header()
        .tab("tab", 1)
        .tab("tab", 2)
        .selected(1)
        .profile("watching the render concept")
        .time("the time is now");
    io.draw(&s).await.unwrap();
}

async fn intro_demo(io: &mut dyn IoSystem) {
    redshell::cutscenes::intro(io)
        .await
        .expect("Failed to run intro");
}

async fn chat_demo(io: &mut dyn IoSystem) {
    let mut s = Screen::new(io.size());

    let mut app = ChatApp::default();
    let state = GameState {
        player_name: "player".into(),
        apps: vec![],
    };
    let frames: Vec<(_, &[Action])> = vec![
        (
            vec![Event::npc_chat(
                "alice",
                "hello there",
                &["hi", "hello", "sup"],
            )],
            &[],
        ),
        (
            vec![Event::npc_chat("bob", "so", &[])],
            &[Action::KeyPress { key: Key::Right }],
        ),
        (
            vec![Event::npc_chat("alice", "buddy", &["hi", "hello", "sup"])],
            &[Action::KeyPress { key: Key::Right }],
        ),
        (vec![], &[Action::KeyPress { key: Key::Enter }]),
        (
            vec![
                Event::npc_chat("bob", "hi friend", &[]),
                Event::npc_chat("charlie", "asdfasdfasdfadsf", &[]),
                Event::npc_chat("charlie", "adskfljalksdjasldkf", &[]),
                Event::npc_chat("bob", "u up?", &["yes", "no"]),
            ],
            &[],
        ),
        (
            vec![Event::npc_chat("alice", "so", &[])],
            &[Action::KeyPress { key: Key::Down }],
        ),
        (
            vec![
                Event::npc_chat("alice", "uh", &[]),
                Event::npc_chat("bob", "hello?", &["yes hello", "no goodbye"]),
                Event::npc_chat("alice", "what's the deal with airline tickets", &[]),
            ],
            &[],
        ),
        (vec![], &[Action::KeyPress { key: Key::Up }]),
    ];
    for (chats, inputs) in frames.into_iter() {
        s.resize(io.size());
        app.on_event(&chats);
        for input in inputs.into_iter() {
            let mut _events = vec![];
            app.input(input.clone(), &mut _events);
        }
        app.render(&state, &mut s);
        s.textbox(text!(
            "This is a ", bold red "demo", " of the chatbox. No input necessary."
        ))
        .pos(0, 0)
        .height(1);
        io.draw(&s).await.unwrap();
        sleep(Duration::from_millis(1000)).await;
    }
    sleep(Duration::from_secs(1)).await;
}

async fn mouse_demo(io: &mut dyn IoSystem) {
    let mut s = Screen::new(io.size());
    s.textbox(text!(invert "Press any keyboard button to exit"));
    io.draw(&s).await.unwrap();
    let mut at = XY(0, 0);
    loop {
        let text;
        match io.input().await.unwrap() {
            Action::KeyPress { .. } | Action::KeyRelease { .. } | Action::Closed => break,
            Action::MousePress { button } => {
                text = format!("{:?} button pressed", button);
            }
            Action::MouseRelease { button } => {
                text = format!("{:?} button released", button);
            }
            Action::MouseMove { pos } => {
                text = format!("Moved to {:?}", pos);
                at = pos;
            }
            Action::Resized => {
                text = format!("Window resized");
            }
            Action::Paused => {
                text = format!("Application refuses to pause");
            }
            Action::Unpaused => {
                text = format!("Application was unpaused anyway");
            }
            Action::Unknown(desc) => {
                text = format!("Unknown input: {}", desc);
            }
            Action::Error(msg) => {
                text = format!("Error: {}", msg);
            }
        };
        s.resize(io.size());
        s.textbox(text!(invert "Press any keyboard button to exit"));
        s.textbox(text!("{}"(text))).xy(at);
        io.draw(&s).await.unwrap();
    }
}

// #[tokio::main]
// async fn main() {
//     let concepts = {
//         type ConceptFn = for<'a> fn(&'a mut dyn IoSystem) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
//         let mut map: HashMap<&'static str, ConceptFn> = HashMap::new();
//         map.insert("render", |s| Box::pin(render_demo(s)));
//         map.insert("intro", |s| Box::pin(intro_demo(s)));
//         map.insert("chat", |s| Box::pin(chat_demo(s)));
//         map.insert("mouse", |s| Box::pin(mouse_demo(s)));
//         map
//     };

//     let mut args = args();
//     let arg0 = args.next().expect("how did you have no argv[0]");
//     if let Some(name) = args.next() {
//         if let Some(func) = concepts.get(name.as_str()) {
//             print!("Playing {}... ", name);
//             stdout().flush().unwrap();
//             {
//                 let (mut iosys, mut runner) = load_or_die().await;
//                 tokio::spawn(async move {
//                     func(iosys.as_mut()).await;
//                     iosys.stop();
//                 });
//                 runner.run()
//             }
//             println!(" Done.");
//             return;
//         }
//     }
//     println!("Show off some concept art, built on the actual UI toolkit of the game.");
//     println!("Pass the name as the first parameter, i.e.:");
//     println!("  {} <name>", arg0);
//     println!();
//     println!("Available concept art is:");
//     for (k, _) in concepts {
//         println!("- {}", k);
//     }
// }

#[tokio::main]
async fn run_concept(name: &str, iosys: &mut dyn IoSystem) {
    match name {
        "render" => render_demo(iosys).await,
        "intro" => intro_demo(iosys).await,
        "chat" => chat_demo(iosys).await,
        "mouse" => mouse_demo(iosys).await,
        _ => println!("Not a valid option"),
    };
    iosys.stop();
}

fn help() -> ! {
    println!(r##"
Show off some concept art, built on the actual UI toolkit of the game.
Pass the name as the first parameter, i.e.:
    redshell-concept <name>

Available concept art is:
- render:   Basic tests of the UI widgets implemented
- intro:    The game's actual intro, separated into its own thing
- chat:     A self-playing demo of the chatroom
- mouse:    Showing off mouse interaction
"##);
    process::exit(0)
}

fn main() {
    let mut args = env::args().skip(1);
    let concept = match args.next() {
        Some(c) => c,
        None => help(),
    };

    let (mut iosys, mut runner) = load_or_die();

    thread::spawn(move || run_concept(&concept, iosys.as_mut()));

    runner.run();
}
