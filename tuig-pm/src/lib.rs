use itertools::Itertools;
use proc_macro2::{Span, TokenTree, TokenStream};
use syn::{parse::{ParseStream, Parser}, Attribute, LitStr, Token};

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

fn do_make_load(input: TokenStream) -> TokenStream {
    // parse each of the loaders
    let LoadInput { attrs, arms } = match parse_load.parse(input.into()) {
        Ok(li) => li,
        Err(e) => return e.to_compile_error(),
    };
    // figure out the individual `match` chunks for each feature
    let chunks = arms.into_iter().map(|(feat, init)| {
        (feat.clone(), quote::quote! {
            match ( #( #init )* ) {
                Ok((iosys, iorun)) => break Ok($($callback)* (iosys, iorun)),
                Err(e) => { errs.insert(#feat, e); }
            }
        })
    }).collect::<Vec<_>>();
    // generate each combination of 1 to n
    let mut options: Vec<TokenStream> = vec![];
    for n in 1..=chunks.len() {
        for c in chunks.iter().combinations(n) {
            let features = c.iter().map(|(f, _)| f).collect::<Vec<_>>();
            let antifeatures = chunks.iter().map(|(f, _)| f).filter(|f| !features.contains(f));
            let tokens = c.iter().map(|(_, ts)| ts);
            options.push(quote::quote! {
                #[cfg(all(not(any( #( feature = #antifeatures ),* )), #( feature = #features ),* ))]
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

#[proc_macro]
pub fn make_load(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    do_make_load(input.into()).into()
}
