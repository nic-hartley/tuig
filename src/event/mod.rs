#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Event {
    PlayerChatMessage {
        to: String,
        text: String,
    },
    NPCChatMessage {
        from: String,
        text: String,
        options: Vec<String>,
    },
}

impl Event {
    pub fn player_chat(to: &str, text: &str) -> Event {
        Event::PlayerChatMessage {
            to: to.into(),
            text: text.into(),
        }
    }

    pub fn npc_chat(from: &str, text: &str, options: &[&str]) -> Event {
        Event::NPCChatMessage {
            from: from.into(),
            text: text.into(),
            options: options.iter().map(|&s| s.to_owned()).collect(),
        }
    }
}
