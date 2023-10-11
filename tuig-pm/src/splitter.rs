use proc_macro2::{Span, TokenStream, TokenTree};
use syn::{
    parse::{ParseStream, Parser},
    LitInt, LitStr, Token,
};

// would be cool if I didn't have to copy/paste the definition but eh.
#[derive(Default)]
pub struct SplitSpec {
    base: TokenStream,
    sizes: Vec<usize>,
    // TODO: &'static [fmt::Cell] separators?
    presep: String,
    seps: Vec<String>,
}

fn parse_base(input: ParseStream) -> syn::Result<TokenStream> {
    let mut res = vec![];
    while !input.lookahead1().peek(Token![@]) {
        res.push(input.parse::<TokenTree>()?)
    }
    let _ = input.parse::<Token![@]>()?;
    Ok(quote::quote!( #( #res )* ))
}

fn parse_sep(input: ParseStream) -> syn::Result<String> {
    let lh = input.lookahead1();
    if lh.peek(LitStr) {
        input.parse::<LitStr>().map(|s| s.value())
    } else {
        Ok("".into())
    }
}

fn parse_size(input: ParseStream) -> syn::Result<(Span, usize)> {
    let lh = input.lookahead1();
    if lh.peek(LitInt) {
        input
            .parse::<LitInt>()
            .and_then(|i| i.base10_parse().map(|n| (i.span(), n)))
    } else if lh.peek(syn::Token!(*)) {
        let t = input.parse::<syn::Token!(*)>()?;
        Ok((t.span, usize::MAX))
    } else {
        Err(input.error("expected a usize or *"))
    }
}

fn parse_splitspec(input: ParseStream) -> syn::Result<SplitSpec> {
    let mut res = SplitSpec {
        base: input.call(parse_base)?,
        presep: input.call(parse_sep)?,
        ..Default::default()
    };
    let mut has_star = false;
    while !input.is_empty() {
        let (t, w) = input.call(parse_size)?;
        res.sizes.push(w);
        if w == usize::MAX {
            if !has_star {
                has_star = true;
            } else {
                return Err(syn::Error::new(t, "maximum one * per split spec"));
            }
        }
        res.seps.push(input.call(parse_sep)?);
    }
    Ok(res)
}

pub fn splitter(input: TokenStream) -> TokenStream {
    let SplitSpec {
        base,
        sizes,
        presep,
        seps,
    } = match parse_splitspec.parse2(input) {
        Ok(d) => d,
        Err(e) => return e.to_compile_error(),
    };

    // TODO: This import probably needs to be more robust but I'm not totally sure how to do it.
    // Maybe add `macro_rules! cols { ($($params:tt)*) => { tuig_pm::cols!(in $crate: $($params)*) } }` if that can
    // get namespaced nicely?
    quote::quote! {
        #[allow(deprecated)]
        #base::new(
            [ #(#sizes),* ],
            #presep,
            [ #(#seps),* ],
        )
    }
}
