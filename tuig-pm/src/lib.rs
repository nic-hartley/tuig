macro_rules! mod_fn {
    ( $(
        $( #[ $( $m:meta ),* $(,)? ] )*
        pub fn $name:ident($( $arg:ident: $ty:ty ),* $(,)?) $( -> $ret:ty )?
    );* $(;)? ) => { $(
        mod $name;
        $( #[ $( $m ),* ] )*
        pub fn $name($( $arg: $ty ),*) $( -> $ret )? {
            $name::$name($( $arg.into() ),*).into()
        }
    )* }
}

mod_fn! {
    #[proc_macro]
    pub fn make_load(input: proc_macro::TokenStream) -> proc_macro::TokenStream;
    #[proc_macro]
    pub fn force_docs_nightly(_input: proc_macro::TokenStream) -> proc_macro::TokenStream;
}

fn is_nightly() -> bool {
    use rustc_version::{version_meta, Channel, VersionMeta};
    matches!(
        version_meta(),
        Ok(VersionMeta {
            channel: Channel::Nightly,
            ..
        })
    )
}
