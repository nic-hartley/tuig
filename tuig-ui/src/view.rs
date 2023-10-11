use core::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr::NonNull,
};

use alloc::slice;
use tuig_iosys::{Screen, fmt::Cell, XY};

use super::Bounds;

/// A mutable view into a region of a `Screen`.
///
/// You don't directly get `ScreenView`s; they're given to you through [`Region::attach`][super::Region::attach] and
/// [`RawAttachment::raw_attach`][super::RawAttachment::raw_attach]. You can use them to directly draw to a screen's
/// textgrid, but bounded in a certain region, so that multiple attachments can be alive at once without causing
/// lifetime issues.
///
/// Primarily, you'll do that through [`Self::row`], which is also available through
pub struct ScreenView<'s> {
    /// Ties the lifetimes together
    _sc: PhantomData<&'s Screen>,
    /// Pointer to the original [`Screen`]'s `Screen.cells`; or `None` for `ScreenView::empty`.
    buf: Option<NonNull<Cell>>,
    /// Full size of the original [`Screen`]
    full_size: XY,
    /// The boundaries of this particular `ScreenView` within the screen
    bounds: Bounds,
}

impl<'s> ScreenView<'s> {
    /// A `ScreenView` not coverying any space on any screen. Roughly analogous to `&mut []`. You can have as many of
    /// these as you want, since you can't access any data through it.
    pub fn empty() -> Self {
        Self {
            _sc: PhantomData,
            buf: None,
            full_size: XY(0, 0),
            bounds: Bounds::empty(),
        }
    }

    /// Create a new `ScreenView` covering the given bounds of the given `Screen`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    ///
    /// - There are never two concurrent `ScreenView`s pointing to overlapping areas on the same `Screen`, even if
    ///   they're never used. The same rules apply as aliased `&mut`s / overlapping `&mut [T]`.
    /// - The `bounds` are actually contained within the `screen`, i.e. `bounds.pos` <= `bounds.pos + bounds.size` <=
    ///   `screen.size()`.
    ///
    /// Strictly speaking, underlying UB won't be triggered unless `Index`/`IndexMut` calls produce illegally aliased
    /// references, but it's much easier to simply treat `ScreenView`s as mutable references into a `Screen`, like a
    /// `&mut [T]` into a `Vec<T>`, than to define all the potential UB elsewhere. Also, declaring the UB here means
    /// the other functions can be safe.
    pub(crate) unsafe fn new(screen: &'s mut Screen, bounds: Bounds) -> Self {
        Self {
            _sc: PhantomData,
            // SAFETY: `Vec::as_mut_ptr` never returns null pointers, only dangling ones.
            buf: Some(unsafe { NonNull::new_unchecked(screen.cells_mut().as_mut_ptr()) }),
            full_size: screen.size(),
            bounds,
        }
    }

    /// Split one `ScreenView` into more.
    ///
    /// # Safety
    ///
    /// The bounds provided must be entirely contained within the original (current) `ScreenView`, and none of them
    /// can overlap at all. There can be gaps between them, though.
    pub(crate) unsafe fn split<const N: usize>(self, subbounds: [Bounds; N]) -> [Self; N] {
        subbounds.map(|sb| Self {
            _sc: PhantomData,
            bounds: sb,
            buf: self.buf,
            full_size: self.full_size,
        })
    }

    /// Get the pointer offset for a relative location.
    ///
    /// Returns `None` if the position is out of bounds, or the offset ready to be used in `add` directly.
    ///
    /// This method is frequently used in `self.buf.as_ptr().add()`, e.g. in [`Self::cell`]. This is safe because:
    ///
    /// - The offset is guaranteed to be within the allocated object (the screen's Vec's buffer): `bounds` logically
    ///   is contained in the total space of the screen, so the location of anything within that space will be, too
    /// - Because the offset is contained within the screen's bounds, and the screen's backing buffer -- a `Vec` --
    ///   can't get larger than `isize`, the offset must be within that range
    /// - It doesn't rely on "wrapping around", it just directly goes to the right address
    fn offset(&self, pos: XY) -> Option<usize> {
        if pos.x() >= self.bounds.size.x() || pos.y() >= self.bounds.size.y() {
            return None;
        }
        let realpos = pos + self.bounds.pos;
        Some(realpos.y() * self.full_size.x() + realpos.x())
    }

