use crate::io::XY;

use super::Text;

const SIZE: XY = XY(80, 24);

pub struct TestScreen {
    chars: [char; SIZE.x() * SIZE.y()],
}

impl TestScreen {
    pub fn get() -> TestScreen {
        println!("Starting output...");
        TestScreen { chars: [' '; SIZE.x() * SIZE.y()] }
    }
}

impl Drop for TestScreen {
    fn drop(&mut self) {
        println!("Done with output!");
    }
}

#[async_trait::async_trait]
impl super::Screen for TestScreen {
    fn size(&self) -> XY {
        println!("Getting size! (oh no! what's the size?)");
        SIZE
    }

    fn write_raw(&mut self, text: Vec<Text>, pos: XY) {
        println!("Writing text {:?} to {:?}", text, pos);
        let string = text.into_iter().map(|t| t.text).collect::<String>();
        let line_start = pos.y() * SIZE.x();
        let start = line_start + pos.x();
        let end = std::cmp::min(start + string.len(), line_start + SIZE.x());
        let mut chars = string.chars();
        for i in start..end {
            self.chars[i] = chars.next().unwrap();
        }
    }

    async fn flush(&mut self) {
        println!("     0         1         2         3         4         5         6         7         80");
        for y in 0..SIZE.y() {
            let start = y * SIZE.x();
            let end = start + SIZE.x();
            let line = &self.chars[start..end];
            println!("{:>4}:{}", y, line.iter().collect::<String>());
        }
        println!();
        self.chars = [' '; SIZE.x() * SIZE.y()];
    }

    fn clear(&mut self) {
        println!("Clearing screen");
        self.chars = [' '; SIZE.x() * SIZE.y()];
    }

    async fn clear_raw(&mut self) {
        println!("Really clearing screen");
    }
}
