//! Contains the "main loop" of the engine, in all its various incarnations based on the `run_` feature selected.

#![cfg_attr(not(feature = "__run"), allow(unused))]

use std::{mem, thread, time::Duration};

use tuig_iosys::{Action, IoRunner, IoSystem, Screen};
use tuig_ui::Region;

use crate::{
    agent::{Agent, ControlFlow},
    game::{Game, Response},
    util::timing::Timer,
    Message, Replies,
};

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

    /// Perform one round of message processing.
    ///
    /// `agents` and `messages` are both input and output:
    ///
    /// - `agents` and `messages` passed in are the agents/messages for this runner to run
    /// - `agents` and `messages` coming out are the agents that this round spawned
    ///
    /// Notably the vecs *will be cleared* and old messages *will not be available*!
    #[cfg_attr(feature = "run_rayon", allow(unused))]
    fn step(&mut self, messages: &mut Vec<M>, agents: &mut Vec<Box<dyn Agent<M>>>) {
        self.agents.extend(
            agents
                .drain(..)
                .map(|mut a| (a.start(&mut self.replies), a)),
        );

        if messages.is_empty() {
            messages.push(M::tick());
        }

        for (cf, agent) in self.agents.iter_mut() {
            if !cf.is_ready() {
                continue;
            }
            for msg in messages.iter() {
                *cf = agent.react(msg, &mut self.replies);
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

        // we're done with the old messages now
        messages.clear();
        // pragmatically this just outputs self.replies.messages and clears it, but this reuses allocations
        mem::swap(&mut self.replies.messages, messages);
        // ditto but for agents (no clear needed because we drained earlier)
        mem::swap(&mut self.replies.agents, agents);
    }

    /// Perform one round of message processing, using rayon.
    ///
    /// `agents` and `messages` are both input and output:
    ///
    /// - `agents` and `messages` passed in are the agents/messages for this runner to run
    /// - `agents` and `messages` coming out are the agents that this round spawned
    ///
    /// Notably the vecs *will be cleared* and old messages *will not be available*!
    #[cfg(feature = "run_rayon")]
    fn step_rayon(&mut self, messages: &mut Vec<M>, agents: &mut Vec<Box<dyn Agent<M>>>) {
        use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};

        let mut replies = Replies::default();
        self.agents
            .extend(agents.drain(..).map(|mut a| (a.start(&mut replies), a)));

        if messages.is_empty() {
            messages.push(M::tick());
        }

        let agent_replies = self
            .agents
            .par_iter_mut()
            .map(|(cf, agent)| {
                let mut replies = Replies::default();
                if !cf.is_ready() {
                    return replies;
                }
                for msg in messages.iter() {
                    *cf = agent.react(msg, &mut replies);
                    if !cf.is_ready() {
                        break;
                    }
                }
                replies
            })
            .reduce(Replies::default, |mut old, new| {
                old.agents.extend(new.agents);
                old.messages.extend(new.messages);
                old
            });
        replies.agents.extend(agent_replies.agents);
        replies.messages.extend(agent_replies.messages);

        // filter out agents that will never wake up
        self.agents.retain(|(cf, _ag)| match cf {
            // never is_ready again
            ControlFlow::Kill => false,
            // if there's only one reference, it's the one in this handle
            ControlFlow::Handle(h) => h.references() > 1,
            // otherwise it might eventually wake up, keep it around
            _ => true,
        });

        // no attempt to reuse allocations because we can't anyway in parallel
        *messages = replies.messages;
        *agents = replies.agents;
    }
}

struct GameRunner<G: Game, IO: IoSystem> {
    /// The game being run
    game: G,
    /// The IO system the game is being drawn onto
    iosys: IO,
    /// The screen drawn last time it rendered to screen
    old_screen: Screen,
    /// The screen that will be drawn next time it's rendered
    new_screen: Screen,
    /// Whether, after receiving a message, the game indicates that it needs to be redrawn
    tainted: bool,
    render_timer: Timer,
}

impl<G: Game, IO: IoSystem> GameRunner<G, IO> {
    fn new(game: G, iosys: IO) -> Self {
        let screen = Screen::new(iosys.size());
        Self {
            game,
            iosys,
            old_screen: screen.clone(),
            new_screen: screen.clone(),
            tainted: false,
            // Render at most ~100fps
            render_timer: Timer::new(1.0 / 100.0),
        }
    }

    /// Feed a list of messages to the associated `Game`.
    ///
    /// Returns whether a stop was requested.
    #[inline]
    fn feed(&mut self, messages: &[G::Message]) -> bool {
        if messages.is_empty() {
            return self.feed(&[G::Message::tick()]);
        }

        for msg in messages {
            match self.game.message(msg) {
                Response::Nothing => (),
                Response::Redraw => self.tainted = true,
                Response::Quit => return true,
            }
        }
        false
    }

    #[inline]
    fn attach_for(&mut self, replies: &mut Replies<G::Message>, action: Action) {
        self.new_screen.resize(self.iosys.size());
        let region = Region::new(&mut self.new_screen, action);
        self.game.attach(region, replies);
        self.tainted = false;
    }

    /// Do a step of IO with the associated `IoSystem` and `Game`, re-rendering to the stored [`Screen`].
    ///
    /// Returns whether a stop was requested.
    #[inline]
    fn attach(
        &mut self,
        messages: &mut Vec<G::Message>,
        agents: &mut Vec<Box<dyn Agent<G::Message>>>,
    ) -> bool {
        let mut replies = Replies {
            agents: mem::take(agents),
            messages: mem::take(messages),
        };
        while let Ok(Some(action)) = self.iosys.poll_input() {
            if action == Action::Closed {
                return true;
            }

            self.attach_for(&mut replies, action);
        }
        if self.tainted {
            // need to redraw, but haven't done it yet
            self.attach_for(&mut replies, Action::Redraw);
        }
        *agents = replies.agents;
        *messages = replies.messages;
        false
    }

