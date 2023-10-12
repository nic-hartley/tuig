# tuig

tuig is a game engine hyperfocused on systemic textmode games.

## Usage

Check out [`tuig::docs::walkthrough`]!
In maybe ten minutes you'll have a simple tic-tac-toe game with an AI.

  [`tuig::docs::walkthrough`]: https://docs.rs/tuig/latest/tuig/docs/walkthrough

The short version, though, is:

- `cargo add tuig` to add it to your project.
  - Pick the features you want: `io_cli_crossterm,run_rayon` is a good, broadly compatible set to start with.
    Just don't forget to handle `Ctrl+C`!
- Implement [`Message`]: This is how the pieces talk to each other.
- Implement [`Game`]: This is how you talk to the player.
- Implement [`Agent`]: This is how you simulate things.
- Use a [`Runner`] to get everything running.

  [`Message`]: https://docs.rs/tuig/latest/tuig/trait.Message.html
  [`Game`]: https://docs.rs/tuig/latest/tuig/trait.Game.html
  [`Agent`]: https://docs.rs/tuig/latest/tuig/trait.Agent.html

## FAQ

*(Don't forget to check out [the repo's README] for the cross-crate FAQ, too!)*

  [the repo's README]: https://github.com/nic-hartley/tuig

### How usable is tuig, *right now*?

Not especially!

I'm actively working on [redshell] in this engine, and using that to inform the development of the engine itself.
It's at 0.0.x for a reason, and I don't see it even hitting 0.1 for a fair while.
If you do still want to make a game in it, please [raise issues] as you encounter them.
Undocumented items, missing features, bugs, ugly APIs, whatever -- I can't fix it if I don't know about it!

And if you're just interested in watching, keep an eye on the [issue list].

  [redshell]: https://github.com/nic-hartley/redshell/
  [raise issues]: https://github.com/nic-hartley/redshell/issues/new
  [issue list]: https://github.com/nic-hartley/redshell/issues

### What's textmode?

Textmode, text UI, or text-based games are video games whose output is exclusively a grid of typical text characters.
The quintessential example of a textmode game is Dwarf Fortress, but things like Zork and Rogue count too.

It's usually used in contrast with "graphical" games -- the ones most people think of when they think video games.
Everything from the latest and greatest photorealistic AAA crunch nightmare to Pong is a graphical game.

The line tuig draws is pretty simple:
Can it run in a terminal?

There's some hidden technical complexity underlying that intuition, but mostly it lines up.
Check the [tuig-iosys] docs for details.
Like the name implies, it's what tuig uses for its IO subsystem.

  [tuig-iosys]: https://docs.rs/tuig-iosys

### What's systemic?

Systemic games design and simulate their worlds as complete, interlocking systems, rather than tailoring the simulation to the specific intended play experience.
Something like Deus Ex is a classic systemic game, as is Dwarf Fortress.
COD:MW2 and Portal aren't.
A good rule of thumb is that the more solutions there are to any given in-game problem, the more systemic the game is.

There isn't as clear a line as with textmode vs. graphical games.
In tuig's case, it's enabled through the message system, which has agents listening for messages they're interested in rather than being reached out to.
It's designed to more cleanly represent a *world*, whose components can interact in a variety of ways.

### What else does tuig offer?

Not much, yet!
Keep an eye on the [v0.1 milestone] for updates.

  [v0.1 milestone]: https://github.com/nic-hartley/redshell/milestone/1
