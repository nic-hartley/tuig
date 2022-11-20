use std::{time::{Duration, Instant}, io};

use rand::prelude::*;

use crate::io::{output::{Screen, Cell}, sys::IoSystem, clifmt::{Color, FormattedExt}};

async fn sleep(s: f32) {
    tokio::time::sleep(Duration::from_secs_f32(s)).await
}

fn rngat(seed: u64, x: usize, y: usize, xor: u64) -> SmallRng {
    let pos_seed = seed ^ xor ^ (x as u64) ^ (y as u64).rotate_left(32);
    // SmallRng is the right choice here because for this use we only care about the appearance of randomness, not
    // true unpredictability, and it's massively faster to initialize and generate than the higher-quality RNGs.
    // Further, we're not trying to generate the same values across different platforms, or even across different
    // runs on the same platform.
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
    let ch = *CHARS.choose(&mut rngat(seed, x, y, 0xBE1A7EDDECEA5ED)).unwrap();
    let fg = rngat(seed, x, y, 0xCA11AB1ECA55E77E).gen();
    Cell::of(ch).fg(fg)
}

fn leaveat(seed: u64, x: usize, y: usize) -> bool {
    rngat(seed, x, y, 0xBA11AD0FBADA55E5).gen_bool(0.01)
}

/// Play the wave, then return the seed of the last shift (which produces the leftovers)
pub async fn sprinkler_wave(io: &mut dyn IoSystem, screen: &mut Screen) -> io::Result<u64> {
    // the unit of width here is fractions of the screen width; time is in seconds

    // the width of one shift
    const SHIFT_WIDTH: f32 = 0.1;
    // how many shifts there are in the wave
    const SHIFT_COUNT: usize = 2;
    // how fast the wave moves right
    const WAVE_SPEED: f32 = 0.5;
    // the width of the whole wave
    const WAVE_WIDTH: f32 = SHIFT_WIDTH * SHIFT_COUNT as f32;

    let seeds: [u64; SHIFT_COUNT] = thread_rng().gen();
    let start = Instant::now();

    loop {
        let now = Instant::now();
        let since_start = now.duration_since(start).as_secs_f32();
        let wave_lead = since_start * WAVE_SPEED;
        let wave_trail = wave_lead - WAVE_WIDTH;
        if wave_trail > 1.0 {
            break;
        }

        screen.resize(io.size());
        for x in 0..screen.size().x() {
            let pct = x as f32 / screen.size().x() as f32;
            if pct > wave_lead {
                // everything past the leading edge is blank
                for y in 0..screen.size().y() {
                    screen[y][x] = Cell::BLANK;
                }
            } else if pct < wave_trail {
                // almost everything past the trailing edge is blank
                for y in 0..screen.size().y() {
                    if leaveat(seeds[SHIFT_COUNT-1], x, y) {
                        screen[y][x] = cellat(seeds[SHIFT_COUNT-1], x, y);
                    } else {
                        // leave it blank
                    }
                }
            } else {
                let shift_pos = (wave_lead - pct) / SHIFT_WIDTH;
                let from_shift = shift_pos as usize;
                let to_shift = from_shift + 1;
                let within = shift_pos.fract();
    
                for y in 0..screen.size().y() {
                    if within < fadeat(seeds[from_shift], x, y) {
                        if from_shift > 0 {
                            screen[y][x] = cellat(seeds[from_shift], x, y);
                        }
                    } else {
                        if to_shift < SHIFT_COUNT {
                            screen[y][x] = cellat(seeds[to_shift], x, y);
                        } else if leaveat(seeds[from_shift], x, y) {
                            screen[y][x] = cellat(seeds[from_shift], x, y);
                        }
                    }
                }
            }
        }
        io.draw(&screen).await?;

        // wait until either the screen is resized, or a brief (randomized) period passes
        tokio::select! {
            _ = io.input() => {}
            _ = sleep(0.01) => {}
        }
    }

    Ok(seeds[SHIFT_COUNT-1])
}

pub async fn cleanup_wave(io: &mut dyn IoSystem, screen: &mut Screen, seed: u64) -> io::Result<()> {
    // render the sparkles, accounting for resizes, until the timer is done
    let timer = sleep(1.0);
    tokio::pin!(timer);
    loop {
        screen.resize(io.size());
        for y in 0..screen.size().y() {
            for x in 0..screen.size().x() {
                if leaveat(seed, x, y) {
                    screen[y][x] = cellat(seed, x, y);
                }
            }
        }
        io.draw(&screen).await?;

        tokio::select! {
            _ = &mut timer => break,
            _ = io.input() => {}
        }
    }

    // draw the wipe
    const WAVE_SPEED: f32 = 1.0;
    const WAVE_WIDTH: f32 = 0.025;
    const SMEAR_WIDTH: f32 = 0.025;
    // from darkest to lightest
    const SMEAR_BGS: [Color; 6] = [
        Color::Black, Color::Blue, Color::Red, Color::Cyan, Color::Yellow, Color::White,
    ];
    let start = Instant::now();
    loop {
        let now = Instant::now();
        let since_start = now.duration_since(start).as_secs_f32();
        let smear_lead = since_start * WAVE_SPEED;
        let wave_lead = smear_lead - SMEAR_WIDTH;
        let wave_trail = wave_lead - WAVE_WIDTH;
        let smear_trail = wave_trail - SMEAR_WIDTH;
        if smear_trail > 1.0 {
            break;
        }

        screen.resize(io.size());
        for x in 0..screen.size().x() {
            let pct = 1.0 - (x as f32 / screen.size().x() as f32);

            for y in 0..screen.size().y() {
                let mut cell;
                if pct > smear_lead {
                    // ahead of the leading edge: just render the cell normally
                    cell = if leaveat(seed, x, y) { cellat(seed, x, y) } else { Cell::BLANK };
                } else if pct > wave_lead {
                    // start smearing: lighten background color accordingly
                    let smear_amt = (pct - wave_lead) / SMEAR_WIDTH;
                    let smear_idx = SMEAR_BGS.len() - 1 - (smear_amt * SMEAR_BGS.len() as f32) as usize;
                    let smear_bg = SMEAR_BGS[smear_idx];
                    cell = if leaveat(seed, x, y) { cellat(seed, x, y) } else { Cell::BLANK };
                    cell = cell.bg(smear_bg);
                } else if pct > wave_trail {
                    // just a blank white line
                    cell = Cell::BLANK;
                    cell = cell.bg(Color::BrightWhite);
                } else if pct > smear_trail {
                    // smear backwards
                    let smear_amt = (pct - smear_trail) / SMEAR_WIDTH;
                    let smear_idx = (smear_amt * SMEAR_BGS.len() as f32) as usize;
                    let smear_bg = SMEAR_BGS[smear_idx];
                    cell = Cell::BLANK;
                    cell = cell.bg(smear_bg);
                } else {
                    cell = Cell::BLANK;
                }
                screen[y][x] = cell;
            }
        }
        io.draw(&screen).await?;

        tokio::select! {
            _ = io.input() => {}
            _ = sleep(0.01) => {}
        }
    }
    screen.resize(io.size());
    io.draw(&screen).await?;

    Ok(())
}

pub async fn run(io: &mut dyn IoSystem) -> io::Result<()> {
    let mut screen = Screen::new(io.size());

    let seed = sprinkler_wave(io, &mut screen).await?;
    cleanup_wave(io, &mut screen, seed).await?;

    

    Ok(())
}