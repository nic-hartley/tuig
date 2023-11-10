//! A very simple "`Game`" which just spawns a bajillion messages and agents. Used for sorta-benchmarking.

use std::time::Instant;

use tuig::{Agent, ControlFlow, Game, Replies, Runner};
use tuig_ui::Region;

const AGENTS: u64 = 10_000;

type TinyMessage = u64;

struct TinyAgent {
    factor: u64,
}

impl Agent<TinyMessage> for TinyAgent {
    fn start(&mut self, replies: &mut Replies<TinyMessage>) -> ControlFlow {
        replies.queue(self.factor);
        ControlFlow::Continue
    }

    fn react(&mut self, msg: &TinyMessage, replies: &mut Replies<TinyMessage>) -> ControlFlow {
        if *msg <= 1 {
            // ignore it: collatz ended
        } else if *msg % AGENTS == self.factor % AGENTS {
            let next = if *msg % 2 == 0 {
                *msg / 2
            } else {
                *msg * 3 + 1
            };
            replies.queue(next);
        }
        ControlFlow::Continue
    }
}

#[derive(Default)]
struct TinyGame {
    count: u64,
    max: TinyMessage,
    complete: u64,
}

impl Game for TinyGame {
    type Message = TinyMessage;
    fn message(&mut self, msg: &Self::Message) {
        if msg != &0 {
            self.count += 1;
        }
        if *msg == 1 {
            self.complete += 1;
        } else if *msg > self.max {
            self.max = *msg;
        }
    }

    fn attach<'s>(&mut self, _into: Region<'s>, _replies: &mut Replies<Self::Message>) -> bool {
        println!(
            "count={}, max={}, complete={}",
            self.count, self.max, self.complete
        );
        self.complete == AGENTS
    }
}

fn main() {
    let mut starter = Runner::new(TinyGame::default()).input_tick(0.0);
    for factor in 1..=AGENTS {
        starter = starter.spawn(TinyAgent { factor });
    }
    let start = Instant::now();
    let TinyGame {
        count,
        max,
        complete,
    } = starter.load_run();
    let dur = Instant::now() - start;
    println!("Completed in {:.02}s", dur.as_secs_f32());
    println!(
        "Final state: count={}, max={}, complete={}",
        count, max, complete
    );
    // Ensure we get the right answers
    assert_eq!(count, 859666);
    assert_eq!(max, 27114424);
    assert_eq!(complete, 10000);
}
