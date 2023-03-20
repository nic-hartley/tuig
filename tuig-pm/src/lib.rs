use proc_macro2::{Span, TokenTree};
use syn::{parse::ParseStream, parse_macro_input, Attribute, LitStr, Token};

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

#[proc_macro]
pub fn make_load(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let LoadInput { attrs, arms } = parse_macro_input!(input with parse_load);
    let chunks = arms.into_iter().map(|(feat, init)| {
        quote::quote! {
            #[cfg(feature = #feat)] {
                match ( #( #init )* ) {
                    Ok((iosys, iorun)) => break Ok($callback(iosys, iorun)),
                    Err(e) => { errs.insert(#feat, e); }
                }
            }
        }
    });
    quote::quote! {
        #( #attrs )*
        #[macro_export]
        macro_rules! load {
            ($callback: expr) => { loop {
                #[allow(unused)]
                let mut errs = $crate::BTreeMap::<&'static str, $crate::Error>::new();
                #( #chunks )*
                break Err(errs);
            } };
        }
    }
    .into()
}
