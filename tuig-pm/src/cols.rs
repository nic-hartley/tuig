use proc_macro2::{TokenStream, Span};
use syn::{parse::{ParseStream, Parser}, LitStr, LitInt};

// would be cool if I didn't have to copy/paste the definition but eh.
#[derive(Default)]
pub struct ColsData {
    widths: Vec<usize>,
    // TODO: &'static [fmt::Cell] separators?
    presep: String,
    seps: Vec<String>,
}

fn parse_sep(input: ParseStream) -> syn::Result<String> {
    let lh = input.lookahead1();
    if lh.peek(LitStr) {
        input.parse::<LitStr>().map(|s| s.value())
    } else {
        Ok("".into())
    }
}

fn parse_width(input: ParseStream) -> syn::Result<(Span, usize)> {
    let lh = input.lookahead1();
    if lh.peek(LitInt) {
        input.parse::<LitInt>().and_then(|i| i.base10_parse().map(|n| (i.span(), n)))
    } else if lh.peek(syn::Token!(*)) {
        let t = input.parse::<syn::Token!(*)>()?;
        Ok((t.span, 0))
    } else {
        Err(input.error("expected a usize or *"))
    }
}

fn parse_cols(input: ParseStream) -> syn::Result<ColsData> {
    let mut res = ColsData::default();
    let mut has_star = false;
    res.presep = input.call(parse_sep)?;
    while !input.is_empty() {
        let (t, w) = input.call(parse_width)?;
        res.widths.push(w);
        if w == 0 {
            if !has_star {
                has_star = true;
            } else {
                return Err(syn::Error::new(t, "maximum one * per cols!()"));
            }
        }
        res.seps.push(input.call(parse_sep)?);
    }
    Ok(res)
}

pub fn cols(input: TokenStream) -> TokenStream {
    let ColsData { widths, presep, seps } = match parse_cols.parse2(input) {
        Ok(d) => d,
        Err(e) => return e.to_compile_error(),
    };

    // TODO: This import probably needs to be more robust but I'm not totally sure how to do it.
    // Maybe add `macro_rules! cols { ($($params:tt)*) => { tuig_pm::cols!(in $crate: $($params)*) } }` if that can
    // get namespaced nicely?
    quote::quote! {
        #[allow(deprecated)]
        tuig_iosys::ui::splitters::cols::Cols::new(
            [ #(#widths),* ],
            #presep,
            [ #(#seps),* ],
        )
    }
}
