use crate::events::Events;

pub trait App {
    /// Process that an event has completed and update internal state
    /// accordingly (or discard it, if it's irrelevant.)
    ///
    /// Returns whether or not it'll need to be re-rendered as a result of the
    /// event. Note returning `true` doesn't guarantee an immediate re-render;
    /// receiving events will be batched and re-rendering will occur at most
    /// once if the app is currently focused.
    fn recv(&mut self, event: &Events) -> bool;
    /// Take some user input.
    // TODO: Take a keypress instead of a line of input
    fn input(&mut self, data: String) -> Vec<Events>;
    /// Actually render the app to screen.
    // TODO: Render to a Screen, not a string
    fn render(&self, into: &mut String);
}

macro_rules! apps_enum {
    (
        $enum_name:ident,
        $( $name:ident($type:path) ),*
        $(,)?
    ) => {
        pub enum $enum_name {
            $(
                $name($type)
            ),*
        }

        impl App for $enum_name {
            fn recv(&mut self, event: &crate::events::Events) -> bool {
                match self {
                    $(
                        $enum_name::$name(s) => s.recv(event)
                    ),*
                }
            }
            fn input(&mut self, data: String) -> Vec<Events> {
                match self {
                    $(
                        $enum_name::$name(s) => s.input(data)
                    ),*
                }
            }
            fn render(&self, into: &mut String) {
                match self {
                    $(
                        $enum_name::$name(s) => s.render(into)
                    ),*
                }
            }
        }
    }
}

apps_enum! {
    Apps,
    ChatApp(crate::gameplay::chat::ChatApp),
}
