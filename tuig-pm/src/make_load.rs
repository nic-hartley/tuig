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
    let mut res = LoadInput {
        attrs: vec![],
        arms: vec![],
    };
    while let Ok(attr) = input.call(Attribute::parse_outer) {
        if attr.is_empty() {
            break;
        }
        res.attrs.extend(attr);
    }
    while !input.is_empty() {
        let feat = input.parse::<LitStr>()?.value();
        input.parse::<Token![=>]>()?;
        let mut body = vec![];
        while !input.peek(Token![,]) && !input.is_empty() {
            body.push(input.parse::<TokenTree>()?.into())
        }
        // extract if it's available, otherwise we exit on the next loop anyway
        let _ = input.parse::<Token![,]>();
        res.arms.push((feat, body));
    }
    if !input.is_empty() {
        return Err(syn::Error::new(
            Span::call_site().into(),
            "unexpected extras after match arms",
        ));
    }
    Ok(res)
}

pub fn make_load(input: TokenStream) -> TokenStream {
    // parse each of the loaders
    let LoadInput { attrs, arms } = match parse_load.parse(input.into()) {
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
    let all_feats = arms.into_iter().map(|(f, _)| f).collect::<Vec<_>>();
    // generate each combination of 1 to n
    let mut options: Vec<TokenStream> = vec![quote::quote! {
        /// Based on IO system features enabled, attempt to initialize an IO system, in the same manner as [`load!`].
        ///
        /// This returns things boxed so they can be used as trait objects, which provides better ergonomics at the
        /// cost of slightly lower max performance.
        #[cfg(any( #( feature = #all_feats ),* ))]
        pub fn load() -> core::result::Result<(Box<dyn IoSystem>, Box<dyn IoRunner>), BTreeMap<&'static str, Error>> {
            #[allow(unused)]
            fn cb(
                sys: impl IoSystem + 'static,
                run: impl IoRunner + 'static,
            ) -> (Box<dyn IoSystem>, Box<dyn IoRunner>) {
                (Box::new(sys), Box::new(run))
            }
            load!(cb)
        }

        #[cfg(not(any( #( feature = #all_feats ),* )))]
        #[macro_export]
        macro_rules! load {
            ($($callback:tt)*) => {
                compile_error!("select an IO system to use tuig_iosys::load");
            }
        }
    }];
    for n in 1..=chunks.len() {
        for c in chunks.iter().combinations(n) {
            let features = c.iter().map(|(f, _)| f).collect::<Vec<_>>();
            let antifeatures = chunks
                .iter()
                .map(|(f, _)| f)
                .filter(|f| !features.contains(f));
            let tokens = c.iter().map(|(_, ts)| ts);
            let doc_cfg = if super::is_nightly() {
                quote::quote! { #[cfg_attr(doc, doc(cfg(any( #( feature = #all_feats ),* ))))] }
            } else {
                quote::quote! {}
            };
            options.push(quote::quote! {
                #[cfg(all(not(any( #( feature = #antifeatures ),* )), #( feature = #features ),* ))]
                #doc_cfg
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
            }.into())
        }
    }
    options.into_iter().collect()
}
