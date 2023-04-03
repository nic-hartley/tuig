#!/bin/sh

set -e

VERSIONS="stable 1.64 nightly"
RUNNERS="single rayon"
SYSTEMS="nop cli_crossterm gui_softbuffer"

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

# setup, predownloading as much as possible in parallel
trap 'trap - INT; wait' INT
for version in $VERSIONS; do
    rustup toolchain install "$version" &
done
wait

# output dir so we can only output when it's actually relevant
outdir="$(mktemp -d)"

cargo fmt --check
for version in $VERSIONS; do
    for run in $RUNNERS; do
        for sys in $SYSTEMS; do
            std="$(needs_std "$run" "$sys")"
            out="$outdir/$version-$run-$sys.log"
            FEATS="run_$run,io_$sys,$std"
            if ! (
                cargo +"$version" check --features "$FEATS" --all-targets
                cargo +"$version" test --features "$FEATS" --all-targets
            ) 2>&1 >"$out"; then
                cat "$out"
            fi
        done
    done
done

for run in $RUNNERS; do
    std="$(needs_std "$run" "nop")"
    cargo +stable run --release --bin mass-messages --features "run_$run,io_nop,$std"
done
