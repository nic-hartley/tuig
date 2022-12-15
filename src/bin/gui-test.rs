use std::io;

use redshell::io::{sys::{gui::{GuiBackend, Gui}, IoSystem}, XY, output::{Screen, Cell, FormattedExt, Color}};
use winit::window::Window;

struct NopBackend;
#[async_trait::async_trait]
impl GuiBackend for NopBackend {
    fn new(_: f32) -> std::io::Result<Self> {
        Ok(Self)
    }

    fn char_size(&self) -> XY {
        XY(10, 20)
    }

    async fn render(&self, _: &Window, _: &Screen) -> io::Result<()> {
        Ok(())
    }
}

async fn real_main(mut iosys: impl IoSystem) {
    let mut sc = Screen::new(iosys.size());
    loop {
        println!("{:?}", iosys.input().await);
        sc.resize(iosys.size());
        for x in 0..sc.size().x() {
            for y in 0..sc.size().y() {
                sc[y][x] = Cell::of(' ')
                    .bg(Color::all()[(x + y) % Color::count()]);
            }
        }
        iosys.draw(&sc).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    let win = Gui::<NopBackend>::new(10.0).await.unwrap();
    real_main(win).await
}
