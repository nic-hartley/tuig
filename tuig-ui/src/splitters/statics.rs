use tuig_iosys::fmt::Cell;

use crate::{Region, ScreenView};

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

            fn fill_sep<'r>(r: &mut Region<'r>, sep: &str) {
                if sep.is_empty() {
                    return;
                }
                // TODO: Get rid of this allocation somehow, maybe with more preprocessing/macrofuckery
                let cells: alloc::vec::Vec<_> = sep.chars().map(|c| Cell::of(c)).collect();
                let region = r.[<split_ $prev _mut>](sep.len());
                region.attach(|_, mut sv: ScreenView| {
                    for $y in 0..sv.size().y() {
                        for $x in 0..sv.size().x() {
                            sv[$y][$x] = cells[$along].clone();
                        }
                    }
                })
            }
        }

        impl<'s, const N: usize> Splitter<'s> for $struct<N> {
            type Output = Result<[Region<'s>; N], Region<'s>>;
            fn split(self, mut parent: Region<'s>) -> Self::Output {
                let total_size =
                    self.sizes.iter().map(|&s| if s == usize::MAX { 0 } else { s }).sum::<usize>() +
                    self.separators.iter().map(|s| s.len()).sum::<usize>() +
                    self.preseparator.len();
                let star_width = match parent.size().$along().checked_sub(total_size) {
                    Some(w) => w,
                    None => return Err(parent),
                };

                Self::fill_sep(&mut parent, self.preseparator);

                Ok(core::array::from_fn(|i| {
                    let width = if self.sizes[i] == usize::MAX {
                        star_width
                    } else {
                        self.sizes[i]
                    };
                    let res = if width == 0 {
                        Region::empty()
                    } else {
                        parent.[< split_ $prev _mut >](width)
                    };
                    Self::fill_sep(&mut parent, self.separators[i]);
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
    use crate::{bounds::Bounds, cols, Region};

    use alloc::string::String;
    use tuig_iosys::{XY, Screen, Action, fmt::Cell};

    fn bounds(x: usize, y: usize, w: usize, h: usize) -> Bounds {
        Bounds {
            pos: XY(x, y),
            size: XY(w, h),
        }
    }

    #[test]
    fn plain_star_returns_original() {
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        let [orig] = r.split(cols!(*)).expect("should have had enough space");
        assert_eq!(orig.bounds(), &bounds(0, 0, 50, 50));
        #[cfg(miri)]
        orig.fill(Cell::of('!'));
    }

    #[test]
    fn slice_off_left() {
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        let [left, rest] = r.split(cols!(5 *)).expect("should have had enough space");
        assert_eq!(left.bounds(), &bounds(0, 0, 5, 50));
        #[cfg(miri)]
        left.fill(Cell::of('!'));
        assert_eq!(rest.bounds(), &bounds(5, 0, 45, 50));
        #[cfg(miri)]
        rest.fill(Cell::of('!'));
    }

    #[test]
    fn slice_off_right() {
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        let [rest, right] = r.split(cols!(*5)).expect("should have had enough space");
        assert_eq!(rest.bounds(), &bounds(0, 0, 45, 50));
        #[cfg(miri)]
        rest.fill(Cell::of('!'));
        assert_eq!(right.bounds(), &bounds(45, 0, 5, 50));
        #[cfg(miri)]
        right.fill(Cell::of('!'));
    }

    #[test]
    fn slice_off_left_presep() {
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        let [left, rest] = r
            .split(cols!("~" 5 *))
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
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        let [left, rest] = r
            .split(cols!(5 "~" *))
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
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        let [left, rest] = r
            .split(cols!(5 * "~"))
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
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        let [rest, right] = r
            .split(cols!("~" * 5))
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
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        let [rest, right] = r
            .split(cols!(* "~" 5))
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
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        let [rest, right] = r
            .split(cols!(* 5 "~"))
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
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        let [left, mid, right] = r.split(cols!(4 * 5)).expect("should have had enough space");
        assert_eq!(left.bounds(), &bounds(0, 0, 4, 50));
        assert_eq!(mid.bounds(), &bounds(4, 0, 41, 50));
        assert_eq!(right.bounds(), &bounds(45, 0, 5, 50));
    }

    #[test]
    fn slice_left_and_right_presep() {
        // TODO: Apply separators to all these tests
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        let [left, mid, right] = r
            .split(cols!("~" 4 * 5))
            .expect("should have had enough space");
        assert_eq!(left.bounds(), &bounds(1, 0, 4, 50));
        assert_eq!(mid.bounds(), &bounds(5, 0, 40, 50));
        assert_eq!(right.bounds(), &bounds(45, 0, 5, 50));
    }

    #[test]
    fn slice_left_and_right_sep0() {
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        let [left, mid, right] = r
            .split(cols!(4 "~" * 5))
            .expect("should have had enough space");
        assert_eq!(left.bounds(), &bounds(0, 0, 4, 50));
        assert_eq!(mid.bounds(), &bounds(5, 0, 40, 50));
        assert_eq!(right.bounds(), &bounds(45, 0, 5, 50));
    }

    #[test]
    fn slice_left_and_right_sep1() {
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        let [left, mid, right] = r
            .split(cols!(4 * "~" 5))
            .expect("should have had enough space");
        assert_eq!(left.bounds(), &bounds(0, 0, 4, 50));
        assert_eq!(mid.bounds(), &bounds(4, 0, 40, 50));
        assert_eq!(right.bounds(), &bounds(45, 0, 5, 50));
    }

    #[test]
    fn slice_left_and_right_sep2() {
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        let [left, mid, right] = r
            .split(cols!(4 * 5 "~"))
            .expect("should have had enough space");
        assert_eq!(left.bounds(), &bounds(0, 0, 4, 50));
        assert_eq!(mid.bounds(), &bounds(4, 0, 40, 50));
        assert_eq!(right.bounds(), &bounds(44, 0, 5, 50));
    }

    #[test]
    fn separator_fills_separations() {
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        let sects = r
            .split(cols!("!" 9 "@" * "#" 9 "$"))
            .expect("should have had enough space");
        for (i, sect) in sects.into_iter().enumerate() {
            let chrs = ['0', '1', '2'];
            sect.fill(Cell::of(chrs[i]));
        }
        for y in 0..s.size().y() {
            let row_txt: String = s[y].iter().map(|c| c.ch).collect();
            assert_eq!(
                row_txt,
                "!000000000@1111111111111111111111111111#222222222$"
            );
        }
    }

    #[test]
    fn split_just_enough_succeeds() {
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        let [l, r] = r.split(cols!(50 *)).expect("should have had enough space");
        assert_eq!(l.bounds(), &bounds(0, 0, 50, 50));
        assert_eq!(r.bounds(), &bounds(0, 0, 0, 0));
    }

    #[test]
    fn split_just_more_fails() {
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        r.split(cols!(50 1))
            .expect_err("should not have had enough space");
    }

    #[test]
    fn split_star_just_more_fails() {
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        r.split(cols!(51 *))
            .expect_err("should not have had enough space");
    }

    #[test]
    fn split_just_enough_plus_presep_fails() {
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        r.split(cols!("a" 50 *))
            .expect_err("should not have had enough space");
    }

    #[test]
    fn split_just_enough_plus_sep0_fails() {
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        r.split(cols!(50 "b" *))
            .expect_err("should not have had enough space");
    }

    #[test]
    fn split_just_enough_plus_sep1_fails() {
        let mut s = Screen::new(XY(50, 50));
        let r = Region::new(&mut s, Action::Redraw);
        r.split(cols!(50 * "c"))
            .expect_err("should not have had enough space");
    }
}
