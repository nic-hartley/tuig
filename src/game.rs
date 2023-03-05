//! Contains the "main loop" bits of the game. Passes events around agents, renders and handles I/O, etc.
//!
//! This is also the primary split between the "engine" and "game" halves.

use core::fmt;
use std::{fmt::Debug, mem, thread};

use crate::{
    agents::{Agent, ControlFlow},
    io::{
        input::Action,
        output::Screen,
        sys::{self, IoSystem, IoRunner},
    }, timing::Timer,
};

pub trait Message: Clone + Send + Sync {
    /// The message to send agents when there aren't any other messages queued for processing, to ensure every awake
    /// agent processes at least one event per round. Will **not** be sent if there are any other events.
    /// 
    /// This method should be as simple and fast as possible, ideally just returning a constant value.
    fn tick() -> Self;
}
impl<T: Clone + Send + Sync + Default> Message for T {
    fn tick() -> Self {
        Self::default()
    }
}

/// Allows a [`Game`] or [`Agent`] to make things happen in the engine in response to events or input.
pub struct Replies<M: Message> {
    agents: Vec<Box<dyn Agent<M>>>,
    messages: Vec<M>,
}

impl<M: Message> Replies<M> {
    #[cfg(test)]
    /// A **test-only** function, so you can ensure your code queues the correct messages.
    pub fn messages(&self) -> &[M] {
        &self.messages
    }

    #[cfg(test)]
    /// A **test-only** function, so you can ensure your code spawns the correct agents.
    pub fn agents(&self) -> &[Box<dyn Agent<M>>] {
        &self.agents
    }
}

impl<M: Message> Default for Replies<M> {
    fn default() -> Self {
        Self {
            agents: Default::default(),
            messages: Default::default(),
        }
    }
}

impl<M: Message> Debug for Replies<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .field("agents", &self.agents.len())
            .field("messages", &self.messages.len())
            .finish()
    }
}

impl<M: Message> Replies<M> {
    pub fn spawn(&mut self, agent: impl Agent<M> + 'static) -> &mut Self {
        self.agents.push(Box::new(agent));
        self
    }
    pub fn spawn_boxed(&mut self, agent: Box<dyn Agent<M>>) -> &mut Self {
        self.agents.push(agent);
        self
    }
    pub fn queue(&mut self, msg: M) -> &mut Self {
        self.messages.push(msg);
        self
    }
    pub fn queue_all(&mut self, msgs: impl IntoIterator<Item = M>) -> &mut Self {
        self.messages.extend(msgs);
        self
    }

    pub fn spawn_len(&self) -> usize {
        self.agents.len()
    }
    pub fn queue_len(&self) -> usize {
        self.messages.len()
    }
}

/// Allows a [`Game`] to control the engine in response to events or input.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Response {
    /// Nothing in particular needs to be done.
    Nothing,
    /// The visual state has updated, and the screen needs to be redrawn.
    Redraw,
    /// The game should be exited, e.g. because the user clicked "Exit" in the menu.
    Quit,
}

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
    /// The message that this `Game` will be passing around between `Agent`s and itself.
    type Message: Message;

    /// The user has done some input; update the UI and inform [`Agent`]s accordingly.
    ///
    /// Returns whether the game needs to be redrawn after the user input.
    fn input(&mut self, input: Action, replies: &mut Replies<Self::Message>) -> Response;

    /// An event has happened; update the UI accordingly.
    ///
    /// Returns whether the game needs to be redrawn after the event.
    fn event(&mut self, event: &Self::Message) -> Response;

    /// Render the game onto the provided `Screen`.
    // TODO: Make this take &self instead
    fn render(&mut self, onto: &mut Screen);
}

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
        self.agents.extend(agents.drain(..).map(|mut a| (a.start(&mut self.replies), a)));

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
    event_timer: Timer,
}

impl<G: Game, IO: IoSystem> GameRunner<G, IO> {
    fn new(game: G, iosys: IO) -> Self {
        let screen = Screen::new(iosys.size());
        Self {
            game, iosys, screen, tainted: true,
            // Render at most ~60fps
            render_timer: Timer::new(1.0 / 60.0),
            // Process events every quarter of a second
            event_timer: Timer::new(1.0 / 4.0),
        }
    }

    /// This uses the same API as [`AgentRunner::step`].
    /// 
    /// Returns whether to keep running the game or not
    fn step(&mut self, events: &mut Vec<G::Message>, agents: &mut Vec<Box<dyn Agent<G::Message>>>) -> bool {
        // handle game events as the last bit before the tickover
        for event in events.iter() {
            match self.game.event(event) {
                Response::Nothing => (),
                Response::Redraw => self.tainted = true,
                Response::Quit => return false,
            }
        }
        // and that's it, there's no replying to events, so we can just clear the vec (to reuse allocations)
        events.clear();
        // we don't care about agents, so clear those as well.
        agents.clear();

        // process input for any remaining time (input_until is supposed to prefer exhausting all input,
        // so we'll eagerly grab all available input until we're out of time)

        // the take/replace ensures we reuse allocations as much as possible; the earlier `clear`s ensure they start
        // empty and don't cause problems.
        let mut replies = Replies { agents: mem::take(agents), messages: mem::take(events) };
        while let Ok(Some(action)) = self.iosys.input_until(self.event_timer.remaining()) {
            match action {
                Action::Closed => return false,
                Action::Redraw => self.tainted = true,
                other => match self.game.input(other, &mut replies) {
                    Response::Nothing => (),
                    Response::Redraw => self.tainted = true,
                    Response::Quit => return false,
                },
            }
        }
        *agents = replies.agents;
        *events = replies.messages;

        self.event_timer.tick();
        self.render();
        true
    }

    fn render(&mut self) {
        if !self.render_timer.ready() {
            return;
        }
        let new_size = self.iosys.size();
        if self.tainted || new_size != self.screen.size() {
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
    pub fn spawn(mut self, agent: impl Agent<G::Message> + 'static) -> Self {
        self.agents.push(Box::new(agent));
        self
    }

    /// Add a message to be handled on the first tick, by the first crop of [`spawn`][Self::spawn]ed agents.
    pub fn queue(mut self, event: G::Message) -> Self {
        self.events.push(event);
        self
    }

    #[cfg(feature = "run_orig")]
    fn run_game(self, iosys: impl IoSystem) -> G {
        let Self {
            game,
            mut events,
            mut agents,
        } = self;

        let mut ar = AgentRunner::new();
        let mut gr = GameRunner::new(game, iosys);

        loop {
            if !gr.step(&mut events, &mut agents) {
                break;
            }
            ar.step(&mut events, &mut agents);
        }
        gr.iosys.stop();
        gr.game
    }

    /// Implementation of [`Self::run`] for `run_orig`: Monopolizes the main thread for the IoRunner, and spins off
    /// another thread to handle the game and all agents.
    #[cfg(feature = "run_orig")]
    fn run_impl(self, iosys: impl IoSystem + 'static, mut iorun: impl IoRunner) -> G {
        let thread = thread::spawn(move || self.run_game(iosys));
        iorun.run();
        thread.join().unwrap()
    }

    /// Start the game running.
    ///
    /// This **must** be run on the main thread. Ideally, you'd run it from `main` directly.
    ///
    /// This function only exits when [`Game::event`] or [`Game::input`] returns [`Response::Quit`]. It returns the
    /// [`Game`], primarily for testing purposes.
    #[cfg(all(feature = "__run", feature = "__sys"))]
    pub fn run(self) -> G {
        sys::load!(self.run_impl).unwrap()
    }
}
