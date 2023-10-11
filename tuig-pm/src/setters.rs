use proc_macro2::{TokenStream, Ident};
use syn::{Expr, parse::{ParseStream, Parse, Parser}, Token, parenthesized, FnArg, Attribute, punctuated::Punctuated, PatType, token::Paren};

/*
/// Shorter, bulk syntax for writing simple setters, which set a field to an argument or constant value.
macro_rules! setters {
    ( $(
        $( #[ $( $meta:meta ),* ] )*
        $name:ident $( ( $($pname:ident: $ptype:ty),* $(,)? ) )?  => $field:ident $( .$subfield:ident )* = $value:expr
    ),* $(,)? ) => {
        $(
            $( #[ $( $meta ),* ] )*
            #[cfg_attr(coverage, no_coverage)]
            pub fn $name(mut self $( , $( $pname: $ptype ),* )?) -> Self {
                self.$field $( .$subfield )* = $value;
                self
            }
        )*
    };
}
*/

struct Setter {
    attrs: Vec<Attribute>,
    name: Ident,
    params: Vec<PatType>,
    field: Vec<Ident>,
    value: Expr,
}

impl Parse for Setter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let name = input.parse()?;
        // when dtolnay isn't busy shouting down better ideas for reflection
        // than raw-dog source code parsing, he shits out awful apis that need
        // workarounds and hacks like this garbage.
        let params = if input.peek(Paren) {
            let content;
            parenthesized!(content in input);
            content.parse_terminated(FnArg::parse, Token![,])?
                .into_iter()
                .map(|fa| match fa {
                    FnArg::Receiver(r) => Err(syn::Error::new(r.self_token.span, "do not include 'self'")),
                    FnArg::Typed(t) => Ok(t),
                })
                .collect::<Result<_, _>>()?
        } else {
            vec![]
        };
        input.parse::<Token![=>]>()?;
        let mut field = vec![input.parse()?];
        while !input.peek(Token![=]) {
            input.parse::<Token![.]>()?;
            field.push(input.parse()?);
        }
        input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(Self { attrs, name, params, field, value })
    }
}

pub fn setters(input: TokenStream) -> TokenStream {
    let setters: Punctuated::<Setter, Token![,]> = match Punctuated::parse_terminated.parse2(input) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error(),
    };
    setters.into_iter().map(|setter| {
        let Setter { attrs, name, params, field, value } = setter;
        quote::quote! {
            #( #attrs )*
            pub fn #name(mut self, #( #params ),*) -> Self {
                self.#( #field )* = #value;
                self
            }
        }
    }).collect()
}
