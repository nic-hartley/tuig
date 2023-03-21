mod make_load;

#[proc_macro]
pub fn make_load(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    make_load::do_make_load(input.into()).into()
}
