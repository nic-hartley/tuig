use std::mem;

use crossterm::terminal;
use tokio::sync::{mpsc, oneshot};

use super::{Action, Input};

async fn process_input(_chan: mpsc::UnboundedSender<Action>, _stop: oneshot::Receiver<()>) {
    todo!("Send keystrokes");
    // TODO: receive inputs from stdin
}

pub struct AnsiInput {
    queue: mpsc::UnboundedReceiver<Action>,
    stop: Option<oneshot::Sender<()>>,
}

impl AnsiInput {
    pub fn get() -> crossterm::Result<AnsiInput> {
        terminal::enable_raw_mode()?;
        let (queue_s, queue_r) = mpsc::unbounded_channel();
        let (stop_s, stop_r) = oneshot::channel();
        tokio::spawn(process_input(queue_s, stop_r));
        Ok(AnsiInput { queue: queue_r, stop: Some(stop_s) })
    }
}

#[async_trait::async_trait]
impl Input for AnsiInput {
    async fn next(&mut self) -> Action {
        self.queue.recv().await.expect("Queue should not die before stop signal sent")
    }

    fn flush(&mut self) {
        while let Ok(_) = self.queue.try_recv() {
            // already pulled the thing out
        }
    }
}

impl Drop for AnsiInput {
    fn drop(&mut self) {
        let stop = mem::replace(&mut self.stop, None).expect("Tried to drop an already dropped AnsiInput");
        stop.send(()).expect("Receiver should still be alive");
        while self.queue.try_recv() != Err(mpsc::error::TryRecvError::Disconnected) {
            // flushing the queue and waiting for it to disconnect in the condition itself
        }
        terminal::disable_raw_mode().expect("Could not exit raw terminal mode");
    }
}
