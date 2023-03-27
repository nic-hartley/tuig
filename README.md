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

- `io_*` are the IO subsystems available. At least one must be selected.
  - `io_gui_*` are the various flavors of GUI rendering backend.
  - If several are selected, they're chosen from at runtime; see `redshell::io::sys::load`.
- `run_*` are the agent runners. Exactly one must be selected.
  - These decide how agents, the `Game`, and the IO system are run, e.g. what type of multithreading is used.
- `plat_*` are the platforms available to be targeted. At most one must be selected.
  - These pick the other features appropriate for the target. Don't use it with others.

If you're doing anything besides developing on `redshell` itself you almost certainly want to pick **only** a `plat_*` feature and nothing else.

## Versioning

Redshell, despite being on crates.io, **does not really follow semver**.
The goal is that:

- Patch versions are bumped for small balance tweaks, bugfixes, etc.
- Minor versions are bumped when gameplay changes in significant ways or mods are likely to become incompatible
- Major versions are bumped with large gameplay changes (which will probably break mods, too)

So a mod written for, say, 1.2.3 should still be compatible with 1.2.4, though it may be unbalanced, but it might not load anymore on 1.3.0 and probably won't on 2.0.0.
Gameplay strategies will break more frequently, as they're much more sensitive to balance changes.
