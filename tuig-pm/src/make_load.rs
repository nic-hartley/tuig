use itertools::Itertools;
use proc_macro2::{Span, TokenStream, TokenTree};
use syn::{
    parse::{ParseStream, Parser},
    Attribute, LitStr, Token,
};

struct LoadInput {
    attrs: Vec<Attribute>,
    arms: Vec<(String, Vec<TokenTree>)>,
}

fn parse_load(input: ParseStream) -> syn::Result<LoadInput> {
    let attrs = input.call(Attribute::parse_outer)?;
    let mut arms = vec![];
    while !input.is_empty() {
        let feat = input.parse::<LitStr>()?.value();
        input.parse::<Token![=>]>()?;
        let mut body = vec![];
        while !input.peek(Token![,]) && !input.is_empty() {
            body.push(input.parse::<TokenTree>()?)
        }
        // extract if it's available, otherwise we exit on the next loop anyway
        let _ = input.parse::<Token![,]>();
        arms.push((feat, body));
    }
    if !input.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            "unexpected extras after match arms",
        ));
    }
    Ok(LoadInput { attrs, arms })
}

pub fn make_load(input: TokenStream) -> TokenStream {
    // parse each of the loaders
    let LoadInput { attrs, arms } = match parse_load.parse2(input) {
        Ok(li) => li,
        Err(e) => return e.to_compile_error(),
    };
    // figure out the individual `match` chunks for each feature
    let chunks = arms
        .iter()
        .map(|(feat, init)| {
            (
                feat.clone(),
                quote::quote! {
                    match ( #( #init )* ) {
                        Ok((iosys, iorun)) => break Ok($($callback)* (iosys, iorun)),
                        Err(e) => { errs.insert(#feat, e); }
                    }
                },
            )
        })
        .collect::<Vec<_>>();
    // generate each combination of 0 to n features
    (0..=chunks.len()).flat_map(|n| {
        chunks.iter().combinations(n).map(|c| {
            let features = c.iter().map(|(f, _)| f).collect::<Vec<_>>();
            let antifeatures = chunks
                .iter()
                .map(|(f, _)| f)
                .filter(|f| !features.contains(f));
            let cfgs = quote::quote! {
                #[cfg(all(not(any( #( feature = #antifeatures ),* )), #( feature = #features ),* ))]
                #[cfg_attr(doc, doc(cfg(has_backend)))]
            };
            let tokens = c.iter().map(|(_, ts)| ts);
            quote::quote! {
                #cfgs
                #( #attrs )*
                #[macro_export]
                macro_rules! load {
                    ($($callback:tt)*) => { loop {
                        #[allow(unused)]
                        let mut errs = $crate::BTreeMap::<&'static str, $crate::Error>::new();
                        #( #tokens )*
                        break Err(errs);
                    } }
                }
            }
        })
    }).collect()
}
