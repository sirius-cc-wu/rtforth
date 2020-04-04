all:  
	cargo build --example rf --release --target=armv7-unknown-linux-gnueabihf
	cargo build --example rf --release --target=i686-unknown-linux-gnu

arm:
	cargo build --example rf --release --target=armv7-unknown-linux-gnueabihf

x86:
	cargo build --example rf --release --target=i686-unknown-linux-gnu

test-x86:
	cargo test --target=i686-unknown-linux-gnu

bench-x86:
	cargo bench --target=i686-unknown-linux-gnu

x86-64:
	cargo build --example rf --release --target=x86_64-unknown-linux-gnu

test-x86-64:
	cargo test --target=x86_64-unknown-linux-gnu

bench-x86-64:
	cargo bench --target=x86_64-unknown-linux-gnu
