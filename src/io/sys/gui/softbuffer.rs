use std::io;

use fontdue::{Font, FontSettings};
use rayon::prelude::*;
use winit::window::Window;

use crate::io::{XY, output::Screen, clifmt::{Color, Formatted}};

use super::GuiBackend;

fn ioe4fe(e: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}

fn color_u32(c: Color) -> u32 {
    match c {
        Color::Black         => 0x00_00_00_00,
        Color::Red           => 0x00_80_00_00,
        Color::Green         => 0x00_00_80_00,
        Color::Yellow        => 0x00_80_80_00,
        Color::Blue          => 0x00_00_00_80,
        Color::Magenta       => 0x00_80_00_80,
        Color::Cyan          => 0x00_00_80_80,
        Color::White         => 0x00_80_80_80,
        Color::BrightBlack   => 0x00_0F_0F_0F,
        Color::BrightRed     => 0x00_FF_0F_0F,
        Color::BrightGreen   => 0x00_0F_FF_0F,
        Color::BrightYellow  => 0x00_FF_FF_0F,
        Color::BrightBlue    => 0x00_0F_0F_FF,
        Color::BrightMagenta => 0x00_FF_0F_FF,
        Color::BrightCyan    => 0x00_0F_FF_FF,
        Color::BrightWhite   => 0x00_FF_FF_FF,
    }
}

fn lerp(from: u32, to: u32, amt: f32) -> u32 {
    fn lerp_u8(from: u8, to: u8, amt: f32) -> u8 {
        let big = from.max(to);
        let lil = from.min(to);
        let amt = if lil == from { amt } else { 1.0 - amt };
        lil + ((big - lil) as f32 * amt) as u8
    }

    let fr = (from   >> 16   & 0xFF) as u8;
    let fg = (from   >> 8    & 0xFF) as u8;
    let fb = (from   >> 0    & 0xFF) as u8;
    let tr = (to     >> 16   & 0xFF) as u8;
    let tg = (to     >> 8    & 0xFF) as u8;
    let tb = (to     >> 0    & 0xFF) as u8;

    let r = lerp_u8(fr, tr, amt);
    let g = lerp_u8(fg, tg, amt);
    let b = lerp_u8(fb, tb, amt);

    (r as u32) << 16 | (g as u32) << 8 | (b as u32) << 0
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

#[async_trait::async_trait]
impl GuiBackend for SoftbufferBackend {
    fn new(scale: f32) -> io::Result<Self> {
        let regular = Font::from_bytes(super::REGULAR_TTF, FontSettings { scale, ..Default::default() }).map_err(ioe4fe)?;
        let bold = Font::from_bytes(super::BOLD_TTF, FontSettings { scale, ..Default::default() }).map_err(ioe4fe)?;

        let line_met = regular.horizontal_line_metrics(scale).ok_or(ioe4fe("No horizontal line metrics"))?;
        // +1 to account for maybe having rounded ascent down
        // +1 to account for maybeh aving rounded descent up
        let height = line_met.new_line_size as usize + 2;
        let width = regular.metrics('m', scale).width;
        let ch_sz = XY(width, height);

        let line_baseline = line_met.ascent as usize + 1;

        let underline_top = height - regular.metrics('_', scale).height;

        Ok(Self { scale, regular, bold, ch_sz, line_baseline, underline_top })
    }

    fn char_size(&self) -> XY {
        self.ch_sz
    }

    async fn render(&self, window: &Window, screen: &Screen) -> io::Result<()> {
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

        // let mut screen_buf = vec![color_u32(Color::Black); window_sz.x() * window_sz.y()];
        let char_rows = (0..bounded_sz.y()).into_par_iter().flat_map(|y| {
            // how many pixels down from the top this starts
            let mut row_buf = vec![color_u32(Color::Black); window_sz.x() * self.ch_sz.y()];
            for x in 0..bounded_sz.x() {
                // how many pixels right from the left this starts
                let col = x * self.ch_sz.x() + buffer_sz.x();

                let cell = &screen[y][x];
                let fmt = cell.get_fmt();

                // TODO: Select bold or normal font
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

                let fg = color_u32(fmt.fg);
                let bg = color_u32(fmt.bg);

                // now we can actually move the rasterized character onto the screen!
                for line_row in 0..self.ch_sz.y() {
                    let dest_row = line_row - y_cutoff;
                    let dest_start = (dest_row * window_sz.x()) + col - x_cutoff;
                    let dest_end = dest_start + self.ch_sz.x();
                    let dest = &mut row_buf[dest_start..dest_end];

                    if fmt.underline && line_row > self.underline_top {
                        dest.fill(fg);
                        continue;
                    }

                    if line_row < y_offset || line_row >= metrics.height + y_offset - y_cutoff {
                        dest.fill(bg);
                        continue;
                    }

                    for line_col in 0..self.ch_sz.x() {
                        if line_col < x_offset || line_col >= metrics.width + x_offset - x_cutoff {
                            dest[line_col] = bg;
                            continue;
                        }

                        let char_row = line_row - y_offset + y_cutoff;
                        let char_col = line_col - x_offset + x_cutoff;
                        let val = char_buf[char_row * metrics.width + char_col];
                        // TODO: switch to [f32; 3] color and u8 from rasterized?
                        let pct = val as f32 / 255.0;
                        let color = lerp(bg, fg, pct);
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
        let mut wh = unsafe { softbuffer::GraphicsContext::new(window) }
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e)))?;

        wh.set_buffer(&screen_buf, window_sz.x() as u16, window_sz.y() as u16);

        Ok(())
    }
}
