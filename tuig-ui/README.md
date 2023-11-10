# tuig-ui

tuig-ui is a predictable immediate-mode UI system for character grids.

## Usage

Take a look at [the examples] for a starting point.
You'll need to run a 'main loop'/'UI thread':

- Get the next user input.
- Create a [`Region`] with that and a [`Screen`].
- Attach an [`Attachment`] of some kind to render.
  (Usually your top level doesn't need to literally implement `Attachment`, because you can just directly use the `Region`.)
- Draw the [`Screen`] to the actual screen.

The *expected* way of doing the first and last steps is with [`tuig-iosys`], but there's no reason you couldn't create `Action`s/`Screen`s yourself from whatever source you find convenient.

  [the examples]: https://github.com/nic-hartley/tuig/tree/release/tuig-ui/examples
  [`Attachment`]: https://docs.rs/tuig-ui/latest/tuig_ui/trait.Attachment.html
  [`Region`]: https://docs.rs/tuig-ui/latest/tuig_ui/struct.Region.html
  [`Screen`]: https://docs.rs/tuig-iosys/latest/tuig_iosys/struct.Screen.html

## FAQ

### What do you mean, "predictable"?

Textmode or character grid UI is weird because you're usually working with an extremely small "resolution", in terms of the size of the grid -- the window I'm writing this in is about 130 columns by 60 rows -- but each "pixel" can contain quite a lot of information.
As a result, you want a UI whose layout you can be sure you'll consistently control.
No fighting with whitespace or single pixels or margins or weird content wrapping; things just go where you put them.

The downside is that, to stay predictable, it has to be simple.
Considering the types of UIs usually seen in textmode, though, it's still plenty for most use-cases.

### What's immediate-mode?

[Immediate mode] refers to an API style in graphics programming where the APIs you call (pretend to) directly draw on the screen.
In the case of a UI, it means that the UI elements are defined -- with the associated input handling and rendering run at the same time -- during the course of each frame.

For example, an immediate-mode UI might look like:

```rs
struct ChatWindow {
  friends: Vec<People>,
  current: usize,
  histories: Vec<History>,
  partial: String,
}
impl<'s> Attachment<'s> for &ChatWindow {
  type Output = ();
  fn attach(self, root: Region<'s>) {
    let (people, right) = root.split(cols!(15 *));
    if let Some(sel) = people.attach(FriendsList::new(&self.friends, &self.current)) {
      self.current = sel;
    }
    let (history, entry) = right.split(rows!(* 1));
    history.attach(ChatHistory::new(&self.histories[self.current]));
    if entry.attach(TextInput::new(&mut self.partial)).is_submitted() {
      self.histories[self.current].add(mem::take(self.partial));
    }
  }
}
```

Notice:

- You didn't store any UI elements.
  The state kept in your object is just the state you care about rendering.
- Your `attach` function is defining both your UI layout *and* your responses to inputs.
  Notice e.g. that a `FriendsList`, when attached, returns what friend the user just selected, if any, and the `TextInput` returns a complex state object that can describe many things.

On the one hand, this approach tends to lead to simpler code.
You don't need to worry about callbacks or element trees or anything else; you just directly code what you want your UI to look like, and that's that.
On the other, your UI is defined in a function, and that function is also where all your event-handling code is, so it has to run on every single input, plus periodically when there's no input to display external state changes.
That can hog CPU, compared to lazily waiting for inputs and only executing the precise callbacks that are needed.

Even so, it's really not *that* much, unless your UI is incredibly complicated.
Considering that `tuig-ui` was designed first and foremost to be easy to use, with performance an important but secondary concern, the tradeoff is very worth it.

  [Immediate mode]: https://en.wikipedia.org/wiki/Immediate_mode_(computer_graphics)
