#![cfg(feature = "nop")]

#[cfg(not(feature = "std"))]
compile_error!("enable std to use nop");

use std::{
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

use crate::{action::Action, screen::Screen, xy::XY, IoRunner, IoSystem};

pub struct NopSystem(NopRunner);

impl NopSystem {
    pub fn new() -> crate::Result<(Self, impl IoRunner)> {
        let run = NopRunner::new();
        Ok((Self(run.clone()), run))
    }
}

impl IoSystem for NopSystem {
    fn draw(&mut self, _screen: &Screen) -> crate::Result<()> {
        Ok(())
    }
    fn input(&mut self) -> crate::Result<Action> {
        loop {
            std::thread::sleep(Duration::MAX);
        }
    }
    fn poll_input(&mut self) -> crate::Result<Option<Action>> {
        Ok(None)
    }
    fn size(&self) -> XY {
        XY(80, 24)
    }
    fn stop(&mut self) {
        self.0.stop()
    }
}

/// An implementation of [`IoRunner`] that doesn't actually do anything except wait for `.stop` to be called. Used by
/// [`NopSystem`], for benchmarking or testing.
#[derive(Clone)]
pub struct NopRunner(Arc<(Mutex<bool>, Condvar)>);

impl NopRunner {
    /// Create a [`NopIoRunner`].
    pub fn new() -> Self {
        Self(Arc::new((Mutex::new(false), Condvar::new())))
    }

    /// Tell the [`NopIoRunner`] to stop.
    pub fn stop(&mut self) {
        *self.0 .0.lock().unwrap() = true;
        self.0 .1.notify_all()
    }
}

impl IoRunner for NopRunner {
    fn step(&mut self) -> bool {
        *self.0 .0.lock().unwrap()
    }

    fn run(&mut self) {
        let _unused = self
            .0
             .1
            .wait_while(self.0 .0.lock().unwrap(), |b| !*b)
            .unwrap();
    }
}
