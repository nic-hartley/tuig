// ScrollableAttachment, ScrollState, ScrollState

use core::ops::{Index, IndexMut};

use crate::{fmt::Cell, Action, Key, MouseButton, XY};

use super::{Bounds, Region};

/// An equivalent to [`Attachment`](super::Attachment), but it can be scrolled.
/// 
/// The actual scrolling behavior is determined by the attachment, but broadly speaking, attachments implementing this
/// will probably be scrolled by the scroll wheel, arrow keys, etc.
pub trait ScrollableAttachment<'s, 'st> {
    type Output;
    fn scroll_attach(self, region: ScrolledRegion<'s, 'st>) -> Self::Output;
}

/// Helper for rendering scrollable attachments.
///
/// This abstracts a scrollable region as a "virtual region", which is "cropped" to just the real region after your
/// code is done. The virtual region starts at `usize::MAX` by `usize::MAX` cells, but you'll want  to narrow it down
/// with [`Self::size`] to something that fits your use-case. For example:
///
///  -  A vertical-scrolling textbox should narrow the virtual region's width to match the real width and its height
///     to the total number of lines, after line wrapping.
///  -  A finite map scrolling in both directions should narrow the virtual region's size to match the total map size.
///
/// Once that's set up, you can scroll the view in any direction, and this struct will handle updating the caller's
/// stored `ScrollState`, ensuring it's bounded within that maximum size you've set, as well as letting you read and
/// write using absolute coordinates with the indexing. It works like [`ScreenView`](super::ScreenView), except that
/// reads and writes outside the virtual area are blank cells and no-ops, respectively.
///
/// This abstraction isn't suitable if you're doing another kind of scrolling, e.g. an infinite map, or wanting to
/// allow scrolling well past the edges of the virtual area. You'll have to build your own. Consider submitting a PR
/// if you like how it turns out!
///
/// # Indexing
///
/// When indexing, be slightly careful. I would very much like it if this worked:
///
/// ```rs,no_run
/// /* 1. */ let (x, y) = some_out_of_bounds_position();
/// /* 2. */ scrolled_region[y][x] = some_cell();
/// /* 3. */ assert!(scrolled_region[y][x], Cell::BLANK);
/// ```
///
/// It'll compile, but because of how the underlying traits are defined, you'll get a panic on line 2 and 3 for
/// indexing out of range. Instead:
///
///  -  Index into individual cells by `XY`: `scrolled_region[XY(x, y)]`
///  -  Be cautious of the length of returned slices: `scrolled_region[y]` will be `&[]` if out-of-bounds, so `.fill`
///     will work, but `[x]` might not.
///  -  Check bounds yourself prior to writing: [`scrolled_region.shows(pos)`](Self::shows), iterating over
///     [`scrolled_region.bounds()`](Self::bounds), etc.
///  -  Don't do `*scrolled_region.index_mut(XY(x, y))`. It won't give you what you expect and it might tank your
///     performance in seemingly unrelated code. (Just use `index`.)
pub struct ScrolledRegion<'s, 'st> {
    region: Region<'s>,
    scroll: &'st mut XY,
    virt_size: XY,
    /// Target for writes that need to be discarded.
    _dummy: Cell,
}

impl<'s, 'st> ScrolledRegion<'s, 'st> {
    /// Create a scrollable region from a real region to render onto and a scroll state to update.
    pub(crate) fn new(region: Region<'s>, scroll: &'st mut XY) -> Self {
        Self {
            region,
            scroll,
            virt_size: XY(usize::MAX, usize::MAX),
            _dummy: Cell::BLANK,
        }
    }

    /// Get the boundaries of the real region, relative to the top left of the virtual region.
    ///
    /// This is meant to be used to optimize, so you don't try rendering things outside of the viewable area. You can
    /// in theory use it for positioning things on the real screen as well, but usually it's a better idea to do that
    /// with the raw region (through [`Self::raw`], or in the parent).
    pub fn bounds(&self) -> Bounds {
        Bounds {
            pos: *self.scroll,
            size: self.region.size(),
        }
    }

    /// Check whether a position in the virtual region is visible in the real region.
    ///
    /// In other words, will reading from or writing to this position do it for real?
    pub fn shows(&self, pos: &XY) -> bool {
        self.bounds().contains(pos)
    }

    /// The maximum scroll state position so the real region won't exit the virtual region.
    fn max_pos(&self) -> XY {
        XY(
            self.virt_size.x().saturating_sub(self.region.size().x()),
            self.virt_size.y().saturating_sub(self.region.size().y()),
        )
    }

