//! Messages are the primary backbone of the engine's communication. Replies are how they're queued.

use std::fmt;

use crate::Agent;

/// A message that [`Agent`]s and [`Game`](crate::Game)s will be passing around.
///
/// The relevant agents and games will be taking this as a generic parameter, not using it as a trait object, and you
/// should do the same. Usually you'll implement this in an enum. If you *want* a trait object, use `Message` as a
/// supertrait for yours.
///
/// This trait requires `Clone`, `Send`, and `Sync`, to ensure it can be properly shared across all threads.
pub trait Message: Clone + Send + Sync {
    /// The message to send agents when there aren't any other messages queued for processing, to ensure every awake
    /// agent processes at least one event per round. Will **not** be sent if there are any other events!
    ///
    /// This method should be as simple and fast as possible, ideally just returning a constant value.
    fn tick() -> Self;
}

impl<T: Clone + Send + Sync + Default> Message for T {
    fn tick() -> Self {
        Self::default()
    }
}

/// Allows a [`Game`](crate::Game) or [`Agent`] to make things happen in the engine in response to events or input.
///
/// Remember that none of these will be acted on immediately -- only once the round ends.
pub struct Replies<M: Message> {
    pub(crate) agents: Vec<Box<dyn Agent<M>>>,
    pub(crate) messages: Vec<M>,
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
    /// Have an agent spawned into the next round of events or so.
    ///
    /// tuig will try to spawn the agent in to the immediately next round, but that's not always guaranteed, and in
    /// particular if the game is lagging it may be some time before new agents are spawned. If you need to ensure
    /// something happens when the agent starts, implement [`Agent::start`]. If one agent needs to synchronize with
    /// one it spawned, have the spawned agent send a "hello" message when it starts.
    pub fn spawn(&mut self, agent: impl Agent<M> + 'static) -> &mut Self {
        self.spawn_boxed(Box::new(agent))
    }

    /// The same as [`Replies::spawn`], but with a boxed agent.
    ///
    /// Generally, you should use `spawn`; it reduces boilerplate. But if you happen to be passing around trait
    /// objects in Boxes *anyway*, then you should use this.
    pub fn spawn_boxed(&mut self, agent: Box<dyn Agent<M>>) -> &mut Self {
        self.agents.push(agent);
        self
    }

    /// Queues up a message to be sent out in the next round.
    ///
    /// Queued messages are guaranteed to be processed in the next round after this one. So:
    ///
    /// - Other running agents won't see this event until next round
    /// - Agents [`Self::spawn`]ed this round **might not** see it (see that method for why)
    pub fn queue(&mut self, msg: M) -> &mut Self {
        self.messages.push(msg);
        self
    }

    /// [`Self::queue`]s up several messages to be sent out in the next round.
    pub fn queue_all(&mut self, msgs: impl IntoIterator<Item = M>) -> &mut Self {
        self.messages.extend(msgs);
        self
    }
}

#[cfg(feature = "test_extras")]
/// Some **test-only** functionality letting you introspect `Replies`, to test that `Agent`s or `Game`s are reacting
/// properly to things that happen.
impl<M: Message> Replies<M> {
    /// A **test-only** function, listing the messages that have been [`Self::queue`]d.
    pub fn _messages(&self) -> &[M] {
        &self.messages
    }

    /// A **test-only** function, listing the agents that have been [`Self::spawn`]ed.
    pub fn _agents(&self) -> &[Box<dyn Agent<M>>] {
        &self.agents
    }
}
