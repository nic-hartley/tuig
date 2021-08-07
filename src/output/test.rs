use crate::output::XY;

use super::Text;

pub struct TestScreen;

impl TestScreen {
    pub fn get() -> TestScreen {
        println!("Starting output...");
        TestScreen
    }
}

impl Drop for TestScreen {
    fn drop(&mut self) {
        println!("Done with output!");
    }
}

impl super::Screen for TestScreen {
    fn clear(&mut self) {
        println!("Clearing screen");
    }

    fn flush(&mut self) {
        println!("== FLUSH ==");
        println!();
    }

    fn size(&self) -> XY {
        println!("Getting size! (oh no! what's the size?)");
        XY(80, 24)
    }

    fn write_raw(&mut self, text: Vec<Text>, pos: XY) {
        println!("Writing some text to screen at {}:", pos);
        for t in text {
            println!("- {:?}", t);
        }
    }
}
