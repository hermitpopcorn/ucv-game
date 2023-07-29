# Build for production
RUSTFLAGS="-C target-feature=+crt-static" cargo build --target x86_64-unknown-linux-gnu --release
cp target/x86_64-unknown-linux-gnu/release/ucv-game ./ucv-game
