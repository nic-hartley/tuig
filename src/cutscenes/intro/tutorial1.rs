use std::io;

use crate::{
    cutscenes::intro::sleep,
    io::{
        clifmt::{FormattedExt, Text},
        input::{Action, Key},
        output::Screen,
        sys::IoSystem,
    },
    text, text1,
};

async fn render(io: &mut dyn IoSystem, screen: &mut Screen, text: &[Text]) -> io::Result<()> {
    screen.resize(io.size());
    let mut text_v: Vec<_> = text.iter().cloned().collect();
    if let Some(last) = text_v.last_mut() {
        // trim trailing newline
        last.text = last.text.trim_end().into();
    }
    screen.textbox(text_v).scroll_bottom(true);
    io.draw(screen).await
}

async fn render_sleep(
    delay: f32,
    io: &mut dyn IoSystem,
    screen: &mut Screen,
    text: &[Text],
) -> io::Result<()> {
    let timer = sleep(delay);
    tokio::pin!(timer);
    loop {
        render(io, screen, &text).await?;

        tokio::select! {
            _ = &mut timer => break,
            _ = io.input() => (),
        }
    }
    Ok(())
}

/// mostly the same as cli_input but has an extra bit of logic for the "type now" prompt, and to limit name length
async fn name_input(
    io: &mut dyn IoSystem,
    screen: &mut Screen,
    text: &[Text],
) -> io::Result<String> {
    let mut last_line = text!("         >  ", white "(type now)");
    let mut name = String::new();
    let mut caps = false;
    loop {
        let cur_text: Vec<_> = text.iter().chain(last_line.iter()).cloned().collect();
        render(io, screen, &cur_text).await?;

        match io.input().await? {
            Action::KeyPress { key: Key::Enter } if !name.is_empty() => return Ok(name),
            Action::KeyPress { key: Key::Char(ch) } => {
                if name.len() < 10 {
                    if caps {
                        name.extend(ch.to_uppercase())
                    } else {
                        name.extend(ch.to_lowercase())
                    }
                }
            }
            Action::KeyPress {
                key: Key::Backspace,
            } => {
                name.pop();
            }
            Action::KeyPress {
                key: Key::LeftShift | Key::RightShift,
            } => caps = true,
            Action::KeyRelease {
                key: Key::LeftShift | Key::RightShift,
            } => caps = false,
            // other inputs can get ignored
            _ => (),
        }
        if last_line.len() == 2 {
            // then we're still in the first format, with the placeholder
            if name.is_empty() {
                // don't get rid of the prompt, they haven't typed anything yet
                // maybe they hit backspace, maybe they resized the window, maybe they moved their mouse
            } else {
                // they typed something into the name, switch into the new format
                last_line[1] = text1!(blue "{}"(name));
                last_line.push(text1!(white on_white "."));
            }
        } else {
            last_line[1].text = name.clone();
        }
    }
}

async fn do_choice<'a>(
    io: &mut dyn IoSystem,
    screen: &mut Screen,
    text: &[Text],
    opts: &[&'a str],
) -> io::Result<&'a str> {
    let mut selected = 0;
    loop {
        let mut last_line = Vec::with_capacity(opts.len() * 2);
        for (i, opt) in opts.iter().enumerate() {
            if i == 0 {
                last_line.push(text1!("         >  "));
            } else {
                last_line.push(text1!("  "));
            }
            let mut text = text1!(bold bright_white "{}"(opt));
            if i == selected {
                text = text.underline();
            }
            last_line.push(text);
        }
        let text: Vec<_> = text.iter().cloned().chain(last_line).collect();

        render(io, screen, &text).await?;

        match io.input().await? {
            Action::KeyPress { key: Key::Enter } => return Ok(opts[selected]),
            Action::KeyPress { key: Key::Left } => {
                if selected > 0 {
                    selected -= 1;
                }
            }
            Action::KeyPress { key: Key::Right } => {
                if selected < opts.len() - 1 {
                    selected += 1;
                }
            }
            _ => (),
        }
    }
}

fn autocomplete<'a>(text: &str, options: &[&'a str]) -> Option<&'a str> {
    let mut complete: Option<&'a str> = None;
    for cand in options {
        if !cand.starts_with(text) {
            continue;
        }
        if cand == &text {
            // don't try to display the exact command as a completion
            continue;
        }
        if let Some(prev) = complete {
            // can unwrap mostly safely:
            // - can't inseminate cuties
            let (shared_end, _) = cand
                .chars()
                .zip(prev.chars())
                .enumerate()
                .find(|(_, (c, p))| c != p)
                .unwrap();
            complete = Some(&prev[..shared_end]);
        } else {
            complete = Some(cand);
        }
    }
    if let Some(c) = complete {
        if c.len() <= text.len() {
            return None;
        } else {
            return Some(&c[text.len()..]);
        }
    }
    None
}

