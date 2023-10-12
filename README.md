# tuig

*and friends!*

[tuig] is a game engine hyperfocused on textmode, systemic games.
[tuig-ui] is the character grid-based UI system it uses.
[tuig-iosys] is the textmode input/output system *it* uses.

They live in the same repo because they're pretty tightly tied together.
You can use `tuig-ui` and `tuig-iosys` on their own, and using `tuig` doesn't require understanding `tuig-iosys`, but the design of each informs the others.

## FAQ

### Do I want `tuig-iosys`, `tuig-ui`, or `tuig`?

That depends.

You want `tuig-iosys` for a bare IO system with:

- A fast way to draw character grids to various targets
- Standardized input/output for any character grid backend
- (Coming eventually!) Built-in support for screen effects, e.g. simulating a CRT display

You want `tuig-ui` for a *UI* system with:

- Easy integration with `tuig-iosys`, but backend-independent.
- Clearly defined and predictable UI element layout
- A well-stocked library of default components

You want `tuig` for a full game engine with:

- A UI hyperfocused on textmode games
- An agent-and-message structure that enables systemic games
- And more!

And you want none of them if you're trying to do general-purpose GUI things, or simple line-based console output.

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