    /// Get the size of this view.
    pub fn size(&self) -> XY {
        self.bounds.size
    }

    /// Get a single cell in this view.
    ///
    /// This returns `None` if the index is out of bounds.
    pub fn cell<'v>(&'v self, pos: XY) -> Option<&'v Cell> {
        let buf = self.buf?;
        let offset = self.offset(pos)?;
        // SAFETY: See [`Self::offset`] docs.
        unsafe { Some(&*buf.as_ptr().add(offset)) }
    }

    /// Get a single cell in this view, mutably.
    ///
    /// This returns `None` if the index is out of bounds.
    pub fn cell_mut<'v>(&'v mut self, pos: XY) -> Option<&'v mut Cell> {
        let buf = self.buf?;
        let offset = self.offset(pos)?;
        // SAFETY: See [`Self::offset`] docs. Mutable references are safe because this method is `&mut self`, which
        // means Rust is preventing aliased references.
        unsafe { Some(&mut *buf.as_ptr().add(offset)) }
    }

    /// Get an entire row of cells as a slice, to read them as a unit.
    ///
    /// No equivalent is available for columns because the underlying storage is row-major. Rather than implying a
    /// false equivalence, [`Self::cell`] will allow you to directly access a single cell, and you can access every
    /// cell in a column independently.
    ///
    /// This returns `None` if the index is out of bounds.
    pub fn row<'v>(&'v self, idx: usize) -> Option<&'v [Cell]> {
        let buf = self.buf?;
        let offset = self.offset(XY(0, idx))?;
        // SAFETY: See [`Self::offset`] docs.
        let start = unsafe { buf.as_ptr().add(offset) };
        let len = self.bounds.size.x();
        // SAFETY: `bounds` from different instances are guaranteed (from `Self::new`) to be exclusive between them,
        // so there can't be any overlap that way. And the use of `&self` ensures that this object  won't be used
        // to get multiple row references simultaneously (except as Rust allows) so there's no risk of bad aliasing.
        // The other conditions are fulfilled because this pointer is entirely contained within a single Vec alloc.
        unsafe { Some(slice::from_raw_parts(start, len)) }
    }

    /// Get an entire row of cells as a mutable slice, to read or write them as a unit.
    ///
    /// No equivalent is available for columns because the underlying storage is row-major. Rather than implying a
    /// false equivalence, [`Self::cell`] will allow you to directly access a single cell, and you can access every
    /// cell in a column independently.
    ///
    /// This returns `None` if the index is out of bounds.
    pub fn row_mut<'v>(&'v mut self, idx: usize) -> Option<&'v mut [Cell]> {
        let buf = self.buf?;
        let offset = self.offset(XY(0, idx))?;
        // SAFETY: The offset is guaranteed to be within the allocated object (the screen's Vec's buffer) because
        // `self.offset` ensures it. It's guaranteed to fit within an `isize` because the Vec can't get that big. And
        // we don't rely on "wrapping around".
        let start = unsafe { buf.as_ptr().add(offset) };
        let len = self.bounds.size.x();
        // SAFETY: `bounds` from different instances are guaranteed (from `Self::new`) to be exclusive between them,
        // so there can't be any overlap that way. And the use of `&mut self` ensures that this object  won't be used
        // to get multiple row references simultaneously (except as Rust allows) so there's no risk of bad aliasing.
        // The other conditions are fulfilled because this pointer is entirely contained within a single Vec alloc.
        unsafe { Some(slice::from_raw_parts_mut(start, len)) }
    }

    /// Fill this section of the screen with a single character.
    pub fn fill(&mut self, cell: Cell) {
        for y in 0..self.size().y() {
            // SAFETY: We're iterating from 0 to the maximum row, they have to exist
            let row = unsafe { self.row_mut(y).unwrap_unchecked() };
            row.fill(cell.clone());
        }
    }
}

