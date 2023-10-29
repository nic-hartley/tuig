#!/bin/sh

CRATES="tuig-pm tuig-iosys tuig-ui tuig"

# parameter should be:
# - fix (default): run lightweight checks and fix things automatically where possible
# - check: run all checks w/o fixing (including publish dry-drun -- likely to fail for bumped versions)
# - publish: run all checks, and if they pass, publish all the crates to crates.io.
mode="$1"
if [ -z "$mode" ]; then
    mode="fix"
fi

set -e

top="$(git rev-parse --show-toplevel)"
cd "$top"
for crate in $CRATES; do
    if ! [ -d "$crate" ]; then
        echo "Can't find $crate -- wrong dir? crates need updating?"
        exit 1
    fi
done

case "$mode" in
    f*)
        cargo fmt
        cargo clippy --fix --allow-dirty --allow-staged
        cargo check
        ;;
    *)
        cargo fmt --check
        cargo clippy
        cargo build --all-features --workspace --all-targets
        cargo doc --all-features --workspace --no-deps --lib --bins --examples
        cargo test --all-features --workspace --all-targets
esac

case "$mode" in
    p*)
        for crate in $CRATES; do
            cargo publish -p "$crate"
        done
        ;;
esac
