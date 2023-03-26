//! Implements CPU-based rendering for GUIs.
//!
//! This should be quite widely available and compatible.

#![cfg(feature = "gui_softbuffer")]

#[cfg(not(feature = "std"))]
compile_error!("enable std to use cli_crossterm");

use std::io;

use fontdue::{Font, FontSettings};
use rayon::prelude::*;
use winit::window::Window;

use crate::{
    fmt::{Color, Formatted},
    screen::Screen,
    xy::XY,
};

use super::GuiBackend;

fn ioe4fe(e: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}

fn color_f32(c: Color) -> (f32, f32, f32) {
    let (h, s, v): (f32, f32, f32) = match c {
        Color::Black => (000.0, 0.0, 0.05),
        Color::Red => (000.0, 1.0, 0.75),
        Color::Green => (120.0, 1.0, 0.75),
        Color::Yellow => (060.0, 1.0, 0.75),
        Color::Blue => (240.0, 0.7, 0.75),
        Color::Magenta => (300.0, 1.0, 0.75),
        Color::Cyan => (180.0, 1.0, 0.75),
        Color::White => (000.0, 0.0, 0.75),
        Color::BrightBlack => (000.0, 0.0, 0.5),
        Color::BrightRed => (000.0, 1.0, 1.0),
        Color::BrightGreen => (120.0, 1.0, 1.0),
        Color::BrightYellow => (060.0, 1.0, 1.0),
        Color::BrightBlue => (240.0, 1.0, 1.0),
        Color::BrightMagenta => (300.0, 1.0, 1.0),
        Color::BrightCyan => (180.0, 1.0, 1.0),
        Color::BrightWhite => (000.0, 0.0, 1.0),
    };

    // make sure we didn't fuck this up
    assert!(0.0 <= h && h <= 360.0);
    assert!(0.0 <= s && s <= 1.0);
    assert!(0.0 <= v && v <= 1.0);

    // taken from https://en.wikipedia.org/wiki/HSL_and_HSV#HSV_to_RGB
    let c = s * v;
    let h_ = h / 60.0;
    let x = c * (1.0 - (h_ % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match h_ as usize {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        5 => (c, 0.0, x),
        _ => unreachable!(),
    };
    let m = v - c;
    let r = r1 + m;
    let g = g1 + m;
    let b = b1 + m;

    (r, g, b)
}

fn color_u32(c: Color) -> u32 {
    let (r_f, g_f, b_f) = color_f32(c);
    let r_b = (r_f * 255.0).round() as u32;
    let g_b = (g_f * 255.0).round() as u32;
    let b_b = (b_f * 255.0).round() as u32;
    (r_b << 16) | (g_b << 8) | (b_b << 0)
}

fn lerp(from: f32, to: f32, amt: f32) -> f32 {
    let big = from.max(to);
    let lil = from.min(to);
    let amt = if lil == from { amt } else { 1.0 - amt };
    lil + ((big - lil) * amt)
}

fn color_of(fg: Color, bg: Color, opacity: f32) -> u32 {
    let (fg_r, fg_g, fg_b) = color_f32(fg);
    let (bg_r, bg_g, bg_b) = color_f32(bg);
    let r_f = lerp(bg_r, fg_r, opacity);
    let g_f = lerp(bg_g, fg_g, opacity);
    let b_f = lerp(bg_b, fg_b, opacity);
    let r_b = (r_f * 255.0).round() as u32;
    let g_b = (g_f * 255.0).round() as u32;
    let b_b = (b_f * 255.0).round() as u32;
    (r_b << 16) | (g_b << 8) | (b_b << 0)
}

pub struct SoftbufferBackend {
    /// the font size, in whatever units fontdue likes
    scale: f32,
    /// the unbolded font
    regular: Font,
    /// the bolded font (all the metrics are based on unbolded)
    bold: Font,
    /// the total size of one character in the font
    ch_sz: XY,
    /// how many pixels down from the top the character baseline is
    line_baseline: usize,
    /// how thick the underline should be, in fractions of a pixel
    underline_top: usize,
}

impl GuiBackend for SoftbufferBackend {
    fn new(scale: f32) -> io::Result<Self> {
        let regular = Font::from_bytes(
            super::REGULAR_TTF,
            FontSettings {
                scale,
                ..Default::default()
            },
        )
        .map_err(ioe4fe)?;
        let bold = Font::from_bytes(
            super::BOLD_TTF,
            FontSettings {
                scale,
                ..Default::default()
            },
        )
        .map_err(ioe4fe)?;

        let line_met = regular
            .horizontal_line_metrics(scale)
            .ok_or(ioe4fe("No horizontal line metrics"))?;
        // +1 to account for maybe having rounded ascent down
        // +1 to account for maybeh aving rounded descent up
        let height = line_met.new_line_size as usize + 2;
        let width = regular.metrics('m', scale).width;
        let ch_sz = XY(width, height);

        let line_baseline = line_met.ascent as usize + 1;

        let underline_top = height - regular.metrics('_', scale).height;

        Ok(Self {
            scale,
            regular,
            bold,
            ch_sz,
            line_baseline,
            underline_top,
        })
    }

    fn char_size(&self) -> XY {
        self.ch_sz
    }

    fn render(&self, window: &Window, screen: &Screen) -> io::Result<()> {
        let window_sz = XY(
            window.inner_size().width as usize,
            window.inner_size().height as usize,
        );
        let bounded_sz = {
            let max = window_sz / self.ch_sz;
            let sz = screen.size();
            XY(sz.x().min(max.x()), sz.y().min(max.y()))
        };
        let buffer_sz = (window_sz % self.ch_sz) / 2;

        let char_rows = (0..bounded_sz.y()).into_par_iter().flat_map(|y| {
            // how many pixels down from the top this starts
            let mut row_buf = vec![color_u32(Color::Black); window_sz.x() * self.ch_sz.y()];
            for x in 0..bounded_sz.x() {
                // how many pixels right from the left this starts
                let col = x * self.ch_sz.x() + buffer_sz.x();

                let cell = &screen[y][x];
                let fmt = cell.get_fmt();

                // select bold or normal font (this is really how we do bold, it bugs me too)
                let font = if fmt.bold { &self.bold } else { &self.regular };
                let (metrics, char_buf) = font.rasterize(cell.ch, self.scale);

                let ch_bottom = metrics.height as i32;
                // + because the axes are inverted (so really it's - (-metrics.ymin))
                let ch_baseline = (ch_bottom + metrics.ymin) as usize;
                // ch_baseline is now how far down the *raster* the character's baseline is
                // so we can align the raster's baseline to the line's baseline
                let y_offset;
                let y_cutoff;
                if ch_baseline <= self.line_baseline {
                    y_offset = self.line_baseline - ch_baseline;
                    y_cutoff = 0;
                } else {
                    y_offset = 0;
                    y_cutoff = ch_baseline - self.line_baseline;
                }

                // ditto for the x offset but that's easier because the "line baseline" is at 0
                let x_offset;
                let x_cutoff;
                if metrics.xmin >= 0 {
                    x_offset = metrics.xmin as usize;
                    x_cutoff = 0;
                } else {
                    x_offset = 0;
                    x_cutoff = -metrics.xmin as usize;
                }

                // now we can actually move the rasterized character onto the screen!
                for line_row in 0..self.ch_sz.y() {
                    let dest_row = line_row - y_cutoff;
                    let dest_start = (dest_row * window_sz.x()) + col - x_cutoff;
                    let dest_end = dest_start + self.ch_sz.x();
                    let dest = &mut row_buf[dest_start..dest_end];

                    if fmt.underline && line_row > self.underline_top {
                        dest.fill(color_u32(fmt.fg));
                        continue;
                    }

                    if line_row < y_offset || line_row >= metrics.height + y_offset - y_cutoff {
                        dest.fill(color_u32(fmt.bg));
                        continue;
                    }

                    for line_col in 0..self.ch_sz.x() {
                        if line_col < x_offset || line_col >= metrics.width + x_offset - x_cutoff {
                            dest[line_col] = color_u32(fmt.bg);
                            continue;
                        }

                        let char_row = line_row - y_offset + y_cutoff;
                        let char_col = line_col - x_offset + x_cutoff;
                        let val = char_buf[char_row * metrics.width + char_col];
                        let pct = val as f32 / 255.0;
                        let color = color_of(fmt.fg, fmt.bg, pct);
                        dest[line_col] = color;
                    }
                }
            }
            row_buf
        });
        let mut screen_buf = Vec::with_capacity(window_sz.x() * window_sz.y());
        screen_buf.resize(window_sz.x() * buffer_sz.y(), color_u32(Color::Black));
        screen_buf.par_extend(char_rows);
        screen_buf.resize(window_sz.x() * window_sz.y(), color_u32(Color::Black));

        // SAFETY: if winit betrays us we have no recourse
        let mut wh = unsafe { softbuffer::GraphicsContext::new(window, window) }
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;

        wh.set_buffer(&screen_buf, window_sz.x() as u16, window_sz.y() as u16);

        Ok(())
    }
}
