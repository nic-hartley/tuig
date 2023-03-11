use crate::{
    io::{
        clifmt::FormattedExt,
        output::{Screen, Text},
        XY,
    },
    text, text1,
};

/// The header at the top of the window, which lists tabs, the current time, etc.
///
/// e.g.:
/// ```text
/// chat 2 | terminal   | you are ...               12:34:56
/// ```
pub struct Header<'a> {
    pub(in super::super) screen: &'a mut Screen,
    pub(in super::super) tabs: Vec<(String, usize)>,
    pub(in super::super) selected: Option<usize>,
    pub(in super::super) profile: String,
    pub(in super::super) time: String,
}

impl<'a> Header<'a> {
    pub fn new(screen: &'a mut Screen) -> Self {
        Self {
            screen,
            tabs: Vec::with_capacity(5),
            selected: None,
            profile: "".into(),
            time: "".into(),
        }
    }

    pub fn tab(mut self, name: &str, notifs: usize) -> Self {
        self.tabs.push((name.into(), notifs));
        self
    }

    crate::util::setters! {
        profile(name: &str) => profile = name.into(),
        time(now: &str) => time = now.into(),
        selected(tab: usize) => selected = Some(tab),
    }
}

impl<'a> Drop for Header<'a> {
    fn drop(&mut self) {
        let mut text = Vec::with_capacity(self.tabs.len() * 3 + 1);
        for (i, (name, notifs)) in self.tabs.iter().enumerate() {
            match self.selected {
                Some(n) if n == i => text.push(Text::of(name.into()).underline()),
                _ => text.push(Text::of(name.into())),
            }
            if *notifs == 0 {
                text.push(text1!("   | "));
            } else if *notifs <= 9 {
                text.extend(text!(red " {} "(notifs), "| "));
            } else {
                text.extend(text!(red " + ", "| "));
            }
        }

        text.push(text1!("you are {}"(self.profile)));

        self.screen.write(XY(0, 0), text);
        let right_align = self.screen.size().x() - self.time.len();
        self.screen
            .write(XY(right_align, 0), text!["{}"(self.time)]);
    }
}
