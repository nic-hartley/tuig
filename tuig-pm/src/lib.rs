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
    proc_macro force_docs_nightly,
}

mod splitters;
#[proc_macro]
pub fn cols(input: TokenStream) -> TokenStream {
    splitters::splitter(
        quote::quote!(ui::splitters::statics::Cols),
        input.into(),
    )
    .into()
}
#[proc_macro]
pub fn rows(input: TokenStream) -> TokenStream {
    splitters::splitter(
        quote::quote!(ui::splitters::statics::Rows),
        input.into(),
    )
    .into()
}

fn is_nightly() -> bool {
    use rustc_version::{version_meta, Channel};
    version_meta()
        .map(|vm| vm.channel == Channel::Nightly)
        .unwrap_or(false)
}
