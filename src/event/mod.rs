pub enum Event {
    ChatMessage {
        from: String,
        text: String,
        options: Vec<String>,
    },
    /// This is only here so that `let` isn't irrefutable. Delete once there's other events.
    Placeholder,
}

impl Event {
    pub fn chat(from: &str, text: &str, options: &[&str]) -> Event {
        Event::ChatMessage {
            from: from.into(),
            text: text.into(),
            options: options.iter().map(|&s| s.to_owned()).collect()
        }
    }
}