    /// Set the size of the full virtual area.
    ///
    /// This will adjust the scroll position so none of the real region is outside of the virtual region, unless the
    /// virtual region is smaller than the real region, in which case the virtual region will be at the top left.
    pub fn size(&mut self, size: XY) {
        self.virt_size = size;
        let max = self.max_pos();
        self.scroll.0 = self.scroll.0.min(max.0);
        self.scroll.1 = self.scroll.1.min(max.1);
    }

    /// Move the scrollable area left by `amount` characters, but not going past the left edge.
    pub fn scroll_left(&mut self, amt: usize) {
        self.scroll.0 = self.scroll.0.saturating_sub(amt)
    }

    /// Move the scrollable area right by `amount` characters, but not going past the right edge.
    pub fn scroll_right(&mut self, amt: usize) {
        let max = self.max_pos();
        if amt > max.0 - self.scroll.0 {
            self.scroll.0 = max.0;
        } else {
            self.scroll.0 += amt;
        }
    }

    /// Move the scrollable area up by `amount` characters, but not going past the top.
    pub fn scroll_up(&mut self, amt: usize) {
        self.scroll.1 = self.scroll.1.saturating_sub(amt)
    }

    /// Move the scrollable area down by `amount` characters, but not going past the bottom.
    pub fn scroll_down(&mut self, amt: usize) {
        let max = self.max_pos();
        if amt > max.1 - self.scroll.1 {
            self.scroll.1 = max.1;
        } else {
            self.scroll.1 += amt;
        }
    }

    /// Handle the default scroll-wheel controls, adjusting the viewing region appropriately for that input.
    ///
    /// Returns whether the viewed area actually got moved, e.g. to let you skip any further input processing. Just be
    /// sure you don't accidentally skip rendering!
    pub fn scroll_with_wheel(&mut self, input: &Action) -> bool {
        match input {
            Action::MousePress {
                button: MouseButton::ScrollUp,
                ..
            } => self.scroll_up(3),
            Action::MousePress {
                button: MouseButton::ScrollDown,
                ..
            } => self.scroll_down(3),
            _ => return false,
        }
        true
    }

    /// Handle the default arrow key controls, adjusting the viewing region appropriately for that input.
    ///
    /// Returns whether the viewed area actually got moved, e.g. to let you skip any further input processing. Just be
    /// sure you don't accidentally skip rendering!
    pub fn scroll_with_arrows(&mut self, input: &Action) -> bool {
        // TODO: Handle shift+arrow, after #58
        match input {
            Action::KeyPress { key: Key::Left } => self.scroll_left(1),
            Action::KeyPress { key: Key::Right } => self.scroll_right(1),
            Action::KeyPress { key: Key::Up } => self.scroll_up(1),
            Action::KeyPress { key: Key::Down } => self.scroll_down(1),
            _ => return false,
        }
        true
    }

    /// Handle all of the default scrolling controls at once, as though you called each indvidual method.
    ///
    /// Returns whether the viewed area actually got moved, e.g. to let you skip any further input processing. Just be
    /// sure you don't accidentally skip rendering!
    pub fn scroll(&mut self, input: &Action) -> bool {
        self.scroll_with_wheel(input) || self.scroll_with_arrows(input)
    }
}

impl<'s, 'st> Index<usize> for ScrolledRegion<'s, 'st> {
    type Output = [Cell];

    fn index(&self, index: usize) -> &Self::Output {
        if self.bounds().ys().contains(&index) {
            &self.region.sv[index - self.scroll.y()]
        } else {
            &[]
        }
    }
}

impl<'s, 'st> IndexMut<usize> for ScrolledRegion<'s, 'st> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if self.bounds().ys().contains(&index) {
            &mut self.region.sv[index - self.scroll.y()]
        } else {
            &mut []
        }
    }
}

impl<'s, 'st> Index<XY> for ScrolledRegion<'s, 'st> {
    type Output = Cell;

    fn index(&self, index: XY) -> &Self::Output {
        if self.shows(&index) {
            &self.region.sv[index - *self.scroll]
        } else {
            &Cell::BLANK
        }
    }
}

impl<'s, 'st> IndexMut<XY> for ScrolledRegion<'s, 'st> {
    fn index_mut(&mut self, index: XY) -> &mut Self::Output {
        if self.shows(&index) {
            &mut self.region.sv[index - *self.scroll]
        } else {
            &mut self._dummy
        }
    }
}
