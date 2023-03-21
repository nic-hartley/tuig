//! Miscellaneous stuff (especially macros) which are used across the project and needed a home

#![allow(unused)]

/// Shorter, bulk syntax for writing simple setters, which set a field to an argument or constant value.
macro_rules! setters {
    ( $(
        $name:ident $( ( $($pname:ident: $ptype:ty),* $(,)? ) )?  => $field:ident $( .$subfield:ident )* = $value:expr
    ),* $(,)? ) => {
        $(
            #[cfg_attr(coverage, no_coverage)]
            pub fn $name(mut self $( , $( $pname: $ptype ),* )?) -> Self {
                self.$field $( .$subfield )* = $value;
                self
            }
        )*
    };
}

/// Short syntax for feature-gated function calls
macro_rules! feature_switch {
    ( $( $feature:literal => $call:expr ),* $(,)? ) => { loop {
        $(
            #[cfg(feature = $feature)]
            {
                break $call;
            }
        )*
        unreachable!("feature_switch! but no features enabled!");
    } }
}

pub(crate) use {feature_switch, setters};
