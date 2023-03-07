//! Contains the "main loop" of the engine, in all its various incarnations based on the `run_` feature selected.

#![cfg(feature = "__run")]

use std::{mem, thread, time::Duration};

use crate::{
    agents::{Agent, ControlFlow},
    game::{Game, Message, Replies, Response},
    io::{input::Action, output::Screen, sys::IoSystem},
};

use crate::{io::sys::IoRunner, timing::Timer};

#[cfg(feature = "__sys")]
use crate::io::sys;

struct AgentRunner<M: Message> {
    agents: Vec<(ControlFlow, Box<dyn Agent<M>>)>,
    replies: Replies<M>,
}

impl<M: Message> AgentRunner<M> {
    fn new() -> Self {
        Self {
            agents: Default::default(),
            replies: Default::default(),
        }
    }

    /// Perform one round of event processing.
    ///
    /// `agents` and `events` are both input and output:
    ///
    /// - `agents` and `events` passed in are the agents/events for this runner to run
    /// - `agents` and `events` coming out are the agents that this round spawned
    ///
    /// Notably the vecs *will be cleared* and old events *will not be available*!
    fn step(&mut self, events: &mut Vec<M>, agents: &mut Vec<Box<dyn Agent<M>>>) {
        self.agents.extend(
            agents
                .drain(..)
                .map(|mut a| (a.start(&mut self.replies), a)),
        );

        if events.is_empty() {
            events.push(M::tick());
        }

        for (cf, agent) in self.agents.iter_mut() {
            if !cf.is_ready() {
                continue;
            }
            for event in events.iter() {
                *cf = agent.react(event, &mut self.replies);
                if !cf.is_ready() {
                    break;
                }
            }
        }

        // filter out agents that will never wake up
        self.agents.retain(|(cf, _ag)| match cf {
            // never is_ready again
            ControlFlow::Kill => false,
            // if there's only one reference, it's the one in this handle
            ControlFlow::Handle(h) => h.references() > 1,
            // otherwise it might eventually wake up, keep it around
            _ => true,
        });

        // we're done with the old events now
        events.clear();
        // pragmatically this just outputs self.replies.messages and clears it, but this reuses allocations
        mem::swap(&mut self.replies.messages, events);
        // ditto but for agents (no clear needed because we drained earlier)
        mem::swap(&mut self.replies.agents, agents);
    }
}

struct GameRunner<G: Game, IO: IoSystem> {
    game: G,
    iosys: IO,
    screen: Screen,
    tainted: bool,
    render_timer: Timer,
}

impl<G: Game, IO: IoSystem> GameRunner<G, IO> {
    fn new(game: G, iosys: IO) -> Self {
        let screen = Screen::new(iosys.size());
        Self {
            game,
            iosys,
            screen,
            tainted: true,
            // Render at most ~60fps
            render_timer: Timer::new(1.0 / 60.0),
        }
    }

    /// Feed a list of events to the associated `Game`.
    ///
    /// Returns whether a stop was requested.
    fn feed(&mut self, events: &[G::Message]) -> bool {
        if events.is_empty() {
            return self.feed(&[G::Message::tick()]);
        }

        for event in events {
            match self.game.event(event) {
                Response::Nothing => (),
                Response::Redraw => self.tainted = true,
                Response::Quit => return true,
            }
        }
        false
    }

    /// Do a step of IO with the associated `IoSystem` and `Game`.
    ///
    /// Returns whether a stop was requested.
    fn io(
        &mut self,
        events: &mut Vec<G::Message>,
        agents: &mut Vec<Box<dyn Agent<G::Message>>>,
    ) -> bool {
        let mut replies = Replies {
            agents: mem::take(agents),
            messages: mem::take(events),
        };
        while let Ok(Some(action)) = self.iosys.poll_input() {
            match action {
                Action::Closed => return true,
                Action::Redraw => self.tainted = true,
                other => match self.game.input(other, &mut replies) {
                    Response::Nothing => (),
                    Response::Redraw => self.tainted = true,
                    Response::Quit => return true,
                },
            }
        }
        *agents = replies.agents;
        *events = replies.messages;
        false
    }

    /// Render to the screen.
    ///
    /// This will automatically only render if:
    ///
    /// - The screen contents have been tainted (e.g. by a [`Response::Redraw`] or [`Action::Redraw`])
    /// - It's been long enough since the last redraw
    fn render(&mut self) {
        let new_size = self.iosys.size();
        if self.tainted || new_size != self.screen.size() {
            if !self.render_timer.tick_ready() {
                // avoid wasting too much time rendering
                return;
            }
            self.screen.resize(new_size);
            self.game.render(&mut self.screen);
            self.iosys.draw(&self.screen).unwrap();
            self.tainted = false;
        }
    }
}

