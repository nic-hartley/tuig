#!/bin/sh

# Files/folders to be ignored for unit test coverage
IGNORE="^$"
# None of these should be complex enough to cause problems, and they'll
# change so much over time that I don't want to break the build for tests
# which I'll have to rewrite soon anyway
IGNORE="$IGNORE|bin/*"
# Cutscenes are primarily visual; there might be some tests in there but most
# of the code is visual code we don't bother testing
IGNORE="$IGNORE|cutscenes/*"
# Everything in clifmt.rs is really simple. It's basically all setters.
IGNORE="$IGNORE|io/clifmt\.rs"

# These are going to be rewritten soon
IGNORE="$IGNORE|app/cli\.rs"

if [ "$CI" = "true" ]; then
    output="--html"
else
    output="--open"
fi
cargo +nightly llvm-cov test \
    --all-features --fail-under-lines 90 \
    --ignore-filename-regex "$IGNORE" \
    $output
