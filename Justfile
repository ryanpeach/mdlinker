# Runs tests and supresses the progress bar
test:
    RUNNING_TESTS=true RUST_LOG=debug RUST_BACKTRACE=1 cargo test --

test-print test_name:
    RUNNING_TESTS=true RUST_LOG=trace RUST_BACKTRACE=1 cargo test -- --test-threads=1 {{test_name}}
