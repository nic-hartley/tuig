//! Miscellaneous stuff (especially macros) which are used across the project and needed a home

macro_rules! setters {
    ( $(
        $name:ident $( ( $($pname:ident: $ptype:ty),* $(,)? ) )?  => $field:ident = $value:expr
    ),* $(,)? ) => {
        $(
            pub fn $name(mut self $( , $( $pname: $ptype ),* )?) -> Self {
                self.$field = $value;
                self
            }
        )*
    };
}

macro_rules! abbrev_debug {
    (
        $class:ident $( < $( $lt:lifetime ),* > )?;
        $( write $always:ident, )*
        $( ignore $ignore:ident, )*
        $( if $sometimes:ident != $default:expr, )*
    ) => {
        impl $( < $( $lt ),* > )?  fmt::Debug for $class $( < $( $lt ),* > )? {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!(stringify!($class), " {{ "))?;
                $(
                    write!(f, concat!(stringify!($always), ": {:?}, "), self.$always)?;
                )*
                $(
                    write!(f, concat!(stringify!($ignore), ": .., "))?;
                )*
                $(
                    if self.$sometimes != $default {
                        write!(f, concat!(stringify!($sometimes), ": {:?}, "), self.$sometimes)?;
                    }
                )*
                write!(f, ".. }}")
            }
        }
    }
}

pub(crate) use {setters, abbrev_debug};
