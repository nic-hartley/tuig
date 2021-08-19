use std::{collections::HashMap, env::args, thread::sleep, time::Duration};

use redshell::{output::{Screen, Text}, text};

fn intro(s: &mut dyn Screen) {
    s.textbox(text!(
        "Hey.\nYou ever used Redshell before?\n",
        green underline "no", "  ", red "yes",
    ));
    s.flush();
}

fn tabswitch(s: &mut dyn Screen) {
    s.textbox(text!(bold "TODO", ": tabswitch"));
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
            let mut screen = <dyn Screen>::get();
            func(screen.as_mut());
            screen.flush();
            sleep(Duration::from_secs(5));
            return;
        }
    }
    println!("Show off some concept art, built on the actual UI toolkit of the game.");
    println!("Pass the name as the first parameter, i.e.:");
    println!("  {} <concept>", arg0);
    println!();
    println!("Available concept art is:");
    for (k, _) in concepts {
        println!("- {}", k);
    }
}