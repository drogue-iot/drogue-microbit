all:
	cargo build --release
	objcopy target/thumbv6m-none-eabi/release/drogue-microbit -O ihex microbit.hex

clean:
	cargo clean
