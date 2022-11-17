use std::{io, time::Duration};

use crate::{io::{sys::IoSystem, output::Screen, text::Text, input::{Action, Key}, widgets::Textbox}, text1};

pub struct Intro<'io> {
    io: &'io mut dyn IoSystem,
    screen: Screen,
    lines: Vec<(String, bool)>,
    headers: Vec<(String, usize)>,
}

impl<'io> Intro<'io> {
    pub fn new(io: &'io mut dyn IoSystem) -> Self {
        let sz = io.size();
        Self { io, screen: Screen::new(sz), lines: vec![], headers: vec![] }
    }

    fn prev_lines(&self) -> Vec<Text> {
        self.lines.iter().map(|(text, from_admin)| {
            let line = if *from_admin {
                format!("     ADMIN: {}\n", text)
            } else {
                format!("         >  {}\n", text)
            };
            Text::of(line)
        }).collect()
    }

    async fn render(&mut self) -> io::Result<()> {
        self.screen.resize(self.io.size());
        self.screen.textbox(self.prev_lines())
            .scroll_bottom(true);
        self.io.draw(&self.screen).await
    }

    async fn admin(&mut self, delay: u64, text: &str) -> io::Result<()> {
        tokio::time::sleep(Duration::from_millis(delay)).await;
        self.lines.push((text.into(), true));

        self.render().await
    }

    async fn choose<'s>(&mut self, options: &[&'s str]) -> io::Result<&'s str> {
        let prev_src = self.prev_lines();
        let mut selected = 0;
        loop {
            // draw the current screen
            let mut lines = prev_src.clone();
            lines.push(text1!(">"));
            for (i, option) in options.iter().enumerate() {
                lines.push(text1!("  "));
                let opt = if i == selected {
                    text1!(underline "{}"(option))
                } else {
                    text1!("{}"(option))
                };
                lines.push(opt);
            }
            self.screen.resize(self.io.size());
            self.screen.textbox(lines)
                .scroll_bottom(true);
            self.io.draw(&self.screen).await?;

            // handle input events
            if let Action::KeyPress { key } = self.io.input().await? {
                match key {
                    Key::Left => if selected > 0 {
                        selected -= 1;
                    }
                    Key::Right => if selected < options.len() - 1 {
                        selected += 1
                    }
                    Key::Enter => break,
                    _ => (),
                }
            }
        }
        self.lines.push((options[selected].into(), false));
        self.render().await?;
        Ok(options[selected])
    }

    /// special function for this choice for the mini-tutorial
    async fn first_choice(&mut self) -> io::Result<&'static str> {
        // TODO: handle timeout and extra prompt
        self.choose(&["yes", "no"]).await
    }

    async fn cli_intro(&mut self) -> io::Result<()> {
        self.admin(0, "the CLI intro will eventually go here").await
    }

    pub async fn run(&mut self) -> io::Result<()> {
        macro_rules! admin {
            ($($delay:literal => $text:literal),* $(,)?) => {
                $(
                    self.admin($delay, $text).await?
                );*
            }
        }
        macro_rules! choose {
            ($( $text:literal => $($expr:expr $(,)?)? )*) => {
                match self.choose(&[ $($text),* ]).await? {
                    $(
                        $text => $( $expr, )?
                    )*
                    _ => unreachable!("choose returned option not selected"),
                }
            }
        }

        admin! {
            1000 => "hey.",
            1500 => "you ever use redshell before?",
        }
        match self.first_choice().await? {
            "yes" => {
                admin!(1000 => "you don't need this explained, then");
                return Ok(());
            }
            "no" => admin!(700 => "let's go over the basics"),
            _ => unreachable!("first choice returned nonexistent option ;-;"),
        }

        admin!(1500 => "have you used a command line?");
        choose! {
            "never" => self.cli_intro().await?,
            "a little" => {
                admin!(500 => "do you know what a command and arguments are?");
                choose! {
                    "yes" => admin!(300 => "you'll do just fine."),
                    "maybe" => {
                        admin!(300 => "let's review, then.");
                        self.cli_intro().await?;
                    }
                    "no" => {
                        admin!(300 => "an introduction, then.");
                        self.cli_intro().await?;
                    }
                }
            }
            "lots" => admin!(500 => "we'll skip that bit, then..."),
        }

        

        todo!()
    }
}
