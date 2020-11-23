# drogue-microbit

Drogue IoT on the BBC micro:bit. This repository contains crates for micro:bit peripherals as well
as example applications.

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
program target/thumbv6m-none-eabi/release/drogue-microbit-rtc-rtic verify reset
```
