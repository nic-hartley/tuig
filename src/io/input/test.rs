use std::{cmp::min, thread::{self, sleep}, time::{Duration, Instant}};

use tokio::sync::{broadcast, oneshot};

use super::{Action, Input};

pub struct UntimedStream {
    contents: Vec<Action>,
}

impl UntimedStream {
    pub fn of(actions: &[Action]) -> UntimedStream {
        UntimedStream { contents: actions.to_vec() }
    }
}

impl Input for UntimedStream {
    fn listen(&mut self) -> broadcast::Receiver<Action> {
        let (send, recv) = broadcast::channel(self.contents.len());
        for act in &self.contents {
            send.send(act.clone()).expect("Failed to send into a receiver we have right there");
        }
        recv
    }
}

pub struct TimedStream {
    // always Some until it's dropped; then it's replaced with None
    kill_thread: Option<oneshot::Sender<()>>,
    // always Some until it's dropped; then it's replaced with None
    thread: Option<thread::JoinHandle<()>>,
    send: broadcast::Sender<Action>,
}

impl TimedStream {
    pub fn with_delays(timings: &[(Action, usize)]) -> TimedStream {
        let timings = timings.to_vec();
        let (send, _) = broadcast::channel(4);
        let thread_send = send.clone();
        let (kill_send, mut kill_recv) = oneshot::channel();
        let thread = thread::spawn(move || {
            let mut next_at = Instant::now();
            for (action, delay) in timings {
                while let Some(dur) = Instant::now().checked_duration_since(next_at) {
                    if kill_recv.try_recv().is_ok() {
                        return;
                    }
                    // ensure that we check at least ~once a second for the killswitch
                    sleep(min(Duration::from_secs(1), dur));
                }
                if kill_recv.try_recv().is_ok() {
                    return;
                }
                let _ = thread_send.send(action);
                next_at += Duration::from_millis(delay as u64);
            }
        });
        TimedStream { kill_thread: Some(kill_send), thread: Some(thread), send }
    }
}

impl Drop for TimedStream {
    fn drop(&mut self) {
        let kill = std::mem::replace(&mut self.kill_thread, None).expect("Tried to destruct a destructed value");
        let _ = kill.send(()); // don't care if this fails: that just means we already exhausted the input
        let thread = std::mem::replace(&mut self.thread, None).expect("Tried to destruct a destructed value");
        thread.join().expect("Failed to join input thread");
    }
}

impl Input for TimedStream {
    fn listen(&mut self) -> broadcast::Receiver<Action> {
        self.send.subscribe()
    }
}
