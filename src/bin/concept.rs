use std::{collections::HashMap, env::args, io::{Write, stdout}, thread::sleep, time::Duration};

use redshell::{io::{Screen, Text, XY}, text};

// TODO: Any more convenient way to do 'frames' than this? Gotta be...

fn intro(s: &mut dyn Screen) {
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
        s.flush();
        sleep(Duration::from_millis(delay as u64));
    }
}

fn tabswitch(s: &mut dyn Screen) {
    s.textbox(text!(bold "TODO", ": tabswitch")).pos(1, 1).size(1000, 1000);
}

#[tokio::main]
async fn main() {
    let concepts = {
        let mut map: HashMap<&str, fn(&mut dyn Screen)> = HashMap::new();
        map.insert("intro", intro);
        map.insert("tabswitch", tabswitch);
        map
    };

    let mut args = args();
    let arg0 = args.next().expect("how did you have no argv[0]");
    if let Some(name) = args.next() {
        if let Some(func) = concepts.get(name.as_str()) {
            print!("Playing {}... ", name);
            stdout().flush().unwrap();
            {
                let mut screen = <dyn Screen>::get();
                func(screen.as_mut());
                let XY(width, height) = screen.size();
                let msg = "fin.";
                write!(stdout(), "\x1b[{};{}H\x1b[107;30m{}\x1b[0m", height, width - msg.len(), msg).unwrap();
                stdout().flush().unwrap();
                sleep(Duration::from_secs(2));
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
