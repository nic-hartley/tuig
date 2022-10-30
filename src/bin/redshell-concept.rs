use std::{collections::HashMap, env::args, io::{Write, stdout}, time::Duration, pin::Pin, future::Future};

use redshell::{io::{XY, output::{Color, Text, Screen}, input::{Action, Key}, sys::{IoSystem, self}}, text, app::{ChatApp, App}, GameState, event::Event};
use tokio::time::sleep;

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
            let text = Text::of(format!("{}", bg.name()))
                .fg(fg).bg(bg);
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
    // 2 second wait outside of this so wait 10s total
    sleep(Duration::from_secs(8)).await;
}

async fn intro(io: &mut dyn IoSystem) {
    let mut s = Screen::new(io.size());
    // TODO: Write the real function and use [`TimedStream::with_delays`] to drive it
    let frames: Vec<(Vec<(&str, usize)>, Vec<Text>, usize)> = vec![
        (vec![], text!(
            "??????????: Hey.\n",
            "??????????: You ever used Redshell before?\n",
            "> ", green underline "no", "  ", red "yes",
        ), 500),
        (vec![], text!(
            "??????????: Hey.\n",
            "??????????: You ever used Redshell before?\n",
            "> ", green underline "no", "  ", red "yes", bold "  (arrow keys to select, enter to submit)"
        ), 1000),
        (vec![], text!(
            "??????????: Hey.\n",
            "??????????: You ever used Redshell before?\n",
            "> ", green "no", "  ", red underline "yes",
        ), 500),
        (vec![], text!(
            "??????????: Hey.\n",
            "??????????: You ever used Redshell before?\n",
            "> ", green underline "no", "  ", red "yes",
        ), 250),
        (vec![], text!(
            "??????????: Hey.\n",
            "??????????: You ever used Redshell before?\n",
            "       you: no\n",
        ), 500),
        (vec![], text!(
            "??????????: Hey.\n",
            "??????????: You ever used Redshell before?\n",
            "       you: no\n",
            "??????????: Cool. Let me explain how it works, then. You familiar with a command line?\n",
            "> ", green "not at all", "  ", yellow underline "a little", "  ", red "intimately",
        ), 1000),
        (vec![], text!(
            "??????????: Hey.\n",
            "??????????: You ever used Redshell before?\n",
            "       you: no\n",
            "??????????: Cool. Let me explain how it works, then. You familiar with a command line?\n",
            "> ", green "not at all", "  ", yellow "a little", "  ", red underline "intimately",
        ), 100),
        (vec![], text!(
            "??????????: Hey.\n",
            "??????????: You ever used Redshell before?\n",
            "       you: no\n",
            "??????????: Cool. Let me explain how it works, then. You familiar with a command line?\n",
            "       you: intimately",
        ), 200),
        (vec![], text!(
            "??????????: Hey.\n",
            "??????????: You ever used Redshell before?\n",
            "       you: no\n",
            "??????????: Cool. Let me explain how it works, then. You familiar with a command line?\n",
            "       you: intimately\n",
            "??????????: Good, that'll make this easier. A moment...",
        ), 1000),
        (vec![], text!(
            "??????????: Hey.\n",
            "??????????: You ever used Redshell before?\n",
            "       you: no\n",
            "??????????: Cool. Let me explain how it works, then. You familiar with a command line?\n",
            "       you: intimately\n",
            "??????????: Good, that'll make this easier.\n",
            "??????????: This is the chat window.",
        ), 250),
        (vec![("chat", 0)], text!(
            "??????????: Hey.\n",
            "??????????: You ever used Redshell before?\n",
            "       you: no\n",
            "??????????: Cool. Let me explain how it works, then. You familiar with a command line?\n",
            "       you: intimately\n",
            "??????????: Good, that'll make this easier.\n",
            "??????????: This is the chat window.",
        ), 750),
        (vec![("chat", 0)], text!(
            "??????????: Hey.\n",
            "??????????: You ever used Redshell before?\n",
            "       you: no\n",
            "??????????: Cool. Let me explain how it works, then. You familiar with a command line?\n",
            "       you: intimately\n",
            "??????????: Good, that'll make this easier.\n",
            "??????????: This is the chat window.\n",
            "??????????: Everyone you talk you on Redshell? You'll talk through this. Nothing in person.",
        ), 1500),
        (vec![("chat", 0)], text!(
            "??????????: Hey.\n",
            "??????????: You ever used Redshell before?\n",
            "       you: no\n",
            "??????????: Cool. Let me explain how it works, then. You familiar with a command line?\n",
            "       you: intimately\n",
            "??????????: Good, that'll make this easier.\n",
            "??????????: This is the chat window.\n",
            "??????????: Everyone you talk you on Redshell? You'll talk through this. Nothing in person.\n",
            "??????????: Too dangerous.",
        ), 500),
        (vec![("chat", 0)], text!(
            "??????????: Hey.\n",
            "??????????: You ever used Redshell before?\n",
            "       you: no\n",
            "??????????: Cool. Let me explain how it works, then. You familiar with a command line?\n",
            "       you: intimately\n",
            "??????????: Good, that'll make this easier.\n",
            "??????????: This is the chat window.\n",
            "??????????: Everyone you talk you on Redshell? You'll talk through this. Nothing in person.\n",
            "??????????: Too dangerous.\n",
            "??????????: No real names, either. So call me Admin.",
        ), 250),
        (vec![("chat", 0)], text!(
            "     Admin: Hey.\n",
            "     Admin: You ever used Redshell before?\n",
            "       you: no\n",
            "     Admin: Cool. Let me explain how it works, then. You familiar with a command line?\n",
            "       you: intimately\n",
            "     Admin: Good, that'll make this easier.\n",
            "     Admin: This is the chat window.\n",
            "     Admin: Everyone you talk you on Redshell? You'll talk through this. Nothing in person.\n",
            "     Admin: Too dangerous.\n",
            "     Admin: No real names, either. So call me Admin.",
        ), 750),
    ];
    let XY(width, height) = s.size();
    for (tabs, frame, delay) in frames {
        if !tabs.is_empty() {
            let mut h = s.header();
            for (name, notifs) in tabs {
                h = h.tab(name, notifs);
            }
        }
        s.textbox(frame).pos(0, 1).size(width, height).indent(12).first_indent(0);
        io.draw(&s).await.unwrap();
        sleep(Duration::from_millis(delay as u64)).await;
    }
}

