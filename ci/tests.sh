#!/bin/sh

set -e

# run_* features
RUNNERS="single rayon"
# io_* features
SYSTEMS="nop cli_crossterm gui_softbuffer"

# needs_std $RUN $IO indicates whether a given runner/iosystem pair needs the std feature
needs_std() {
    case "$1" in
    single|rayon)
        echo "std"
        return
        ;;
    esac
    case "$2" in
    nop|cli_crossterm|gui_softbuffer)
        echo "std"
        return
        ;;
    esac
}

cargo fmt --check
for run in $RUNNERS; do
    for sys in $SYSTEMS; do
        std="$(needs_std "$run" "$sys")"
        FEATS="run_$run,io_$sys,$std"
        cargo check --features "$FEATS" --all-targets
        cargo test --features "$FEATS" --all-targets
    done
done
