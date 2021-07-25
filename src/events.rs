use std::fmt::Debug;

pub trait Event: Clone + Debug {
    fn complete(&self) -> bool;
}

macro_rules! events_enum {
    (
        $enum_name:ident,
        { $(
            $sys_name:ident $sys_contents:tt
        ),* $(,)? },
        $( $name:ident($type:path) ),*
        $(,)?
    ) => {
        #[derive(Clone, Debug)]
        pub enum $enum_name {
            $(
                $sys_name $sys_contents,
            )*
            $(
                $name($type),
            )*
        }

        impl $enum_name {
            pub fn is_system(&self) -> bool {
                match self {
                    $(
                        $enum_name::$sys_name(_) => true,
                    )*
                    $(
                        $enum_name::$name(_) => false,
                    )*
                }
            }
        }

        impl Event for $enum_name {
            fn complete(&self) -> bool {
                match self {
                    $(
                        $enum_name::$sys_name(_) => true,
                    )*
                    $(
                        $enum_name::$name(s) => s.complete(),
                    )*
                }
            }
        }
    }
}

events_enum! {
    Events,
    {
        AddSystem(crate::systems::Systems),
    },
    ChatMessage(crate::gameplay::chat::ChatMessage),
}
