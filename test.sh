
cargo build --release
cbindgen --output network_traffic.h --lang=c
cc test.c -o test -I./network_traffic.h ./target/release/libnetwork_traffic.dylib
LD_LIBRARY_PATH=./target/release ./test > ~/Desktop/test.txt