# tuig-iosys

tuig-iosys is the textmode renderer used by [`tuig`](https://crates.io/crates/tuig).
You can use it separately if all you want is to render to a text grid.

## Usage

`tuig-iosys` is a typical Rust crate, so you can add it to your project:

```sh
cargo add tuig-iosys
```

And then [the docs](https://crates.io/crates/tuig-iosys), especially the [`docs` module](https://docs.rs/tuig-iosys/latest/tuig_iosys/docs), will tell you how to use the library.

## FAQ

### What's textmode?

Textmode, text UI, or text-based interfaces are those whose output is exclusively a grid of typical text characters.
Think `nmtui`, or Dwarf Fortress, and contrast it with a GUI or a webpage -- which might only have text *content*, but can render more complex shapes than 

The line `tuig-iosys` draws is the same as `tuig`:
Can it run in a terminal?

There's some hidden technical complexity underlying that intuition, but mostly it lines up.

#### What hidden technical complexity?

First, you can actually do a lot with traditional "textmode" systems. For example, custom character sets can allow for extremely complex graphics, even pixel-perfect rendering.
Check out your local demo scene sometime; they do really cool stuff.
`tuig-iosys` doesn't try to replicate that -- theoretically you could use its font support to accomplish the same thing, but you're on your own there.
(This also covers using ▀/▄ or Braille.)

Similarly, modern terminals are actually... really cool.
Some allow embedding images, some can be drawn to like any pixel buffer, some are just magic.
Again, `tuig-iosys` doesn't try to replicate any of that functionality.
You get character grids, and you like it!

The most frustrating bit of complexity for users of the library, though, is that even among reasonable character grids, there's some wide variance between available features.
Neither `tuig-iosys` nor `tuig` tries to normalize them, because there's no one obvious "right way" to replace e.g. underline on backends without support for it.
Instead, it's left up to the library user to figure out what common idioms to use, and how they should degrade.

(Tip: Try creating a `trait MyFormatting: FormattedExt`, and using that to define things like "highlight" or "trim color".)

Finally: Terminals have differing behavior on special characters.
`tuig-iosys` usually tries to normalize to the lowest common denominator.
It might not behave how you expect if you write an ASCII BEL, for example.

### Do I want this or `tuig`?

That depends.

You want `tuig-iosys` for an IO system with:

- A fast way to draw character grids to various targets
- A text UI system that focuses on predictable layout
- Standardized input/output for any character grid backend

You want `tuig` for a game engine with:

- A hyperfocus on textmode games
- An agent-and-event structure that enables systemic games
- And more!

And you want *neither* if you're trying to general-purpose GUI things, or simple console output.

### How does `tuig-iosys` do versioning?

`tuig-iosys` tries to follow semantic versioning, but the line between "bugfix", "new feature", and "breaking change" can be difficult to draw.
In short:
- If it's undocumented, I can change it however I like and count it as a bugfix.
- If it's documented, *just adding things* is never a breaking change, only a new feature -- even if I forgot to add `#[non_exhaustive]` to that enum. (Though I'll try very hard to make sure your code works without changes for minor version bumps, regardless.)
- Changing documented things is almost certainly breaking regardless of the change.
