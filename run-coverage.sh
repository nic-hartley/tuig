#!/bin/sh

if [ "$CI" = "true" ]; then
    output="--html"
else
    output="--open"
fi
cargo +nightly llvm-cov test \
    --all-features --fail-under-lines 90 \
    --ignore-filename-regex 'bin/*|cutscenes/*' \
    --ignore-filename-regex 'app/cli\.rs' \
    $output
