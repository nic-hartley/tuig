use std::{env, process, thread, time::Duration};

use redshell::{
    agents::Event,
    app::{App, ChatApp},
    io::{
        input::{Action, Key},
        output::{Color, FormattedExt, Screen, Text},
        sys::{self, IoRunner, IoSystem},
        XY,
    },
    text, GameState,
};
use tokio::time::{interval, sleep};

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
    texts.extend(text!(underline "underline", " ", bold "bold", "\n"));
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

    s.textbox(texts).pos(1, 2);
    s.header()
        .tab("tab", 1)
        .tab("tab", 2)
        .selected(1)
        .profile("watching the render concept")
        .time("the time is now");
    io.draw(&s).await.unwrap();

    sleep(Duration::from_secs(5)).await;
}

async fn intro_demo(io: &mut dyn IoSystem) {
    let size = io.size();
    redshell::cutscenes::intro(io, &mut Screen::new(size))
        .await
        .expect("Failed to run intro");
}

async fn chat_demo(io: &mut dyn IoSystem) {
    let mut s = Screen::new(io.size());

    let mut app = ChatApp::default();
    let state = GameState::for_player("player".into());
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
        for chat in chats {
            app.on_event(&chat);
        }
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
    s.textbox(text!(black on_white "Press any keyboard button to exit"));
    io.draw(&s).await.unwrap();
    let mut at = XY(0, 0);
    let mut render_tick = interval(Duration::from_secs_f32(0.01));
    let mut last_text = String::new();
    let mut text = String::new();
    loop {
        tokio::select! {
            _ = render_tick.tick() => {
                if text == last_text {
                    continue;
                }
                s.resize(io.size());
                s.textbox(text!(black on_white "Press any keyboard button to exit"));
                s.textbox(text!("{}"(text))).xy(at);
                io.draw(&s).await.unwrap();
                last_text = text.clone();
            }
            Ok(Some(act)) = io.flush() => {
                match act {
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
                    Action::Redraw => {
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
            }
        }
    }
}

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
    println!(
        r##"
Show off some concept art, built on the actual UI toolkit of the game.
Pass the name as the first parameter, i.e.:
    redshell-concept <name>

Available concept art is:
- render:   Basic tests of the UI widgets implemented
- intro:    The game's actual intro, separated into its own thing
- chat:     A self-playing demo of the chatroom
- mouse:    Showing off mouse interaction
"##
    );
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
