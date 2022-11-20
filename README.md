# Redshell

For now, this is a half-built engine and some 'concept art'.
You can try it out by installing from crates.io:

```sh
cargo install redshell
```

Or you can install it from source:

```sh
git clone https://github.com/nic-hartley/redshell.git
cd redshell
cargo install --path .
```

Once installed, you'll have a `redshell` binary, which for now does nothing, and a `redshell-concept` binary, which will let you run miscellaneous tech demos that I've very generously called 'concept art'.

Eventually, this crate will contain:

- The Redshell game itself, implemented as a library, parameterized over the IO subsystem
- A variety of IO systems targeting a variety of platforms (native/Steam/web/etc.)
- A large handful of `redshell` binary variants, tailored to each platform

This will make extremely heavy use of features, so I'll explain my intended setup here:

- `sys_*` are the IO subsystems available. At least one must be selected.
  - `sys_gui_*` are the various flavors of GUI rendering backend
- `save_*` are the saving/loading subsystems available. Select as many as you want, though some are only compatible with certain platforms.
- `plat_*` are the platforms available to be targeted. At most one must be selected.
  - These swap out the `main` function in the `redshell` binary, and pick the `sys_*`/`save_*` appropriate for the target.

If you're doing anything besides developing on `redshell` itself you almost certainly want to pick **only** a `plat_*` feature and nothing else.
