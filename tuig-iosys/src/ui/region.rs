use crate::{Screen, Action, XY};

use super::Bounds;

pub struct Region<'s> {
    screen: &'s mut Screen,
    input: Option<Action>,
    bounds: Bounds,
}

impl<'s> Region<'s> {
    pub fn new(screen: &'s mut Screen, input: Option<Action>) -> Self {
        let bounds = Bounds { pos: XY(0, 0), size: screen.size() };
        Self { screen, input, bounds }
    }
}