async fn chat_demo(io: &mut dyn IoSystem) {
    let mut s = Screen::new(io.size());

    let mut app = ChatApp::default();
    let state = GameState {
        player_name: "player".into(),
        apps: vec![],
    };
    let frames: Vec<(_, &[Action])> = vec![
        (vec![
            Event::npc_chat("alice", "hello there", &["hi", "hello", "sup"]),
        ], &[]),
        (vec![
            Event::npc_chat("bob", "so", &[]),
        ], &[
            Action::KeyPress { key: Key::Right },
        ]),
        (vec![
            Event::npc_chat("alice", "buddy", &["hi", "hello", "sup"]),
        ], &[
            Action::KeyPress { key: Key::Right },
        ]),
        (vec![], &[
            Action::KeyPress { key: Key::Enter },
        ]),
        (vec![
            Event::npc_chat("bob", "hi friend", &[]),
            Event::npc_chat("charlie", "asdfasdfasdfadsf", &[]),
            Event::npc_chat("charlie", "adskfljalksdjasldkf", &[]),
            Event::npc_chat("bob", "u up?", &["yes", "no"]),
        ], &[]),
        (vec![
            Event::npc_chat("alice", "so", &[]),
        ], &[
            Action::KeyPress { key: Key::Down },
        ]),
        (vec![
            Event::npc_chat("alice", "uh", &[]),
            Event::npc_chat("bob", "hello?", &["yes hello", "no goodbye"]),
            Event::npc_chat("alice", "what's the deal with airline tickets", &[]),
        ], &[]),
        (vec![], &[
            Action::KeyPress { key: Key::Up },
        ]),
    ];
    for (chats, inputs) in frames.into_iter() {
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
    loop {
        s.textbox(text!(invert "Press any keyboard button to exit"));
        let text;
        let at;
        match io.input().await.unwrap() {
            Action::KeyPress { .. } | Action::KeyRelease { .. } => break,
            Action::MousePress { button, pos } => {
                text = format!("{:?} button pressed at {:?}", button, pos);
                at = pos;
            }
            Action::MouseRelease { button, pos } => {
                text = format!("{:?} button released at {:?}", button, pos);
                at = pos;
            }
            Action::MouseMove { button: Some(b), pos } => {
                text = format!("Moved to {:?} holding {:?}", pos, b);
                at = pos;
            }
            Action::MouseMove { button: None, pos } => {
                text = format!("Moved to {:?} holding nothing", pos);
                at = pos;
            }
            Action::Unknown(desc) => {
                text = format!("Unknown input: {}", desc);
                at = XY(0, 0);
            }
            Action::Error(msg) => {
                text = format!("Error: {}", msg);
                at = XY(0, 0);
            }
        };
        s.textbox(text!("{}"(text))).xy(at);
        io.draw(&s).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    let concepts = {
        type ConceptFn = for<'a> fn(&'a mut dyn IoSystem) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
        let mut map: HashMap<&'static str, ConceptFn> = HashMap::new();
        map.insert("render", |s| Box::pin(render_demo(s)));
        map.insert("intro", |s| Box::pin(intro(s)));
        map.insert("chat", |s| Box::pin(chat_demo(s)));
        map.insert("mouse", |s| Box::pin(mouse_demo(s)));
        map
    };

    let mut args = args();
    let arg0 = args.next().expect("how did you have no argv[0]");
    if let Some(name) = args.next() {
        if let Some(func) = concepts.get(name.as_str()) {
            print!("Playing {}... ", name);
            stdout().flush().unwrap();
            {
                let mut iosys = sys::load();
                func(iosys.as_mut()).await;
                let XY(width, height) = iosys.size();
                let msg = "fin.";
                write!(stdout(), "\x1b[{};{}H\x1b[107;30m{}\x1b[0m", height, width - msg.len(), msg).unwrap();
                stdout().flush().unwrap();
                sleep(Duration::from_secs(2)).await;
            }
            println!(" Done.");
            return;
        }
    }
    println!("Show off some concept art, built on the actual UI toolkit of the game.");
    println!("Pass the name as the first parameter, i.e.:");
    println!("  {} <name>", arg0);
    println!();
    println!("Available concept art is:");
    for (k, _) in concepts {
        println!("- {}", k);
    }
}
