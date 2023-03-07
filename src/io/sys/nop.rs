use std::{
    io,
    sync::{
        Arc, Condvar, Mutex,
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
pub struct NopIoRunner(Arc<(Mutex<bool>, Condvar)>);

impl NopIoRunner {
    /// Create a [`NopIoRunner`].
    pub fn new() -> Self {
        Self(Arc::new((Mutex::new(false), Condvar::new())))
    }

    /// Tell the [`NopIoRunner`] to stop.
    pub fn stop(&mut self) {
        *self.0.0.lock().unwrap() = true;
        self.0.1.notify_all()
    }
}

impl IoRunner for NopIoRunner {
    fn step(&mut self) -> bool {
        *self.0.0.lock().unwrap()
    }

    fn run(&mut self) {
        let _unused = self.0.1.wait_while(self.0.0.lock().unwrap(), |b| !*b).unwrap();
    }
}
