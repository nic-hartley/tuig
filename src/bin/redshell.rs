use std::{mem, thread, time::Duration};

use redshell::{
    agents::{tools, Agent, ControlFlow, Event},
    app::{App, ChatApp, CliApp},
    io::{
        input::{Action, Key},
        output::Screen,
        sys::{self, IoSystem},
    },
};
use tokio::time::sleep;

struct ChatState {
    messages: Vec<(String, usize)>,
    options: Vec<(String, usize)>,
}

#[derive(Default)]
struct NPC {
    name: String,
    all_states: Vec<ChatState>,
    state: usize,
    message: usize,
}

impl NPC {
    fn state(&self) -> &ChatState {
        &self.all_states[*&self.state]
    }

    fn message(&self) -> &(String, usize) {
        &self.state().messages[*&self.message]
    }

    fn advance(&mut self, replies: &mut Vec<Event>) -> ControlFlow {
        if self.state >= self.all_states.len() {
            return ControlFlow::Kill;
        }
        let (text, delay) = self.message().clone();
        let mut options = vec![];
        if self.message == self.state().messages.len() - 1 {
            options = self
                .state()
                .options
                .iter()
                .map(|(s, _)| s.clone())
                .collect();
        }

        self.message += 1;
        replies.push(Event::NPCChatMessage {
            from: self.name.clone(),
            text,
            options,
        });
        ControlFlow::sleep_for(Duration::from_millis(delay as u64))
    }
}

impl Agent for NPC {
    fn start(&mut self, replies: &mut Vec<Event>) -> ControlFlow {
        self.advance(replies)
    }

    fn react(&mut self, events: &[Event], replies: &mut Vec<Event>) -> ControlFlow {
        if self.state >= self.all_states.len() {
            ControlFlow::Kill
        } else if self.message >= self.all_states[self.state].messages.len() {
            // look for a reply
            let mut cf = ControlFlow::Continue;
            for event in events {
                let (dest, text) = match event {
                    Event::PlayerChatMessage { to, text } => (to, text),
                    _ => continue,
                };
                if dest != &self.name {
                    continue;
                }
                let options = &self.all_states[self.state].options;
                let new_state = match options.iter().find(|(opt, _)| opt == text) {
                    Some((_, s)) => s,
                    None => continue,
                };
                self.state = *new_state;
                self.message = 0;
                cf = self.advance(replies);
                break;
            }
            cf
        } else {
            self.advance(replies)
        }
    }
}

macro_rules! npc {
    ( $name:literal, $(
        [
            $( say $msg:literal : $delay:literal ),* ,
            ask $( $option:literal => $state:literal ),* $(,)?
        ]
    ),* $(,)? ) => {
        Box::new(NPC {
            name: $name.into(),
            all_states: vec![ $(
                ChatState {
                    messages: vec![ $(
                        ( $msg.into(), $delay )
                    ),* ],
                    options: vec![
                        $( ( $option.into(), $state ) ),*
                    ],
                }
            ),* ],
            ..Default::default()
        })
    };
}

#[tokio::main]
async fn run(iosys: &mut dyn IoSystem) {
    // TODO: multithreading
    let mut screen = Screen::new(iosys.size());

    let mut apps: Vec<(Box<dyn App>, usize)> =
        vec![(Box::new(ChatApp::default()), 0), (Box::new(CliApp::default()), 0)];
    let mut sel = 0;
    let mut events = vec![
        Event::install(tools::Ls),
        Event::install(tools::Touch),
    ];
    let mut agents: Vec<(Box<dyn Agent>, ControlFlow)> = [npc!(
        "yotie",
        [
            say "hey": 500,
            say "hello": 500,
            say "hi": 500,
            say "my close personal friend": 1000,
            say "whose name I do not need to say": 1000,
            say "because we're so close and all": 1000,
            say "how you doin?": 1500,
            ask "good" => 1, "bad" => 2,
        ],
        [
            say "ey that's nice": 2000,
            say "glad you're doing well": 500,
            ask "thanks" =>  3,
        ],
        [
            say "ey that's bad": 2000,
            say "sucks you're doing meh": 500,
            ask "thanks?" =>  3,
        ],
        [
            say "anyway bye": 500,
            ask "uh ok" => 100,
        ]
    )]
    .into_iter()
    .map(|mut a| {
        let cf = a.start(&mut events);
        (a as Box<dyn Agent>, cf)
    })
    .collect();

    let mut replies = vec![];
    let mut new_agents: Vec<Box<dyn Agent>> = vec![];
    let mut tainted = true;
    loop {
        agents.extend(new_agents.drain(..).map(|mut a| {
            let cf = a.start(&mut replies);
            (a, cf)
        }));
        for (agent, cf) in agents.iter_mut().filter(|(_, cf)| cf.is_ready()) {
            *cf = agent.react(&events, &mut replies);
        }
        agents.retain(|(_, cf)| cf != &ControlFlow::Kill);
        for (i, (app, old_notifs)) in apps.iter_mut().enumerate() {
            for ev in &events {
                let ev_taint = app.on_event(ev);
                if i == sel {
                    tainted |= ev_taint;
                }
            }
            let new_notifs = app.notifs();
            tainted |= new_notifs != *old_notifs;
            *old_notifs = new_notifs;
        }
        for ev in &events {
            match ev {
                Event::SpawnAgent(b) => {
                    if let Some(ag) = b.take() {
                        new_agents.push(ag);
                    }
                }
                _ => (),
            }
        }
        mem::swap(&mut events, &mut replies);
        replies.clear();

        let new_size = iosys.size();
        if new_size != screen.size() {
            tainted = true;
        }

        if tainted {
            screen.resize(new_size);
            apps[sel].0.render(&Default::default(), &mut screen);
            {
                let mut header = screen
                    .header()
                    .profile("test thing")
                    .selected(sel)
                    .time("right now >:3");
                for (app, _) in &apps {
                    header = header.tab(app.name(), app.notifs());
                }
            }
            iosys.draw(&screen).await.unwrap();
            tainted = false;
        }

        tokio::select! {
            inp = iosys.input() => {
                match inp.unwrap() {
                    Action::KeyPress { key: Key::F(num) } => {
                        if num <= apps.len() {
                            sel = num as usize - 1;
                            tainted = true;
                        }
                    }
                    Action::Closed => break,
                    Action::Resized => tainted = true,

                    other => tainted |= apps[sel].0.input(other, &mut events),
                }
            }
            _ = sleep(Duration::from_millis(25)) => {
                // nothing to do here, we just want to make sure events are handled regularly
            }
        }
    }
    iosys.stop();
}

fn main() {
    let (mut iosys, mut iorun) = sys::load().expect("failed to load");
    thread::spawn(move || run(iosys.as_mut()));
    iorun.run()
}
