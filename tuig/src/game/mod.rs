use std::fmt;

use tuig_iosys::{Action, Screen};

use crate::agent::Agent;

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
    pub(crate) agents: Vec<Box<dyn Agent<M>>>,
    pub(crate) messages: Vec<M>,
}

impl<M: Message> Replies<M> {
    #[cfg(feature = "test_extras")]
    /// A **test-only** function, so you can ensure your code queues the correct messages.
    pub fn _messages(&self) -> &[M] {
        &self.messages
    }

    #[cfg(feature = "test_extras")]
    /// A **test-only** function, so you can ensure your code spawns the correct agents.
    pub fn _agents(&self) -> &[Box<dyn Agent<M>>] {
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

impl<M: Message> fmt::Debug for Replies<M> {
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
/// - When there's relevant user input, you can send `Message`s or make new agents, and/or update state for rendering
/// - When a `Message` happens (including one you spawned!), you can update internal state for rendering
/// - You *don't* react to messages with more messages -- that's an `Agent`'s job
/// - Come time to render, you already have all the info you need from previous inputs/events
///
/// This is a fairly typical Elm-style GUI, though obviously the event bus is used for more than just events, as it's
/// also the primary method of communication between agents and the game. This makes the code a bit harder to write,
/// but it clearly separates concerns and encourages you to put heavy logic somewhere other than the render thread.
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
    fn render(&self, onto: &mut Screen);
}
