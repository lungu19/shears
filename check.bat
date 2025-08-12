cargo check --quiet --workspace --all-targets
cargo check --quiet --workspace --all-features --lib --target x86_64-pc-windows-msvc
cargo fmt --all -- --check
cargo clippy --quiet --workspace --all-targets --all-features --  -D warnings -W clippy::all