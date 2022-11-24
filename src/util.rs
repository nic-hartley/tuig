//! Miscellaneous stuff (especially macros) which are used across the project and needed a home

macro_rules! setters {
    ( $(
        $name:ident $( ( $($pname:ident: $ptype:ty),* $(,)? ) )?  => $field:ident $( .$subfield:ident )* = $value:expr
    ),* $(,)? ) => {
        $(
            pub fn $name(mut self $( , $( $pname: $ptype ),* )?) -> Self {
                self.$field $( .$subfield )* = $value;
                self
            }
        )*
    };
}

pub(crate) use setters;
