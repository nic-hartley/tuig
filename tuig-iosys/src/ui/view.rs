use core::{marker::PhantomData, ptr::NonNull, ops::{Index, IndexMut}};

use alloc::slice;

use crate::{Screen, fmt::Cell, XY};

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
    /// Pointer to the original [`Screen`]'s `Screen.cells`
    buf: NonNull<Cell>,
    /// Full size of the original [`Screen`]
    size: XY,
    /// The boundaries of this particular `ScreenView` within the screen
    bounds: Bounds,
}

impl<'s> ScreenView<'s> {
    /// Create a new `ScreenView` covering the given bounds of the given `Screen`.
    /// 
    /// # Safety
    /// 
    /// The caller must ensure that there are never two `ScreenView`s with overlapping bounds on the same screen at a
    /// time, even if they're never used. The rules are similar to those for `&mut [T]` and other aliased/overlapping
    /// mutable references (i.e. don't).
    /// 
    /// Strictly speaking, underlying UB won't be triggered unless `Index`/`IndexMut` calls produce illegally alised
    /// references, but it's much easier to simply treat `ScreenView`s as mutable references into a `Screen`.
    pub(crate) unsafe fn new(screen: &'s mut Screen, bounds: Bounds) -> Self {
        Self {
            _sc: PhantomData,
            // SAFETY: `Vec::as_mut_ptr` never returns null pointers, only dangling ones.
            buf: unsafe { NonNull::new_unchecked(screen.cells.as_mut_ptr()) },
            size: screen.size(),
            bounds,
        }
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
        if pos.x() > self.bounds.size.x() || pos.y() > self.bounds.size.y() {
            return None;
        }
        let realpos = pos + self.bounds.pos;
        Some(realpos.y() * self.size.x() + realpos.x())
    }

    /// Get a single cell in this view.
    /// 
    /// This returns `None` if the index is out of bounds.
    pub fn cell<'v>(&'v self, idx: usize) -> Option<&'v Cell> {
        let offset = self.offset(XY(0, idx))?;
        // SAFETY: See [`Self::offset`] docs.
        unsafe { Some(&*self.buf.as_ptr().add(offset)) }
    }

    /// Get a single cell in this view, mutably.
    /// 
    /// This returns `None` if the index is out of bounds.
    pub fn cell_mut<'v>(&'v mut self, idx: usize) -> Option<&'v mut Cell> {
        let offset = self.offset(XY(0, idx))?;
        // SAFETY: See [`Self::offset`] docs. Mutable references are safe because this method is `&mut self`, which
        // means Rust is preventing aliased references.
        unsafe { Some(&mut *self.buf.as_ptr().add(offset)) }
    }

    /// Get an entire row of cells as a slice, to read them as a unit.
    /// 
    /// No equivalent is available for columns because the underlying storage is row-major. Rather than implying a
    /// false equivalence, [`Self::cell`] will allow you to directly access a single cell, and you can access every
    /// cell in a column independently.
    /// 
    /// This returns `None` if the index is out of bounds.
    pub fn row<'v>(&'v self, idx: usize) -> Option<&'v [Cell]> {
        let offset = self.offset(XY(0, idx))?;
        // SAFETY: See [`Self::offset`] docs.
        let start = unsafe { self.buf.as_ptr().add(offset) };
        let len = self.bounds.size.x();
        // SAFETY: `bounds` from different instances are guaranteed (from `Self::new`) to be exclusive between them,
        // so there can't be any overlap that way. And the use of `&self` ensures that this object  won't be used
        // to get multiple row references simultaneously (except as Rust allows) so there's no risk of bad aliasing.
        // The other conditions are fulfilled because this pointer is entirely contained within a single Vec alloc.
        unsafe {
            Some(slice::from_raw_parts(start, len))
        }
    }

    /// Get an entire row of cells as a mutable slice, to read or write them as a unit.
    /// 
    /// No equivalent is available for columns because the underlying storage is row-major. Rather than implying a
    /// false equivalence, [`Self::cell`] will allow you to directly access a single cell, and you can access every
    /// cell in a column independently.
    /// 
    /// This returns `None` if the index is out of bounds.
    pub fn row_mut<'v>(&'v mut self, idx: usize) -> Option<&'v mut [Cell]> {
        let offset = self.offset(XY(0, idx))?;
        // SAFETY: The offset is guaranteed to be within the allocated object (the screen's Vec's buffer) because
        // `self.offset` ensures it. It's guaranteed to fit within an `isize` because the Vec can't get that big. And
        // we don't rely on "wrapping around".
        let start = unsafe { self.buf.as_ptr().add(offset) };
        let len = self.bounds.size.x();
        // SAFETY: `bounds` from different instances are guaranteed (from `Self::new`) to be exclusive between them,
        // so there can't be any overlap that way. And the use of `&mut self` ensures that this object  won't be used
        // to get multiple row references simultaneously (except as Rust allows) so there's no risk of bad aliasing.
        // The other conditions are fulfilled because this pointer is entirely contained within a single Vec alloc.
        unsafe {
            Some(slice::from_raw_parts_mut(start, len))
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
