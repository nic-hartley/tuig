//! You probably want one of the other crates. See [the repo](https://github.com/nic-hartley/tuig) for help choosing.

use proc_macro::TokenStream;

macro_rules! convert {
    (
        $( #[ $( $m:meta ),* $(,)? ] )*
        fn $name:ident ( $( $arg:ident: $ty:ty ),* $(,)? ) $( -> $ret:ty )?
    ) => {
        mod $name;
        $( #[ $( $m ),* ] )*
        pub fn $name($( $arg: $ty ),*) $( -> $ret )? {
            $name::$name($( $arg.into() ),*).into()
        }
    };
}

macro_rules! mod_fn {
    ( $(
        $kind:ident $( ( $type:ident ) )? $name:ident
    ),* $(,)? ) => { $(
        mod_fn! { @ $kind $( ( $type ) )? $name }
    )* };
    ( @ proc_macro $name:ident ) => {
        convert! {
            #[proc_macro]
            #[doc(hidden)]
            fn $name(input: TokenStream) -> TokenStream
        }
    };
    ( @ proc_macro_derive ( $type:ident ) $name:ident ) => {
        convert! {
            #[proc_macro_derive( $type )]
            #[doc(hidden)]
            fn $name(input: TokenStream) -> TokenStream
        }
    };
    ( @ proc_macro_attribute $name:ident ) => {
        convert! {
            #[proc_macro_attribute]
            #[doc(hidden)]
            fn $name(attr: TokenStream, item: TokenStream) -> TokenStream
        }
    };
}

mod_fn! {
    proc_macro make_load,
    proc_macro splitter,
    proc_macro setters,
}
