# This scripts runs various CI-like checks in a convenient way.
set -eux

cargo check --quiet --all-targets
cargo fmt --all -- --check
cargo clippy --quiet --all-targets --all-features --  -D warnings -W clippy::all
cargo build
