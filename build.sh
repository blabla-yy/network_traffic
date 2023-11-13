set -e

cargo build --target aarch64-apple-darwin --release
cargo build --target x86_64-apple-darwin --release

#lipo -create \
#./target/x86_64-apple-darwin/release/libNetworkTraffic.a \
#./target/aarch64-apple-darwin/release/libNetworkTraffic.a \
#-output libNetworkTraffic.a