/// Handles starting up and running a `Game`.
#[must_use]
pub struct Runner<G: Game + 'static> {
    events: Vec<G::Message>,
    agents: Vec<Box<dyn Agent<G::Message>>>,
    game: G,
    input_tick: f32,
}

impl<G: Game + 'static> Runner<G> {
    /// Prepare a game to be run
    pub fn new(game: G) -> Self {
        Self {
            game,
            events: vec![],
            agents: vec![],
            input_tick: 0.25,
        }
    }

    /// Set an agent to be running at game startup, to process the first tick of events
    pub fn spawn(mut self, agent: impl Agent<G::Message> + 'static) -> Self {
        self.agents.push(Box::new(agent));
        self
    }

    /// Add a message to be handled on the first tick, by the first crop of [`spawn`][Self::spawn]ed agents.
    pub fn queue(mut self, event: G::Message) -> Self {
        self.events.push(event);
        self
    }

    /// Set the target time between rounds of events.
    ///
    /// Note that rounds may take longer, if it just takes longer to handle all the events in a round.
    pub fn input_tick(mut self, tick: f32) -> Self {
        self.input_tick = tick;
        self
    }

    /// Implementation of [`Self::run`] for `run_orig`: Monopolizes the main thread for the IoRunner, and spins off
    /// another thread to handle the game and all agents.
    #[cfg(feature = "run_orig")]
    fn run_orig(self, iosys: impl IoSystem + 'static, mut iorun: impl IoRunner) -> G {
        let Self {
            game,
            mut events,
            mut agents,
            input_tick,
        } = self;

        let thread = thread::spawn(move || {
            let mut ar = AgentRunner::new();
            let mut gr = GameRunner::new(game, iosys);
            let mut input_timer = Timer::new(input_tick);

            'mainloop: loop {
                loop {
                    gr.render();
                    if gr.io(&mut events, &mut agents) {
                        break 'mainloop;
                    }
                    if input_timer.tick_ready() {
                        break;
                    }
                    thread::sleep(input_timer.remaining().min(Duration::from_millis(2)));
                }
                gr.render();
                if gr.feed(&events) {
                    break 'mainloop;
                }
                ar.step(&mut events, &mut agents);
            }
            gr.iosys.stop();
            gr.game
        });
        iorun.run();
        thread.join().unwrap()
    }

    #[cfg(feature = "run_single")]
    fn run_single(self, iosys: impl IoSystem + 'static, mut iorun: impl IoRunner) -> G {
        let Self {
            game,
            mut events,
            mut agents,
            input_tick,
        } = self;

        let mut ar = AgentRunner::new();
        let mut gr = GameRunner::new(game, iosys);
        let mut input_timer = Timer::new(input_tick);

        'mainloop: loop {
            loop {
                gr.render();
                if iorun.step() {
                    break 'mainloop;
                }
                if gr.io(&mut events, &mut agents) {
                    break 'mainloop;
                }
                if input_timer.tick_ready() {
                    break;
                }
                thread::sleep(input_timer.remaining().min(Duration::from_millis(2)));
            }
            gr.render();
            if gr.feed(&events) {
                break 'mainloop;
            }
            ar.step(&mut events, &mut agents);
        }
        gr.iosys.stop();
        iorun.run();
        gr.game
    }

    /// Start the game running, according to the feature-selected runner.
    ///
    /// This function only exits when [`Game::event`] or [`Game::input`] returns [`Response::Quit`]. It returns the
    /// [`Game`], primarily for testing purposes.
    #[cfg(feature = "__run")]
    #[allow(unreachable_code)] // for `cargo check --all-features`
    pub fn run(self, iosys: impl IoSystem + 'static, iorun: impl IoRunner) -> G {
        macro_rules! run_call {
            ( $( $feature:literal => $function:ident ),* $(,)? ) => { $(
                #[cfg(feature = $feature)]
                {
                    return self.$function(iosys, iorun);
                }
            )* };
        }
        run_call!("run_orig" => run_orig, "run_single" => run_single);
    }

    /// Use [`sys::load`] to intelligently pick an iosystem, load it, and [`Self::run`].
    ///
    /// This **must** be run on the main thread. Ideally, you'd run it from `main` directly.
    ///
    /// This function only exits when [`Game::event`] or [`Game::input`] returns [`Response::Quit`]. It returns the
    /// [`Game`], primarily for testing purposes.
    #[cfg(all(feature = "__sys", feature = "__run"))]
    pub fn load_run(self) -> G {
        sys::load!(self.run).unwrap()
    }
}
