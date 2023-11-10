use alloc::vec::Vec;
use tuig_iosys::{
    fmt::{Cell, Text},
    Action, Screen, XY,
};

use super::{
    attachments::{Attachment, Textbox, TextboxData},
    splitters::Splitter,
    Bounds, ScreenView,
};

macro_rules! split_fn {
    ( $lt:lifetime: $( $name:ident ),* $(,)? ) => { paste::paste! { $(
        #[allow(dead_code)] // might as well have all of them, even if unused
        pub(crate) fn [<split_ $name>](mut self, amt: usize) -> (Region<$lt>, Region<$lt>) {
            let chunk = self.[<split_ $name _mut>](amt);
            (chunk, self)
        }

        #[allow(dead_code)] // might as well have all of them, even if unused
        pub(crate) fn [<split_ $name _mut >](&mut self, amt: usize) -> Region<$lt> {
            let (chunk, rest) = self.bounds.[<split_ $name>](amt);
            let chunk_input = chunk.filter(&self.input);
            // SAFETY: `chunk` and `rest` are guaranteed to be non-overlapping by the `bounds.split_*` methods
            let [chunk_sv, rest_sv] = unsafe {
                core::mem::take(&mut self.sv).split([chunk, rest])
            };
            self.sv = rest_sv;
            self.input = rest.filter(&self.input);
            self.bounds = rest;
            Region { sv: chunk_sv, input: chunk_input, bounds: chunk }
        }
    )* } }
}

/// Something you can put an [`Attachment`] in.
///
/// You start with a [`Region`] that occupies the whole screen and captures one input. Then you [`split`](Self::split)
/// it to get more regions, and split those to get more, until you've got your whole layout. You can split a region
/// with anything that implements [`Splitter`], including the built-in ones in the "Implementors" section. When a
/// region gets split, some inputs -- notably mouse movements -- will only get passed to the regions that they happen
/// in.
///
/// In each region, you can put an [`Attachment`], which is what `tuig-ui` calls UI elements. Attachments serve two
/// purposes:
/// - Handle user input in a way that makes sense for that element (e.g. a button returning `true` when it's clicked)
/// - Render the element to the screen (e.g. a button rendering its text and, when you click it, a fill color)
///
/// You can split regions and add attachments in whatever order you like, so e.g. you can change how you display bits
/// depending on the state of user input, and refresh things on the same frame you get input.
pub struct Region<'s> {
    sv: ScreenView<'s>,
    pub(crate) input: Action,
    bounds: Bounds,
}

impl<'s> Region<'s> {
    /// Create a new `Region` encompassing an entire screen with a single input.
    ///
    /// This is most typically called in your wrapper when you have input to handle, to create the root element you
    /// attach everything else into.
    pub fn new(screen: &'s mut Screen, input: Action) -> Self {
        let bounds = Bounds::new(0, 0, screen.size().x(), screen.size().y());
        Self {
            sv: unsafe { ScreenView::new(screen, bounds) },
            input,
            bounds,
        }
    }

    #[cfg(test)]
    pub(crate) fn bounds(&self) -> &Bounds {
        &self.bounds
    }

    /// Get the size of this region of the screen.
    pub fn size(&self) -> XY {
        self.bounds.size
    }

    split_fn!('s: left, right, top, bottom);

    /// Split the region into one or more children.
    ///
    /// The child regions never overlap each other, and never extend beyond the bounds of the parent. If you want to
    /// overlap, wait for [#61](https://github.com/nic-hartley/tuig/issues/61) to be completed.
    ///
    /// This consumes the parent and returns the child regions. It doesn't modify anything in-place. If you don't use
    /// the children, why even bother doing the split?
    #[must_use = "child regions can't be used if you discard them -- why split?"]
    pub fn split<S: Splitter<'s>>(self, splitter: S) -> S::Output {
        splitter.split(self)
    }

    pub(crate) fn raw_pieces(self) -> (Action, ScreenView<'s>) {
        (self.input, self.sv)
    }

    /// Attach something to this region, returning whatever it wants based on the input.
    ///
    /// Remember that it's common to implement `Attachment` for `&T` or `&mut T`, especially for elements that need to
    /// store state of some sort. If you're getting weird errors about a type not implementing `Attachment` when
    /// you're 100% sure it does, check the type's docs and `impl Attachment` block more carefully.
    pub fn attach<A: Attachment<'s>>(self, attachment: A) -> A::Output {
        attachment.attach(self)
    }

    /// Fill the whole region with (copies of) a cell.
    pub fn fill(self, cell: Cell) {
        self.attach(|_, mut sv: ScreenView| sv.fill(cell))
    }

    /// Fill the whole region with some text.
    ///
    /// If this runs out of space, it will cut off the bottom of the text. If you need more control over how it's
    /// displayed, create and attach a [`Textbox`] directly.
    ///
    /// Returns a [`TextboxData`], as usual.
    pub fn text(self, text: Vec<Text>) -> TextboxData {
        self.attach(Textbox::new(text))
    }
}

impl Region<'static> {
    /// Create an empty region taking any input.
    ///
    /// This is able to have `'static` lifetime because it's not actually referring to any part of any screen. It has
    /// zero size; it doesn't need any backing memory because any writes into that will just vanish regardless.
    pub fn empty(input: Action) -> Self {
        Self {
            sv: ScreenView::empty(),
            input,
            bounds: Bounds::empty(),
        }
    }
}

impl<'s> core::fmt::Debug for Region<'s> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.bounds.fmt(f)
    }
}
