use redshell::{io::{input::{Input, ansi_cli::AnsiInput, Action}, output::{ansi_cli::AnsiScreen, Screen}, XY}, text};

#[tokio::main]
async fn main() {
    println!("Displaying mouse events at cursors");
    println!("Exit with any keyboard input");
    let mut input = AnsiInput::get()
        .expect("Failed to construct input");
    let mut output: Box<dyn Screen> = Box::new(AnsiScreen::get()
        .expect("Failed to construct output"));
    loop {
        let text;
        let at;
        match input.next().await {
            Action::KeyPress { .. } | Action::KeyRelease { .. } => break,
            Action::MousePress { button, pos } => {
                text = format!("{:?} button pressed at {:?}", button, pos);
                at = pos;
            }
            Action::MouseRelease { button, pos } => {
                text = format!("{:?} button released at {:?}", button, pos);
                at = pos;
            }
            Action::MouseMove { button: Some(b), pos } => {
                text = format!("Moved to {:?} holding {:?}", pos, b);
                at = pos;
            }
            Action::MouseMove { button: None, pos } => {
                text = format!("Moved to {:?} holding nothing", pos);
                at = pos;
            }
            Action::Unknown(desc) => {
                text = format!("Unknown input: {}", desc);
                at = XY(0, 0);
            }
            Action::Error(msg) => {
                text = format!("Error: {}", msg);
                at = XY(0, 0);
            }
        };
        output.textbox(text!("{}"(text)))
            .xy(at);
        output.flush().await;
    }
}