async fn cli_input(
    io: &mut dyn IoSystem,
    screen: &mut Screen,
    text: &[Text],
    completes: &[&str],
) -> io::Result<String> {
    let mut last_line = text!("$ ", "", bright_black on_white " ", bright_black "");
    let mut cmd = String::new();
    let mut caps = false;
    let mut complete: Option<&str> = None;
    loop {
        let cur_text: Vec<_> = text.iter().chain(last_line.iter()).cloned().collect();
        render(io, screen, &cur_text).await?;

        match io.input().await? {
            Action::KeyPress { key: Key::Enter } => return Ok(cmd),
            Action::KeyPress { key: Key::Char(ch) } => {
                complete = None;
                if caps {
                    cmd.extend(ch.to_uppercase())
                } else {
                    cmd.extend(ch.to_lowercase())
                }
            }
            Action::KeyPress {
                key: Key::Backspace,
            } => {
                complete = None;
                cmd.pop();
            }
            Action::KeyPress {
                key: Key::LeftShift | Key::RightShift,
            } => caps = true,
            Action::KeyRelease {
                key: Key::LeftShift | Key::RightShift,
            } => caps = false,
            Action::KeyPress { key: Key::Tab } => {
                if let Some(already) = complete {
                    cmd.extend(already.chars());
                    complete = None;
                } else {
                    complete = autocomplete(&cmd, &completes);
                }
            }
            // other inputs can get ignored
            _ => (),
        }
        last_line[1].text = cmd.clone();
        if let Some(complete) = complete {
            let (ch, rest) = complete.split_at(1);
            last_line[2].text = ch.into();
            last_line[3].text = rest.into();
        } else {
            last_line[2].text = " ".into();
            last_line[3].text = String::new();
        }
    }
}

async fn cli_tut(io: &mut dyn IoSystem, screen: &mut Screen) -> io::Result<()> {
    let mut output = vec![];
    macro_rules! say {
        ( $( $delay:expr => $text:literal $( ( $( $arg:expr ),* $(,)? ) )? );* $(;)? ) => {
            {
                $(
                    render_sleep($delay, io, screen, &output).await?;
                    let ele = Text::of(format!(concat!("# ", $text, "\n"), $($arg),* )).green();
                    output.push(ele);
                )*
            }
        };
    }
    macro_rules! prompt {
        ( $( $opts:literal ),* ) => { {
            let input = cli_input(io, screen, &output, &[ $($opts),* ]).await?;
            output.push(text1!("$ {}\n"(input)));
            input
        } };
    }

    say!(
        0.5 => "k, so, the first thing you'll notice:";
        0.5 => "real spartan.";
        1.0 => "try typing `help`";
    );
    let mut explained_quotes = false;
    loop {
        let in1 = prompt!("help", "skip");
        if in1 == "help" {
            say!(0.5 => "yeah, like that.");
            break;
        } else if in1 == "skip" {
            say!(1.0 => "...okay.");
            render_sleep(0.5, io, screen, &output).await?;
            return Ok(());
        } else if in1.contains("help") {
            say!(0.5 => "not quite");
            if !explained_quotes {
                say!(
                    1.0 => "when you see this: `";
                    1.0 => "that's just quoting the thing to type";
                );
                explained_quotes = true;
            }
        } else {
            say!(0.5 => "try again, type `help`");
        }
    }

    render_sleep(0.5, io, screen, &output).await?;
    Ok(())
}

pub async fn tutorial(io: &mut dyn IoSystem, screen: &mut Screen) -> io::Result<String> {
    let mut text: Vec<Text> = vec![];
    macro_rules! admin_say {
        ( $( $delay:expr =>
            $(
                $( $_name:ident )* $_fmt:literal $( ( $($arg:expr),* $(,)? ) )?
            ),* $(,)?
        );* $(;)? ) => { $(
            render_sleep($delay, io, screen, &text).await?;
            text.extend(text!(
                "     admin: ",
                $( bold bright_white $($_name)* $_fmt $(($($arg),*))? ),*,
                "\n"
            ));
        )* };
    }

    admin_say!(
        1.0 => "welcome to the fight";
        1.5 => "what can I call you?";
        0.75 => "not your real name.";
    );
    let name = name_input(io, screen, &text).await?;
    text.extend(text!("         >  ", blue "{}"(name), "\n"));

    macro_rules! choose {
        ( $( $option:literal => $then:expr $(,)? )* ) => { {
            match do_choice(io, screen, &text, &[$($option),*]).await? {
                $( $option => {
                    text.extend(text!(
                        blue "{:>10}: "(name),
                        bold bright_white $option,
                        "\n",
                    ));
                    $then
                }, )*
                _ => unreachable!("selected unavailable choice"),
            }
        } };
    }

    admin_say!(
        1.0 => "you're ", blue "{}"(name), "?";
        1.5 => "good name";
        1.5 => "do you need an intro?";
    );
    choose! {
        "yes" => (), // continues below
        "no" => {
            admin_say!(
                0.5 => "cool!";
                1.0 => "good luck";
            );
            return Ok(name);
        }
    }

    admin_say!(
        1.0 => "ok.";
        0.5 => "so.";
        1.0 => "do you know what a command line is?";
    );
    let do_cli_tut = choose! {
        "yes" => {
            admin_say!(1.0 => "do you know how to use one?");
            choose! {
                "yes" => false,
                "no" => true,
            }
        }
        "no" => true
    };
    if do_cli_tut {
        admin_say!(
            0.5 => "quick intro then";
            1.0 => "actually wait, give me a second...",
        );
        render_sleep(0.5, io, screen, &text).await?;
        text.push(text1!(green "RUNNING CLI TUTORIAL..."));
        render_sleep(1.0, io, screen, &text).await?;
        cli_tut(io, screen).await?;
    }

    admin_say!(
        1.0 => "alright, let me boot up the full interface...";
    );

    Ok(name)
}
