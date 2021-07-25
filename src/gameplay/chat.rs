use std::{collections::HashMap, fmt::Write as _, ops::Shr, time::{Duration, SystemTime}};

use crate::{events::{Event, Events}, systems::Systems};

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub from: String,
    pub to: String,
    pub msg: String,
    pub time: SystemTime,
}

impl Event for ChatMessage {
    fn complete(&self) -> bool { SystemTime::now() > self.time }
}

pub struct ChatApp {
    chats: HashMap<String, Vec<String>>,
}

impl ChatApp {
    pub fn new() -> ChatApp {
        ChatApp {
            chats: HashMap::new()
        }
    }
}

impl crate::apps::App for ChatApp {
    fn recv(&mut self, event: &Events) -> bool {
        match event {
            Events::ChatMessage(cm) if cm.to == "player" => {
                if let Some(msgs) = self.chats.get_mut(&cm.from) {
                    msgs.push(cm.msg.clone());
                } else {
                    self.chats.insert(cm.from.clone(), vec![cm.msg.clone()]);
                }
                true
            }
            _ => false,
        }
    }

    fn input(&mut self, data: String) -> Vec<Events> {
        if data.is_empty() {
            vec![]
        } else if let Some((to, msg)) = data.split_once(": ") {
            vec![
                Events::ChatMessage(ChatMessage {
                    from: "player".into(),
                    to: to.into(),
                    msg: msg.into(),
                    time: SystemTime::now(),
                })
            ]
        } else {
            let msg = format!("Hello, {}!", &data);
            vec![
                Events::AddSystem(Systems::ChatSystem(ChatSystem::with(data.clone()))),
                Events::ChatMessage(ChatMessage {
                    from: "player".into(),
                    to: data,
                    msg,
                    time: SystemTime::now() + Duration::from_millis(50)
                })
            ]
        }
    }

    fn render(&self, into: &mut String) {
        into.clear();
        write!(into, "Messages:\n").unwrap();
        for (name, msgs) in &self.chats {
            write!(into, " {}:\n", name).unwrap();
            for msg in msgs {
                write!(into, "  {}\n", msg).unwrap();
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChatSystem {
    name: String,
}

impl ChatSystem {
    pub fn with(name: String) -> Self {
        Self { name }
    }
}

impl crate::systems::System for ChatSystem {
    fn recv(&mut self, event: &Events) -> Vec<Events> {
        let cm = match event {
            Events::ChatMessage(cm) => cm,
            _ => return vec![],
        };
        if cm.to == "player" {
            vec![]
        } else if cm.to == self.name {
            vec![Events::ChatMessage(ChatMessage {
                from: self.name.clone(),
                to: cm.from.clone(),
                msg: format!("Replying to {}", cm.msg),
                time: SystemTime::now() + Duration::from_millis(250),
            })]
        } else {
            let rand = (&String::new() as *const _ as u64)
                // one phase of musl's rand() LCG (https://git.musl-libc.org/cgit/musl/tree/src/prng/rand.c)
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1)
                .shr(33);
            if rand % 4u64 == 0u64 {
                vec![Events::ChatMessage(ChatMessage {
                    from: self.name.clone(),
                    to: cm.from.clone(),
                    msg: format!("Commenting on {}", cm.msg),
                    time: SystemTime::now() + Duration::from_millis(1000),
                })]
            } else {
                vec![]
            }
        }
    }
}
