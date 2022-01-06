#[derive(Debug, Clone, PartialEq, Eq, )]
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
    /// This is only here so that `let` isn't irrefutable. Delete once there's other events.
    Placeholder,
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
