use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Barrier,
    },
    time::Duration,
};

use crate::io::{input::Action, output::Screen, XY};

use super::{IoRunner, IoSystem};

pub struct NopSystem(NopIoRunner);

impl NopSystem {
    pub fn new() -> io::Result<(Self, impl IoRunner)> {
        let run = NopIoRunner::new();
        Ok((Self(run.clone()), run))
    }
}

impl IoSystem for NopSystem {
    fn draw(&mut self, _screen: &Screen) -> io::Result<()> {
        Ok(())
    }
    fn input(&mut self) -> io::Result<Action> {
        loop {
            std::thread::sleep(Duration::MAX);
        }
    }
    fn poll_input(&mut self) -> io::Result<Option<Action>> {
        Ok(None)
    }
    fn input_until(&mut self, time: Duration) -> io::Result<Option<Action>> {
        std::thread::sleep(time);
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
pub struct NopIoRunner(Arc<AtomicBool>, Arc<Barrier>);

impl NopIoRunner {
    /// Create a [`NopIoRunner`].
    pub fn new() -> Self {
        Self(Arc::new(false.into()), Arc::new(Barrier::new(2)))
    }

    /// Tell the [`NopIoRunner`] to stop.
    pub fn stop(&mut self) {
        self.0.store(true, Ordering::Release);
        self.1.wait();
    }
}

impl IoRunner for NopIoRunner {
    fn step(&mut self) -> bool {
        if self.0.load(Ordering::Acquire) {
            // ensure that `stop` returns
            self.1.wait();
            true
        } else {
            false
        }
    }

    fn run(&mut self) {
        // we don't need to constantly check
        while !self.0.load(Ordering::Acquire) {
            self.1.wait();
        }
    }
}
