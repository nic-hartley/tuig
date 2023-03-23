# tuig

tuig is a game engine hyperfocused on textmode systemic games.

## I want to make a game with tuig.

Checking out [`tuig::docs::walkthrough`]!
In maybe ten minutes you'll have a simple tic-tac-toe game with an AI.

  [`tuig::docs::walkthrough`]: https://docs.rs/tuig/latest/tuig/docs/walkthrough

## FAQ

### How usable is tuig, *right now*?

Not especially!

I'm actively working on [redshell] in this engine, and using that to inform the development of the engine itself.
It's at 0.0.x for a reason, and I don't see it even hitting 0.1 for a fair while.
If you do still want to make a game in it, please [raise issues] as you encounter them.
Undocumented items, missing features, bugs, ugly APIs, whatever -- I can't fix it if I don't know about it!

And if you're just interested in watching, keep an eye on the [issue list], in particular the [v0.1 milestone].

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

### What's systemic?

Systemic games design and simulate their worlds as complete, interlocking systems, rather than tailoring the simulation to the specific intended play experience.
Something like Deus Ex is a classic systemic game, as is Dwarf Fortress.
COD:MW2 and Portal aren't.
A good rule of thumb is that the more solutions there are to any given in-game problem, the more systemic the game is.

There isn't as clear a line as with textmode vs. graphical games.
In tuig's case, it's enabled through the event system, which has agents listening for events they're interested in rather than being reached out to.
It's designed to more cleanly represent a *world*, whose components can interact in a variety of ways.

### What else does tuig offer?

Not much, yet.
You can see the roadmap in the [issue list] and [v0.1 milestone].

### How does tuig do versioning?

tuig tries to follow semantic versioning, but the line between "bugfix", "feature", and "breaking change" can be difficult to draw.
In short:
- If it's undocumented, I can change it however I like and count it as a bugfix.
- If it's documented, *just adding things* is never a breaking change, only a feature -- even if I forgot to add `#[non_exhaustive]` to that enum. (Though I'll try very hard to make sure your code works without changes for minor version bumps, regardless.)
- Changing documented things is almost certainly breaking regardless of the change.

  [tuig-iosys]: https://docs.rs/tuig-iosys
  [redshell]: https://github.com/nic-hartley/redshell/
  [raise issues]: https://github.com/nic-hartley/redshell/issues/new
  [issue list]: https://github.com/nic-hartley/redshell/issues
  [v0.1 milestone]: https://github.com/nic-hartley/redshell/milestone/1
