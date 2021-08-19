use std::{collections::HashMap, env::args, io::{Write, stdout}, thread::sleep, time::Duration};

use redshell::{output::{Screen, Text, XY}, text};

fn intro(s: &mut dyn Screen) {
    let (width, height) = s.size().tuple();
    s.textbox(text!(
        "Hey.\nYou ever used Redshell before?\n",
        green underline "no", "  ", red "yes",
    )).size(width, height);
    s.flush();
}

fn tabswitch(s: &mut dyn Screen) {
    s.textbox(text!(bold "TODO", ": tabswitch")).pos(1, 1).size(1000, 1000);
}

fn main() {
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
                let (width, height) = screen.size().tuple();
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