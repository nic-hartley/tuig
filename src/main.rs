//! Redshell is divided into three parts, architecturally:
//!
//! - Apps, one for each of the tabs in the user's UI, which handle the actual
//!   user interaction. They render the display and handle the input. They do
//!   NOT handle any actual gameplay code, just displaying the game state for
//!   the user to interact with and sending the input back to the relevant
//!   systems.
//! - Systems, which run the actual systems of the game. Note that each system
//!   will usually be implemented as a type, then constructed once for every
//!   relevant piece -- e.g. there's a `struct Relations`, instanced once per
//!   group the player encounters, which tracks their relationships with that
//!   group. There most likely isn't a 1-to-1 mapping between systems and
//!   apps, though each app is associated most directly with only a small
//!   handful of system types.
//! - Events, which are how systems can communicate with each other, the UI,
//!   and the engine. They can be immediate (processed in the same frame as
//!   they were created) or delayed (processed at an indeterminate time in the
//!   future), as appropriate for the event.
//!
//! Note that, for ease of serialization, all three are implemented as enums
//! rather than trait objects. There are traits, but they're not mandatory at
//! compile time; they're just offered as an easy way to know what changes
//! need to be made across everything.

mod apps;
mod events;
mod gameplay;
mod systems;

use std::{io::{Write, stdin, stdout}, time::{Instant, SystemTime}};

use rayon::iter::{IntoParallelIterator, IntoParallelRefMutIterator, ParallelExtend, ParallelIterator};

use crate::{apps::App, events::{Event, Events}, gameplay::chat::{ChatApp, ChatMessage, ChatSystem}, systems::System};

fn main() {
    let mut systems = vec![
        systems::Systems::ChatSystem(ChatSystem::with("admin".into())),
    ];

    let mut apps = vec![
        apps::Apps::ChatApp(ChatApp::new())
    ];

    let mut events = vec![events::Events::ChatMessage(ChatMessage {
        msg: "message".into(),
        from: "admin".into(),
        to: "player".into(),
        // TODO: Proper time system
        time: SystemTime::now(),
    })];

    // TODO: Proper ticking (repeatable timer in tokio?)
    loop {
        println!("Events:");
        for ev in &events {
            println!("- {:?} (complete: {})", ev, ev.complete());
        }

        // TODO: Correctly handle immediately-complete events
        let start = Instant::now();
        let event_count = events.len();
        let (relevant, mut next_events) = events.into_par_iter()
            .partition::<Vec<_>, Vec<_>, _>(|e| e.complete());
        let relevant_count = relevant.len();
        let (engine, general) = relevant.into_par_iter()
            .partition::<Vec<_>, Vec<_>, _>(|e| e.is_system());
        for ev in engine {
            match ev {
                Events::AddSystem(sys) => systems.push(sys),
                _ => unreachable!("Unhandled system event"),
            }
        }
        next_events.par_extend(
            systems.par_iter_mut()
                .flat_map_iter(|s| {
                    general.iter().flat_map(move |ev| {
                        s.recv(ev).into_iter()
                    })
                })
        );
        let events_time = Instant::now().duration_since(start);

        let start = Instant::now();
        apps.par_iter_mut().for_each(|app| {
            for ev in &general {
                app.recv(ev);
            }
        });
        let apps_time = Instant::now().duration_since(start);

        let mut render_to = String::new();
        let start = Instant::now();
        apps[0].render(&mut render_to);
        let render_time = Instant::now().duration_since(start);

        println!("{}", render_to);
        print!("> ");
        stdout().flush().unwrap();
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        input = input.trim_end().into();
        let start = Instant::now();
        let evs = apps[0].input(input);
        next_events.extend(evs.into_iter());
        let input_time = Instant::now().duration_since(start);

        events = next_events;

        println!("--- Timing Statistics ---");
        println!("Processed {}/{} relevant events in {}us", relevant_count, event_count, events_time.as_micros());
        println!("Sent events to {} apps in {}us", apps.len(), apps_time.as_micros());
        println!("Rendered output in {}us", render_time.as_micros());
        println!("Processed input in {}us", input_time.as_micros());
        println!();
    }
}
