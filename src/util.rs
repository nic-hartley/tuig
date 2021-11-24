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
    (@member $self:ident $f:ident; write $always:ident) => {
        write!($f, concat!(stringify!($always), ": {:?}, "), $self.$always)?
    };

    (@member $self:ident $f:ident; ignore $always:ident) => {
        write!($f, concat!(stringify!($ignore), ": .., "))?
    };

    (@member $self:ident $f:ident; if $sometimes:ident != $default:expr) => {
        if $self.$sometimes != $default {
            write!($f, concat!(stringify!($sometimes), ": {:?}, "), $self.$sometimes)?
        }
    };

    (
        $class:ident $( < $( $lt:lifetime ),* > )?;
        $( $key:ident $member:ident $( != $e:expr )? ),* $(,)?
    ) => {
        impl $( < $( $lt ),* > )?  fmt::Debug for $class $( < $( $lt ),* > )? {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!(stringify!($class), " {{ "))?;
                $(
                    $crate::util::abbrev_debug!(@member self f; $key $member $( != $e )?);
                )*
                write!(f, ".. }}")
            }
        }
    };
}

pub(crate) use {setters, abbrev_debug};
