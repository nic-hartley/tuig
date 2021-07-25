use crate::events::Events;

pub trait System {
    /// Process that an event has completed and update internal state
    /// accordingly (or discard it, if it's irrelevant.)
    ///
    /// Returns any events to be dispatched in response. If any, at the moment
    /// of this returning, are `complete`, they'll be processed in the same
    /// frame. Otherwise they'll be queued like normal.
    fn recv(&mut self, event: &Events) -> Vec<Events>;
}

macro_rules! systems_enum {
    (
        $enum_name:ident,
        $( $name:ident($type:path) ),*
        $(,)?
    ) => {
        #[derive(Clone, Debug)]
        pub enum $enum_name {
            $(
                $name($type)
            ),*
        }

        impl System for $enum_name {
            fn recv(&mut self, event: &crate::events::Events) -> Vec<crate::events::Events> {
                match self {
                    $(
                        $enum_name::$name(s) => s.recv(event)
                    ),*
                }
            }
        }
    }
}

systems_enum! {
    Systems,
    ChatSystem(crate::gameplay::chat::ChatSystem),
}
