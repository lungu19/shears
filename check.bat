cls
cargo +nightly check --quiet --workspace --all-targets
cargo +nightly check --quiet --workspace --all-features --lib --target x86_64-pc-windows-msvc
cargo +nightly fmt --all -- --check
cargo +nightly clippy --quiet --workspace --all-targets --all-features --  -D warnings -W clippy::all