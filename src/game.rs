//! Contains the "main loop" bits of the game. Passes events around agents, renders and handles I/O, etc.
//!
//! This is also the primary split between the "engine" and "game" halves.

use std::{mem, thread, time::Duration};

use crate::{
    agents::{Agent, ControlFlow, Event},
    io::{
        input::Action,
        output::Screen,
        sys::{self, IoSystem},
    },
    util,
};

/// Represents a game which can be run in the main loop.
///
/// Note that `Game`s don't run the bulk of the game logic; that's the `Agent`'s job. The `Game` trait is the place
/// where user input and rendering happen. The idea here is:
///
/// - When there's relevant user input, you can send [`Event`]s or make new agents, and/or update state for rendering
/// - When an [`Event`] happens (including one you spawned!), you can update internal state for rendering
/// - You *don't* react to events with more events -- that's an `Agent`'s job
/// - Come time to render, you already have all the info you need from previous inputs/events
///
/// This makes the code a bit harder to write, but it clearly separates concerns and encourages you to put your heavy
/// logic somewhere other than the render thread.
pub trait Game: Send {
    /// The user has done some input; update the UI and inform [`Agent`]s accordingly.
    ///
    /// Returns whether the game needs to be redrawn after the user input.
    fn input(&mut self, input: Action, events: &mut Vec<Event>) -> bool;

    /// An event has happened; update the UI accordingly.
    ///
    /// Returns whether the game needs to be redrawn after the event.
    fn event(&mut self, event: &Event) -> bool;

    /// Render the game onto the provided `Screen`.
    // TODO: Make this take &self instead
    fn render(&mut self, onto: &mut Screen);
}

/// Handles starting up and running a `Game`.
#[must_use]
pub struct Runner<G: Game + 'static> {
    events: Vec<Event>,
    agents: Vec<Box<dyn Agent>>,
    game: G,
}

impl<G: Game + 'static> Runner<G> {
    /// Prepare a game to be run
    pub fn new(game: G) -> Self {
        Self {
            game,
            events: vec![],
            agents: vec![],
        }
    }

    /// Set an agent to be running at game startup, to process the first tick of events
    pub fn spawn(mut self, agent: impl Agent + Send + Sync + 'static) -> Self {
        self.agents.push(Box::new(agent));
        self
    }

    /// Add an [`Event`] to be executed on the first tick, by the first crop of [`spawn`][Self::spawn]ed agents.
    pub fn queue(mut self, event: Event) -> Self {
        self.events.push(event);
        self
    }

    fn run_game(self, iosys: &mut dyn IoSystem) {
        let mut screen = Screen::new(iosys.size());

        let Self {
            mut game,
            mut events,
            agents,
        } = self;

        let mut replies = vec![];
        let mut agents: Vec<_> = agents
            .into_iter()
            .map(|mut a| (a.start(&mut events), a))
            .collect();
        let mut tainted = true;
        loop {
            let new_size = iosys.size();
            if new_size != screen.size() {
                tainted = true;
            }

            if tainted {
                screen.resize(new_size);
                game.render(&mut screen);
                iosys.draw(&screen).unwrap();
                tainted = false;
            }

            if let Some(inp) = iosys.input_until(Duration::from_secs_f32(0.25)).unwrap() {
                match inp {
                    Action::Closed => break,
                    Action::Redraw => tainted = true,
                    other => tainted |= game.input(other, &mut replies),
                }
            }

            for event in &events {
                // TODO: better way to spawn agents
                if let Event::SpawnAgent(bundle) = event {
                    if let Some(mut ag) = bundle.take() {
                        agents.push((ag.start(&mut replies), ag));
                    }
                    continue;
                }
                tainted |= game.event(event);
                for (cf, agent) in &mut agents {
                    if !cf.is_ready() {
                        continue;
                    }
                    *cf = agent.react(event, &mut replies);
                }
            }

            mem::swap(&mut events, &mut replies);
            replies.clear();

            // filter out agents that will never wake up
            util::retain_unstable(&mut agents, |(cf, _ag)| match cf {
                // never is_ready again
                ControlFlow::Kill => false,
                // if there's only one reference, it's the one in the handle
                ControlFlow::Handle(h) => h.references() > 1,
                // otherwise it might eventually wake up, keep it around
                _ => true,
            });
        }
    }

    /// Start the game running.
    ///
    /// This **must** be run on the main thread. Ideally, you'd run it from `main` directly.
    pub fn run(self) {
        let (mut iosys, mut iorun) = sys::load().expect("failed to load");
        thread::spawn(move || self.run_game(iosys.as_mut()));
        iorun.run();
    }
}
