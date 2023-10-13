# tuig

[tuig] is a game engine hyperfocused on systemic textmode games.

[tuig-ui] is the underlying character grid-based UI system, and [tuig-iosys] is its textmode input/output system.
You can use them mostly independently, if you don't find yourself needing a whole game engine.

They live in the same repo because they're pretty tightly coupled.
You can use `tuig-ui` and `tuig-iosys` on their own, and using `tuig` doesn't require understanding `tuig-iosys`, but the design of each informs the others.

## FAQ

### Do I want `tuig-iosys`, `tuig-ui`, or `tuig`?

That depends.

You want `tuig` for a full game engine with:

- A UI hyperfocused on textmode games
- An agent-and-message structure that enables systemic games
- And more!

You want `tuig-ui` for a *UI* system with:

- Easy integration with `tuig-iosys`, but backend-independent.
- Clearly defined and predictable UI element layout
- A well-stocked library of default components

You want `tuig-iosys` for a bare IO system with:

- A fast way to draw character grids to various targets
- Standardized input/output for any character grid backend
- (Coming eventually!) Built-in support for screen effects, e.g. simulating a CRT display

And you want none of them if you're trying to do general-purpose GUI things, or simple line-based console output.

### What's textmode?

Textmode, text UI, or text-based games are video games whose output is exclusively a grid of typical text characters.
The quintessential example of a textmode game is Dwarf Fortress, but things like Zork and Rogue count too.

It's usually used in contrast with "graphical" games -- the ones most people think of when they think video games.
Everything from the latest and greatest photorealistic AAA crunch nightmare to Pong is a graphical game.

The line tuig draws is pretty simple:
Can it run in a terminal?

There's some hidden technical complexity underlying that intuition, but mostly it lines up.
Check the [tuig-iosys] docs for details.

  [tuig-iosys]: https://github.com/nic-hartley/tuig/blob/release/tuig-iosys/README.md

### What's systemic?

Systemic games design and simulate their worlds as complete, interlocking systems, rather than tailoring the simulation to the specific intended play experience.
Something like Deus Ex is a classic systemic game, as is Dwarf Fortress.
COD:MW2 and Portal aren't.
A good rule of thumb is that the more solutions there are to any given in-game problem, the more systemic the game is.

There isn't as clear a line as with textmode vs. graphical games.
In tuig's case, it's enabled through the message system, which has agents listening for messages they're interested in rather than being reached out to.
It's designed to more cleanly represent a *world*, whose components can interact in a variety of ways.

### How do I use tuig?

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

### What else does tuig offer?

Not much, yet!
Keep an eye on the [v0.1 milestone] for updates.

  [v0.1 milestone]: https://github.com/nic-hartley/redshell/milestone/1

### How does this repo do versioning?

`tuig` and co. try to follow semantic versioning, but the line between "bugfix", "new feature", and "breaking change" can be difficult to draw.
In short:
- If it's undocumented, I can change it however I like and count it as a bugfix.
- If it's documented, *just adding things* is never a breaking change, only a new feature -- even if I forgot to add `#[non_exhaustive]` to that enum. (Though I'll try very hard to make sure your code works without changes for minor version bumps, regardless.)
- Changing documented things is almost certainly breaking regardless of the change.

The versions are also pinned to each other, so when `vX.Y.Z` of `tuig-iosys` comes out, `tuig` will get updated as well, 
If you're fighting dependency hell, there's some leeway in there, but by and large `tuig` vX.Y.Z will depend on `tuig-iosys` vX.Y.Z, etc.

  [tuig]: https://crates.io/crates/tuig
  [tuig-ui]: https://crates.io/crates/tuig-ui
  [tuig-iosys]: https://crates.io/crates/tuig-iosys