impl<'s> Index<usize> for ScreenView<'s> {
    type Output = [Cell];
    fn index(&self, index: usize) -> &Self::Output {
        self.row(index).expect("row index is out of bounds")
    }
}

impl<'s> IndexMut<usize> for ScreenView<'s> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.row_mut(index).expect("row index is out of bounds")
    }
}

impl<'s> Index<XY> for ScreenView<'s> {
    type Output = Cell;
    fn index(&self, index: XY) -> &Self::Output {
        self.cell(index).expect("row index is out of bounds")
    }
}

impl<'s> IndexMut<XY> for ScreenView<'s> {
    fn index_mut(&mut self, index: XY) -> &mut Self::Output {
        self.cell_mut(index).expect("row index is out of bounds")
    }
}

impl<'s> Default for ScreenView<'s> {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_cell_inbounds() {
        let mut screen = Screen::new(XY(5, 5));
        screen.cells_mut()[0] = Cell::of('~');
        let sv = unsafe { ScreenView::new(&mut screen, Bounds::new(0, 0, 5, 5)) };
        assert_eq!(sv[XY(0, 0)], Cell::of('~'));
    }

    #[test]
    fn set_cell_inbounds() {
        let mut screen = Screen::new(XY(5, 5));
        screen.cells_mut()[0] = Cell::of('~');
        let mut sv = unsafe { ScreenView::new(&mut screen, Bounds::new(0, 0, 5, 5)) };
        sv[XY(0, 0)] = Cell::of('!');
        assert_eq!(screen.cells_mut()[0], Cell::of('!'));
    }

    #[test]
    fn get_row_inbounds() {
        let mut screen = Screen::new(XY(5, 5));
        for (i, v) in screen.cells_mut()[0..5].iter_mut().enumerate() {
            *v = Cell::of(char::from_digit(i as u32, 36).unwrap());
        }
        let sv = unsafe { ScreenView::new(&mut screen, Bounds::new(0, 0, 5, 5)) };
        assert_eq!(
            sv[0],
            [
                Cell::of('0'),
                Cell::of('1'),
                Cell::of('2'),
                Cell::of('3'),
                Cell::of('4')
            ]
        );
    }

    #[test]
    fn set_row_inbounds() {
        let mut screen = Screen::new(XY(5, 5));
        let mut sv = unsafe { ScreenView::new(&mut screen, Bounds::new(0, 0, 5, 5)) };
        for (i, v) in sv[0].iter_mut().enumerate() {
            *v = Cell::of(char::from_digit(i as u32, 36).unwrap());
        }
        assert_eq!(
            &screen.cells_mut()[0..5],
            [
                Cell::of('0'),
                Cell::of('1'),
                Cell::of('2'),
                Cell::of('3'),
                Cell::of('4')
            ]
        );
    }

    #[test]
    fn get_cell_outside() {
        let mut screen = Screen::new(XY(5, 5));
        let sv = unsafe { ScreenView::new(&mut screen, Bounds::new(0, 0, 5, 5)) };
        assert_eq!(sv.cell(XY(5, 0)), None);
    }

    #[test]
    fn get_cell_mut_outside() {
        let mut screen = Screen::new(XY(5, 5));
        let mut sv = unsafe { ScreenView::new(&mut screen, Bounds::new(0, 0, 5, 5)) };
        assert_eq!(sv.cell_mut(XY(5, 0)), None);
    }

    #[test]
    fn get_row_outside() {
        let mut screen = Screen::new(XY(5, 5));
        let sv = unsafe { ScreenView::new(&mut screen, Bounds::new(0, 0, 5, 5)) };
        assert_eq!(sv.row(5), None);
    }

    #[test]
    fn get_row_mut_outside() {
        let mut screen = Screen::new(XY(5, 5));
        let mut sv = unsafe { ScreenView::new(&mut screen, Bounds::new(0, 0, 5, 5)) };
        assert_eq!(sv.row_mut(5), None);
    }
}
