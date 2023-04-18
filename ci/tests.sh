#!/bin/sh

set -e

miri=""
parallel="0"
for arg in "$@"; do
    case "$arg" in
    --miri)
        miri="miri"
        parallel="1"
        ;;
    --para)
        parallel="1"
        ;;
    esac
done

lines_to() {
    label="$1"
    while read -r line; do
        echo "[$label]" $line
    done
}
para_setup() {
    if [ "$parallel" = "1" ]; then
        pipe="$(mktemp -u tuig-ci-pipe.XXXXXXXXXX)"
        mkfifo $pipe
        trap 'trap - TERM; para_kill' INT TERM
    fi
}
para() {
    label="$1"
    shift
    if [ "$parallel" = "1" ]; then
        ("$@" 2>&1 | lines_to "$label" >> "$pipe") &
    else
        "$@"
    fi
}
para_kill() {
    if [ "$parallel" = "1" ]; then
        rm "$pipe"
        kill -- -$$
    fi
}
para_teardown() {
    if [ "$parallel" = "1" ]; then
        cat "$pipe"
        wait
        rm "$pipe"
    fi
}

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
para_setup
for run in $RUNNERS; do
    for sys in $SYSTEMS; do
        std="$(needs_std "$run" "$sys")"
        FEATS="run_$run,io_$sys,$std"
        cargo check --features "$FEATS" --all-targets
        para "run:$run,sys:$sys" cargo $miri test --features "$FEATS" --all-targets --no-fail-fast
    done
done
para_teardown
