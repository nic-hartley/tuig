# tuig-iosys

`tuig-iosys` is the textmode renderer used by [`tuig`](https://crates.io/crates/tuig).
You can use it separately if all you want is to render a character grid.

## Usage

`tuig-iosys` is a typical Rust crate, so you can add it to your project with `cargo add tuig-iosys`.
And then [the docs](https://crates.io/crates/tuig-iosys), especially the [`docs` module](https://docs.rs/tuig-iosys/latest/tuig_iosys/docs), will tell you how to use the library.

Do keep in mind that `tuig-iosys` is a pretty low-level crate.
It's a little like the textmode equivalent of [`softbuffer`]: You can use it to render a *character* grid, rather than a pixel grid, and get basic events back.
That's about it.

  [`softbuffer`]: https://github.com/rust-windowing/softbuffer

## FAQ

*(Don't forget to check out [the repo's README] for the cross-crate FAQ, too!)*

  [the repo's README]: https://github.com/nic-hartley/tuig

### What's textmode?

Textmode, text UI, or text-based interfaces are those whose output is exclusively a grid of typical text characters.
Think `nmtui`, or Dwarf Fortress, and contrast it with a GUI or a webpage -- which might only have text *content*, but can render more complex shapes than 

The line `tuig-iosys` draws is the same as `tuig`:
Can it run in a terminal?

There's some hidden technical complexity underlying that intuition, but mostly it lines up.

#### What hidden technical complexity?

First, you can actually do a lot with traditional "textmode" systems. For example, custom character sets can allow for extremely complex graphics, even pixel-perfect rendering.
Check out your local demo scene sometime; they do really cool stuff.
`tuig-iosys` doesn't try to replicate that -- theoretically you could use its font support to accomplish the same thing, but you're more or less on your own there.
(This also covers using ▀/▄ or Braille.)
The most it supports easily is bitmap-based fonts for tilesets.

Similarly, modern terminals are actually... really cool.
Some allow embedding images, some can be drawn to like any pixel buffer, some are just magic.
Again, `tuig-iosys` doesn't try to replicate any of that functionality.
You get character grids, and you like it!
More seriously, if you want advanced terminal features, use an advanced terminal library.
This isn't meant to provide access to every feature your terminal supports, it's meant to render things in a retro style.

The most frustrating bit of complexity for users of the library, though, is that even among reasonable character grids, there's some wide variance between available features.
Neither `tuig-iosys` nor `tuig` tries to normalize them, because there's no one obvious "right way" to replace e.g. underline on backends without support for it.
Instead, it's left up to the library user to figure out what common idioms to use, and how they should degrade.

(Tip: Try creating a `trait MyFormatting: FormattedExt`, and using that to define things like "highlight" or "trim color".)

Finally: Terminals have differing behavior on special characters.
`tuig-iosys` usually tries to normalize to the lowest common denominator.
It might not behave how you expect if you write an ASCII BEL, for example.
