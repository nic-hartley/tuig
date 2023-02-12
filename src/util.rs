//! Miscellaneous stuff (especially macros) which are used across the project and needed a home

/// Shorter, bulk syntax for writing simple setters, which set a field to an argument or constant value.
macro_rules! setters {
    ( $(
        $name:ident $( ( $($pname:ident: $ptype:ty),* $(,)? ) )?  => $field:ident $( .$subfield:ident )* = $value:expr
    ),* $(,)? ) => {
        $(
            #[cfg_attr(coverage, no_coverage)]
            pub fn $name(mut self $( , $( $pname: $ptype ),* )?) -> Self {
                self.$field $( .$subfield )* = $value;
                self
            }
        )*
    };
}

pub(crate) use setters;

/// The same as [`Vec::retain`], but doesn't attempt to preserve the ordering of elements.
pub(crate) fn retain_unstable<T, P: Fn(&T) -> bool>(this: &mut Vec<T>, cond: P) {
    let mut new_size = this.len();
    let mut idx = 0;
    while idx < new_size {
        let elem = &this[idx];
        if cond(elem) {
            idx += 1;
        } else {
            this.swap(idx, new_size - 1);
            new_size -= 1;
        }
    }
    this.truncate(new_size);
}
