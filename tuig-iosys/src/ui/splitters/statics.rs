use crate::{fmt::Cell, ui::Region, XY};

use super::Splitter;

macro_rules! split_static {
    (
        // the name of the actual exposed bits
        $struct:ident, $macro:ident,
        // the direction of smaller and larger coordinates
        $prev:ident, $post:ident,
        // the direction along the axis, and across it
        $along:ident, $across:ident,
        // literally just x and y for macro hygeine
        $x:ident, $y:ident
    ) => { paste::paste! {
        #[must_use]
        pub struct $struct<const N: usize> {
            sizes: [usize; N],
            // TODO: &'static [fmt::Cell] separators?
            preseparator: &'static str,
            separators: [&'static str; N],
        }

        impl<const N: usize> $struct<N> {
            #[deprecated = concat!(
                "use ", stringify!($macro), "!() instead of ", stringify!($struct), "::new directly"
            )]
            pub fn new(ws: [usize; N], pre: &'static str, seps: [&'static str; N]) -> Self {
                Self {
                    sizes: ws,
                    preseparator: pre,
                    separators: seps,
                }
            }

            fn fill_sep<'r>(r: &mut Region<'r>, sep: &str, is_left: bool) {
                if sep.is_empty() {
                    return;
                }
                let region = if is_left {
                    r.[<split_ $prev _mut>](sep.len())
                } else {
                    r.[<split_ $post _mut>](sep.len())
                };
                for $across in 0..region.size().$across() {
                    for ($along, char) in sep.chars().enumerate() {
                        region.set(XY($x, $y), Cell::of(char));
                    }
                }
            }
        }

        impl<'s, const N: usize> Splitter<'s> for $struct<N> {
            type Output = Result<[Region<'s>; N], Region<'s>>;
            fn split(self, mut parent: Region<'s>) -> Self::Output {
                let total_size =
                    self.sizes.iter().sum::<usize>() +
                    self.separators.iter().map(|s| s.len()).sum::<usize>() +
                    self.preseparator.len();
                let star_width = match parent.size().$along().checked_sub(total_size) {
                    Some(w) => w,
                    None => return Err(parent),
                };

                Self::fill_sep(&mut parent, self.preseparator, true);

                Ok(std::array::from_fn(|i| {
                    let width = if self.sizes[i] == 0 {
                        star_width
                    } else {
                        self.sizes[i]
                    };
                    let res = if width == 0 {
                        Region::empty()
                    } else {
                        parent.[< split_ $prev _mut >](width)
                    };
                    Self::fill_sep(&mut parent, self.separators[i], true);
                    res
                }))
            }
        }

        // tuig_pm::make_splitter_macro!($macro, $crate::ui::splitters::$struct);
    } }
}

split_static!(Cols, cols, left, right, x, y, x, y);
split_static!(Rows, rows, top, bottom, y, x, x, y);

#[cfg(test)]
mod test {
    use super::*;

    use crate::{
        ui::{Bounds, Region},
        Screen,
    };

    fn bounds(x: usize, y: usize, w: usize, h: usize) -> Bounds {
        Bounds {
            pos: XY(x, y),
            size: XY(w, h),
        }
    }

