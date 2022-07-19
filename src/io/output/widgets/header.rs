use crate::{io::{output::{Screen, Text}, XY}, text1, text};

pub struct Header<'a> {
    pub(in super::super) screen: &'a mut dyn Screen,
    pub(in super::super) tabs: Vec<(String, usize)>,
    pub(in super::super) selected: Option<usize>,
    pub(in super::super) profile: String,
    pub(in super::super) time: String,
}

impl<'a> Header<'a> {
    /// Add a tab to the header being rendered this frame
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

crate::util::abbrev_debug! {
    Header<'a>;
    if tabs != vec![],
    write selected,
    write profile,
    write time,
}

impl<'a> Drop for Header<'a> {
    fn drop(&mut self) {
        let mut text = Vec::with_capacity(self.tabs.len() * 3 + 1);
        let mut width = 0;
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
            width += name.len() + 5; // 5 for " n | "
        }

        text.push(text1!("you are {}"(self.profile)));
        width += 8 + self.profile.len();

        // this weird construction ensures that, if we manually highlight the header, the whole line gets highlighted
        // and doesn't have any weird gaps.
        let space_left = self.screen.size().x() - width;
        text.push(text1!("{:>1$}"(self.time, space_left)));
        self.screen.write_raw(text, XY(0, 0));
    }
}
