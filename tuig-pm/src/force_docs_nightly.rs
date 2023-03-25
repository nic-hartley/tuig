use proc_macro2::TokenStream;

pub fn force_docs_nightly(_input: TokenStream) -> TokenStream {
    if super::is_nightly() {
        quote::quote!()
    } else {
        quote::quote! {
            #[cfg(doc)]
            compile_error!("this crate can only be documented on nightly");
        }
    }
}