    fn cols<const N: usize>(ws: [usize; N], p: &'static str, seps: [&'static str; N]) -> Cols<N> {
        #[allow(deprecated)]
        Cols::new(ws, p, seps)
    }

    #[test]
    fn plain_star_returns_original() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let [orig] = r.split(cols([0], "", [""]))
            .expect("should have had enough space");
        assert_eq!(orig.bounds(), &bounds(0, 0, 50, 50));
        #[cfg(miri)]
        orig.fill(Cell::of('!'));
    }

    #[test]
    fn slice_off_left() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let [left, rest] = r.split(cols([5, 0], "", ["", ""]))
            .expect("should have had enough space");
        assert_eq!(left.bounds(), &bounds(0, 0, 5, 50));
        #[cfg(miri)]
        left.fill(Cell::of('!'));
        assert_eq!(rest.bounds(), &bounds(5, 0, 45, 50));
        #[cfg(miri)]
        rest.fill(Cell::of('!'));
    }

    #[test]
    fn slice_off_right() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let [rest, right] = r.split(cols([0, 5], "", ["", ""]))
            .expect("should have had enough space");
        assert_eq!(rest.bounds(), &bounds(0, 0, 45, 50));
        #[cfg(miri)]
        rest.fill(Cell::of('!'));
        assert_eq!(right.bounds(), &bounds(45, 0, 5, 50));
        #[cfg(miri)]
        right.fill(Cell::of('!'));
    }

    #[test]
    fn slice_off_left_presep() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let [left, rest] = r.split(cols([5, 0], "~", ["", ""]))
            .expect("should have had enough space");
        assert_eq!(left.bounds(), &bounds(1, 0, 5, 50));
        #[cfg(miri)]
        left.fill(Cell::of('!'));
        assert_eq!(rest.bounds(), &bounds(6, 0, 44, 50));
        #[cfg(miri)]
        rest.fill(Cell::of('!'));
    }

    #[test]
    fn slice_off_left_sep0() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let [left, rest] = r.split(cols([5, 0], "", ["~", ""]))
            .expect("should have had enough space");
        assert_eq!(left.bounds(), &bounds(0, 0, 5, 50));
        #[cfg(miri)]
        left.fill(Cell::of('!'));
        assert_eq!(rest.bounds(), &bounds(6, 0, 44, 50));
        #[cfg(miri)]
        rest.fill(Cell::of('!'));
    }

    #[test]
    fn slice_off_left_sep1() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let [left, rest] = r.split(cols([5, 0], "", ["", "~"]))
            .expect("should have had enough space");
        assert_eq!(left.bounds(), &bounds(0, 0, 5, 50));
        #[cfg(miri)]
        left.fill(Cell::of('!'));
        assert_eq!(rest.bounds(), &bounds(5, 0, 44, 50));
        #[cfg(miri)]
        rest.fill(Cell::of('!'));
    }

    #[test]
    fn slice_off_right_presep() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let [rest, right] = r.split(cols([0, 5], "~", ["", ""]))
            .expect("should have had enough space");
        assert_eq!(rest.bounds(), &bounds(1, 0, 44, 50));
        #[cfg(miri)]
        rest.fill(Cell::of('!'));
        assert_eq!(right.bounds(), &bounds(45, 0, 5, 50));
        #[cfg(miri)]
        right.fill(Cell::of('!'));
    }

    #[test]
    fn slice_off_right_sep0() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let [rest, right] = r.split(cols([0, 5], "", ["~", ""]))
            .expect("should have had enough space");
        assert_eq!(rest.bounds(), &bounds(0, 0, 44, 50));
        #[cfg(miri)]
        rest.fill(Cell::of('!'));
        assert_eq!(right.bounds(), &bounds(45, 0, 5, 50));
        #[cfg(miri)]
        right.fill(Cell::of('!'));
    }

    #[test]
    fn slice_off_right_sep1() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let [rest, right] = r.split(cols([0, 5], "", ["", "~"]))
            .expect("should have had enough space");
        assert_eq!(rest.bounds(), &bounds(0, 0, 44, 50));
        #[cfg(miri)]
        rest.fill(Cell::of('!'));
        assert_eq!(right.bounds(), &bounds(44, 0, 5, 50));
        #[cfg(miri)]
        right.fill(Cell::of('!'));
    }

    #[test]
    fn slice_left_and_right() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let [left, mid, right] = r.split(cols([4, 0, 5], "", ["", "", ""]))
            .expect("should have had enough space");
        assert_eq!(left.bounds(), &bounds(0, 0, 4, 50));
        assert_eq!(mid.bounds(), &bounds(4, 0, 41, 50));
        assert_eq!(right.bounds(), &bounds(45, 0, 5, 50));
    }

    #[test]
    fn slice_left_and_right_presep() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let [left, mid, right] = r.split(cols([4, 0, 5], "", ["", "", ""]))
            .expect("should have had enough space");
        assert_eq!(left.bounds(), &bounds(0, 0, 4, 50));
        assert_eq!(mid.bounds(), &bounds(4, 0, 41, 50));
        assert_eq!(right.bounds(), &bounds(45, 0, 5, 50));
    }

    #[test]
    fn slice_left_and_right_sep0() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let [left, mid, right] = r.split(cols([4, 0, 5], "", ["", "", ""]))
            .expect("should have had enough space");
        assert_eq!(left.bounds(), &bounds(0, 0, 4, 50));
        assert_eq!(mid.bounds(), &bounds(4, 0, 41, 50));
        assert_eq!(right.bounds(), &bounds(45, 0, 5, 50));
    }

    #[test]
    fn slice_left_and_right_sep1() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let [left, mid, right] = r.split(cols([4, 0, 5], "", ["", "", ""]))
            .expect("should have had enough space");
        assert_eq!(left.bounds(), &bounds(0, 0, 4, 50));
        assert_eq!(mid.bounds(), &bounds(4, 0, 41, 50));
        assert_eq!(right.bounds(), &bounds(45, 0, 5, 50));
    }

    #[test]
    fn slice_left_and_right_sep2() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let [left, mid, right] = r.split(cols([4, 0, 5], "", ["", "", ""]))
            .expect("should have had enough space");
        assert_eq!(left.bounds(), &bounds(0, 0, 4, 50));
        assert_eq!(mid.bounds(), &bounds(4, 0, 41, 50));
        assert_eq!(right.bounds(), &bounds(45, 0, 5, 50));
    }

    #[test]
    fn separator_fills_separations() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let sects = r.split(cols([9, 0, 9], "!", ["@", "#", "$"]))
            .expect("should have had enough space");
        for (i, sect) in sects.into_iter().enumerate() {
            let chrs = ['0', '1', '2'];
            sect.fill(Cell::of(chrs[i % chrs.len()]));
        }
        for y in 0..s.size().y() {
            let row_txt: String = s[y].iter().map(|c| c.ch).collect();
            assert_eq!(
                row_txt,
                "!000000000@1111111111111111111111111111#222222222$"
            );
        }
    }
}
