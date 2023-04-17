use core::mem::MaybeUninit;

use crate::{fmt::Cell, ui::Region, XY};

use super::Splitter;

fn fill_sep<'r>(r: &mut Region<'r>, sep: &str, is_left: bool) {
    if sep.is_empty() {
        return;
    }
    let region = if is_left {
        r.split_left_mut(sep.len())
    } else {
        r.split_right_mut(sep.len())
    };
    for y in 0..region.size().y() {
        for (x, char) in sep.chars().enumerate() {
            region.set(XY(x, y), Cell::of(char));
        }
    }
}

pub struct Cols<const COLS: usize> {
    widths: [usize; COLS],
    // TODO: &'static [fmt::Cell] separators?
    preseparator: &'static str,
    separators: [&'static str; COLS],
}

impl<const COLS: usize> Cols<COLS> {
    #[deprecated = "use cols!() instead of ColsSplitter::new directly"]
    pub fn new(ws: [usize; COLS], pre: &'static str, seps: [&'static str; COLS]) -> Self {
        Self {
            widths: ws,
            preseparator: pre,
            separators: seps,
        }
    }

    fn split_widths(&self) -> (&[usize], usize, &[usize]) {
        if let Some(idx) = self.widths.iter().position(|&i| i == 0) {
            let (l, r) = self.widths.split_at(idx);
            (l, idx, &r[1..])
        } else {
            (&self.widths, COLS, &[])
        }
    }
}

impl<'s, const COLS: usize> Splitter<'s> for Cols<COLS> {
    type Output = [Region<'s>; COLS];
    fn split(self, mut parent: Region<'s>) -> Self::Output {
        // SAFETY: Arrays only require that each member be initialized as the member type requires, nothing extra.
        // `MaybeUninit` doesn't require it be initialized in any specific way, that's the whole point.
        // https://doc.rust-lang.org/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
        let mut res: [MaybeUninit<Region<'s>>; COLS] =
            unsafe { MaybeUninit::uninit().assume_init() };

        fill_sep(&mut parent, self.preseparator, true);

        let (lefts, star, rights) = self.split_widths();
        for (i, left) in lefts.iter().enumerate() {
            res[i].write(parent.split_left_mut(*left));
            fill_sep(&mut parent, self.separators[i], true);
        }
        for (i, right) in rights.iter().enumerate() {
            let i = (COLS - 1) - i;
            fill_sep(&mut parent, self.separators[i], false);
            res[i].write(parent.split_right_mut(*right));
        }
        // then there actually was one, add it
        if star != COLS {
            // we'll be missing the separator to its right
            fill_sep(&mut parent, self.separators[star], false);
            res[star].write(parent);
        }

        // SAFETY: At this point every region being returned has been initialized
        res.map(|mu| unsafe { mu.assume_init() })
    }
}

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
        let [orig] = r.split(cols([0], "", [""]));
        assert_eq!(orig.bounds(), &bounds(0, 0, 50, 50));
        #[cfg(miri)]
        orig.fill(Cell::of('!'));
    }

    #[test]
    fn slice_off_left() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let [left, rest] = r.split(cols([5, 0], "", ["", ""]));
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
        let [rest, right] = r.split(cols([0, 5], "", ["", ""]));
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
        let [left, rest] = r.split(cols([5, 0], "~", ["", ""]));
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
        let [left, rest] = r.split(cols([5, 0], "", ["~", ""]));
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
        let [left, rest] = r.split(cols([5, 0], "", ["", "~"]));
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
        let [rest, right] = r.split(cols([0, 5], "~", ["", ""]));
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
        let [rest, right] = r.split(cols([0, 5], "", ["~", ""]));
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
        let [rest, right] = r.split(cols([0, 5], "", ["", "~"]));
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
        let [left, mid, right] = r.split(cols([4, 0, 5], "", ["", "", ""]));
        assert_eq!(left.bounds(), &bounds(0, 0, 4, 50));
        assert_eq!(mid.bounds(), &bounds(4, 0, 41, 50));
        assert_eq!(right.bounds(), &bounds(45, 0, 5, 50));
    }

    #[test]
    fn slice_left_and_right_presep() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let [left, mid, right] = r.split(cols([4, 0, 5], "", ["", "", ""]));
        assert_eq!(left.bounds(), &bounds(0, 0, 4, 50));
        assert_eq!(mid.bounds(), &bounds(4, 0, 41, 50));
        assert_eq!(right.bounds(), &bounds(45, 0, 5, 50));
    }

    #[test]
    fn slice_left_and_right_sep0() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let [left, mid, right] = r.split(cols([4, 0, 5], "", ["", "", ""]));
        assert_eq!(left.bounds(), &bounds(0, 0, 4, 50));
        assert_eq!(mid.bounds(), &bounds(4, 0, 41, 50));
        assert_eq!(right.bounds(), &bounds(45, 0, 5, 50));
    }

    #[test]
    fn slice_left_and_right_sep1() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let [left, mid, right] = r.split(cols([4, 0, 5], "", ["", "", ""]));
        assert_eq!(left.bounds(), &bounds(0, 0, 4, 50));
        assert_eq!(mid.bounds(), &bounds(4, 0, 41, 50));
        assert_eq!(right.bounds(), &bounds(45, 0, 5, 50));
    }

    #[test]
    fn slice_left_and_right_sep2() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let [left, mid, right] = r.split(cols([4, 0, 5], "", ["", "", ""]));
        assert_eq!(left.bounds(), &bounds(0, 0, 4, 50));
        assert_eq!(mid.bounds(), &bounds(4, 0, 41, 50));
        assert_eq!(right.bounds(), &bounds(45, 0, 5, 50));
    }

    #[test]
    fn separator_fills_separations() {
        let mut s = Screen::new(crate::XY(50, 50));
        let r = Region::new(&mut s, None);
        let sects = r.split(cols([9, 0, 9], "!", ["@", "#", "$"]));
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
