#!/bin/sh

CRATES="tuig-pm tuig-iosys tuig-ui tuig"

mode="$1"

set -e

top="$(git rev-parse --show-toplevel)"
cd "$top"
for crate in $CRATES; do
    if ! [ -d "$crate" ]; then
        echo "Can't find $crate -- wrong dir? crates need updating?"
        exit 1
    fi
done
cargo fmt --check
cargo build --all-features --workspace --all-targets
cargo doc --all-features --workspace --no-deps --lib --bins --examples
cargo test --all-features --workspace --all-targets

case "$mode" in
    pub*)
        for crate in $CRATES; do
            cargo publish -p "$crate"
        done
        ;;
    precom*) ;;
    *)
        for crate in $CRATES; do
            cargo publish -p "$crate" --dry-run
        done
        ;;
esac
