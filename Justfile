# Runs tests and supresses the progress bar
test:
    RUNNING_TESTS=true RUST_LOG=debug RUST_BACKTRACE=1 cargo test
