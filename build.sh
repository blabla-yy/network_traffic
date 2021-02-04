set -e

cargo build --target aarch64-apple-darwin --release
cargo build --target x86_64-apple-darwin --release

lipo -create \
./target/x86_64-apple-darwin/release/libNetworkTraffic.a \
./target/aarch64-apple-darwin/release/libNetworkTraffic.a \
-output libNetworkTraffic.a

echo "codesign"
# 循环签名
codesign -f -s "Apple Development: 563335734@qq.com (Q5E96A7VL4)" libNetworkTraffic.a