all:  
	cargo build --example rf --release --target=armv7-unknown-linux-gnueabihf
	cargo build --example rf --release --target=i686-unknown-linux-gnu

arm:
	cargo build --example rf --release --target=armv7-unknown-linux-gnueabihf

x86:
	cargo build --example rf --release --target=i686-unknown-linux-gnu
