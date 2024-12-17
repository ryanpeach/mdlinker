# Runs tests and supresses the progress bar
test:
    RUNNING_TESTS=true RUST_LOG=debug RUST_BACKTRACE=1 cargo test --

test-print test_name:
    RUNNING_TESTS=true RUST_LOG=trace RUST_BACKTRACE=1 cargo test -- --test-threads=1 {{test_name}}

[macos]
test-debug test_name breakpoint:
    #!/bin/bash
    TEST_OUTPUT=$(RUNNING_TESTS=true cargo test --no-run 2>&1 >/dev/null)
    DEP1=$(echo $TEST_OUTPUT | grep -ohe 'Executable tests/logseq/main.rs (target/debug/deps/logseq-[a-z0-9]*' | awk -F'[()]' '{print $2}')
    echo $DEP1
    RUNNING_TESTS=true RUST_LOG=debug RUST_BACKTRACE=full rust-lldb $DEP1 \
        -o "b {{breakpoint}}" \
        -o "r {{test_name}}"