    /// Render the stored [`Screen`] to the real screen. This will automatically only render if the screen contents
    /// have been tainted (e.g. by a [`Response::Redraw`] or [`Action::Redraw`]) and the render timer says it's time.
    fn render(&mut self) {
        if self.new_screen != self.old_screen && self.render_timer.tick_ready() {
            self.iosys.draw(&self.new_screen).unwrap();
            mem::swap(&mut self.new_screen, &mut self.old_screen);
        }
    }
}

/// Handles starting up and running a `Game`.
#[must_use]
pub struct Runner<G: Game + 'static> {
    messages: Vec<G::Message>,
    agents: Vec<Box<dyn Agent<G::Message>>>,
    game: G,
    input_tick: f32,
}

impl<G: Game + 'static> Runner<G> {
    /// Prepare a game to be run.
    pub fn new(game: G) -> Self {
        Self {
            game,
            messages: vec![],
            agents: vec![],
            input_tick: 0.1,
        }
    }

    /// Set an agent to be running at game startup, to process the first round of messages.
    pub fn spawn(mut self, agent: impl Agent<G::Message> + 'static) -> Self {
        self.agents.push(Box::new(agent));
        self
    }

    /// Add a message to be handled in the first round, by the first crop of [`Self::spawn`]ed agents.
    pub fn queue(mut self, msg: G::Message) -> Self {
        self.messages.push(msg);
        self
    }

    /// Set the desired time between rounds of messages.
    ///
    /// If processing a round takes longer than this, the game is considered to be "lagging". If it takes less time,
    /// then the runner will sit around, just processing input until the round is done.
    ///
    /// The exact mechanics of round timing in laggy games is deliberately left unspecified so I can fiddle with it to
    /// make it "work nicer". Broadly, though: If it lags a little and sporadically, the rounds tick over immediately
    /// until it "catches up" to approximately match realtime. If it properly *lags out*, getting too far behind, then
    /// the timer resets and starts ticking relative to the end of the lag.
    pub fn input_tick(mut self, tick: f32) -> Self {
        self.input_tick = tick;
        self
    }

    #[cfg(feature = "run_orig")]
    fn run_orig(self, iosys: impl IoSystem + 'static, mut iorun: impl IoRunner) -> G {
        let Self {
            game,
            mut messages,
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
                    if gr.attach(&mut messages, &mut agents) {
                        break 'mainloop;
                    }
                    if input_timer.tick_ready() {
                        break;
                    }
                    thread::sleep(input_timer.remaining().min(Duration::from_millis(2)));
                }
                gr.render();
                if gr.feed(&messages) {
                    break 'mainloop;
                }
                ar.step(&mut messages, &mut agents);
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
            mut messages,
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
                if gr.attach(&mut messages, &mut agents) {
                    break 'mainloop;
                }
                if input_timer.tick_ready() {
                    break;
                }
                thread::sleep(input_timer.remaining().min(Duration::from_millis(2)));
            }
            gr.render();
            if gr.feed(&messages) {
                break 'mainloop;
            }
            ar.step(&mut messages, &mut agents);
        }
        gr.iosys.stop();
        iorun.run();
        gr.game
    }

    #[cfg(feature = "run_rayon")]
    fn run_rayon(self, iosys: impl IoSystem + 'static, mut iorun: impl IoRunner) -> G {
        let (send, recv) = crossbeam::channel::bounded(0);
        rayon::spawn(move || {
            let Self {
                game,
                mut messages,
                mut agents,
                input_tick,
            } = self;

            let mut ar = AgentRunner::new();
            let mut gr = GameRunner::new(game, iosys);
            let mut input_timer = Timer::new(input_tick);

            'mainloop: loop {
                loop {
                    gr.render();
                    if gr.attach(&mut messages, &mut agents) {
                        break 'mainloop;
                    }
                    if input_timer.tick_ready() {
                        break;
                    }
                    thread::sleep(input_timer.remaining().min(Duration::from_millis(2)));
                }
                gr.render();
                if gr.feed(&messages) {
                    break 'mainloop;
                }
                ar.step_rayon(&mut messages, &mut agents);
            }
            gr.iosys.stop();
            send.send(gr.game).unwrap();
        });
        iorun.run();
        recv.recv().unwrap()
    }

    /// Start the game running, according to the feature-selected runner.
    ///
    /// This function only exits when [`Game::message`] or [`Game::input`] returns [`Response::Quit`]. It returns the
    /// [`Game`], primarily for testing purposes.
    #[allow(unreachable_code)] // for `cargo check --all-features`
    pub fn run(self, iosys: impl IoSystem + 'static, iorun: impl IoRunner) -> G {
        use crate::util::macros::feature_switch;

        feature_switch!(
            "run_orig" => self.run_orig(iosys, iorun),
            "run_single" => self.run_single(iosys, iorun),
            "run_rayon" => self.run_rayon(iosys, iorun),
        )
    }

    /// Use [`crate::io::load!`] to intelligently pick an iosystem, load it, and [`Self::run`].
    ///
    /// This **must** be run on the main thread. Ideally, you'd run it from `main` directly.
    ///
    /// This function only exits when [`Game::message`] or [`Game::input`] returns [`Response::Quit`]. It returns the
    /// [`Game`], primarily for testing purposes. If loading fails, it panics.
    #[cfg(feature = "__io")]
    #[cfg_attr(doc, doc(cfg(feature = "io_*")))]
    pub fn load_run(self) -> G {
        tuig_iosys::load!(self.run).unwrap()
    }
}
