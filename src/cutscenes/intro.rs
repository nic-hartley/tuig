use std::{time::{Duration, Instant}, io};

use rand::{prelude::*, distributions::Standard};

use crate::{io::{output::{Screen, Cell}, sys::IoSystem}, text};

fn randf(min: f32, max: f32) -> f32 {
    thread_rng().gen_range(min..max)
}

async fn sleep(s: f32) {
    tokio::time::sleep(Duration::from_secs_f32(s)).await
}

fn rngat(seed: u64, x: usize, y: usize, xor: u64) -> SmallRng {
    let pos_seed = seed ^ xor ^ (x as u64) ^ (y as u64).rotate_left(32);
    SmallRng::seed_from_u64(pos_seed)
}

fn fadeat(seed: u64, x: usize, y: usize) -> f32 {
    rngat(seed, x, y, 0x5CA1AB1E7E1ECA57).gen()
}

fn cellat(seed: u64, x: usize, y: usize) -> Cell {
    const CHARS: [char; 92] = [
        'a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z',
        'A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z',
        '1','2','3','4','5','6','7','8','9','0',
        '!','@','#','$','%','^','&','*','(',')',
        ',','.','<','>','[',']','{','}','`','~','/','?','\\','|','\'','"',';',':','-','_',
    ];
    Cell {
        ch: *CHARS.choose(&mut rngat(seed, x, y, 0xBE1A7EDDECEA5ED)).unwrap(),
        fg: rngat(seed, x, y, 0xCA11AB1ECA55E77E).gen(),
        ..Cell::BLANK
    }
}

async fn charsplash(io: &mut dyn IoSystem) -> io::Result<()> {
    const SHIFT_LEN: f32 = 0.5;
    const NUM_STAGES: usize = 3;

    let mut screen = Screen::new(io.size());

    let mut seeds = [0u64; NUM_STAGES + 1];
    thread_rng().fill(&mut seeds);
    let seeds = seeds;
    let start = Instant::now();
    loop {
        let time = Instant::now().duration_since(start).as_secs_f32();
        let stage = (time / SHIFT_LEN) as usize;
        if stage > NUM_STAGES {
            break;
        }
        let within = (time / SHIFT_LEN).fract();

        screen.resize(io.size());
        for y in 0..screen.size().y() {
            for x in 0..screen.size().x() {
                let before_flip = within < fadeat(seeds[stage], x, y);
                let cell_val = if before_flip {
                    if stage == 0 {
                        // if we're in the first stage, we go from a blank cell
                        Cell::BLANK
                    } else {
                        // otherwise we generate the cell from the previous stage
                        cellat(seeds[stage-1], x, y)
                    }
                } else {
                    if stage == NUM_STAGES {
                        // if we're in the final stage, we go to a blank cell
                        Cell::BLANK
                    } else {
                        // otherwise we generate the cell from the current stage
                        cellat(seeds[stage], x, y)
                    }
                };
                screen[y][x] = cell_val;
            }
        }
        io.draw(&screen).await?;

        // wait until either the screen is resized, or a brief (randomized) period passes
        tokio::select! {
            _ = io.input() => {}
            _ = sleep(randf(0.03, 0.06)) => {}
        }
    }

    screen.resize(io.size());
    io.draw(&screen).await?;

    Ok(())
}

pub async fn run(io: &mut dyn IoSystem) -> io::Result<()> {
    // First we do the fun fuzzy character thing.
    // This bit is a little complex to make sure it handles resizing right.
    charsplash(io).await?;
    Ok(())
}