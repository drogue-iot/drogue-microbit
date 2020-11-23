# drogue-microbit

Drogue IoT on the BBC micro:bit. This repository contains crates for micro:bit peripherals as well
as example applications.

## Examples

* `examples/rtc-rtic` - example of how to use the LED matrix and real time counter with [RTIC](https://rtic.rs)
* `examples/rtc-baremetal` - example of how to use the real time counter using "bare metal" (only cortex-m crate) and setting up interrupt handlers.

## Drivers

* `drogue-microbit-matrix` - driver for working with the LED matrix on the micro:bit

# Build

```
cargo build --release
```

# Program

In a separate terminal, run:

```
openocd
```

To program:

```
telnet localhost 4444

# Enter the following in the openocd CLI:
program target/thumbv6m-none-eabi/release/rtc-rtic verify reset
```
