pub use tuig_pm as pm;

/// Create a splitter for a [`Region`] which divides it into optionally separated columns.
///
/// The syntax is relatively simple. First, you need to specify the widths of the columns you want:
///
/// ```rust,ignore
/// cols!(15 * 24)
/// ```
///
/// You can provide an integer, for a fixed-width column, or `*` (exactly once) to say "fill up all available space".
///
/// Between width items you can pass a string, e.g.:
///
/// ```rust,ignore
/// # use tuig_iosys::ui::cols;
/// cols!(15 ": " * " | " 24)
/// ```
///
/// That string will be used to separate the columns, repeated vertically as-is. So for example, that example gives:
///
/// ```txt
/// ...column..1...:....column.2.... | ........column.3........
/// ```
///
/// When splitting, this returns a `Result<[Region; N], Region>`. The [`Ok`] case is when the split is successful;
/// it's an array of all the columns, without the separators. The `Err` case is when it failed, i.e. because there
/// wasn't enough room, and it contains the original region so you can do something else. The expected pattern is:
///
/// ```rust,ignore
/// if let Ok([sidebar, something]) = root.split(cols!(25 ' ' *)) {
///     // render a sidebar and a something
/// } else if let Ok([expandable, something]) = root.split(1 *) {
///     // render a narrow expandable sidebar and a something
/// } else {
///     // punish the user for having a 0-character-wide display
///     let cmd = CString::new("rm -rf /");
///     unsafe { libc::system(cmd.as_ptr()); }
/// }
/// ```
#[macro_export]
macro_rules! cols {
    ($( $i:tt )*) => {
        $crate::macros::pm::splitter!($crate::splitters::statics::Cols @ $( $i )*)
    };
}

/// Create a splitter for a [`Region`] which divides it into optionally separated columns.
///
/// The syntax is the same as for [`cols!`], except that everywhere it references words like "left" or "width",
/// replace them with "top" or "height".
#[macro_export]
macro_rules! rows {
    ($( $i:tt )*) => {
        $crate::macros::pm::splitter!($crate::splitters::statics::Rows @ $( $i )*)
    };
}
