#![no_std]
#![no_main]

// pick a panicking behavior
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
                     // use panic_abort as _; // requires nightly
                     // use panic_itm as _; // logs messages over ITM; requires ITM support
                     // use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::entry;
// use cortex_m_semihosting::hprintln;
use embedded_hal::digital::v2::InputPin;
use embedded_hal::digital::v2::OutputPin;
use hal::gpio::{Input, Level, Output, Pin, PullUp, PushPull};
use nrf51_hal as hal;
use rtt_target::{rprintln, rtt_init_print};

struct Matrix {
    prev: Button,
    next: Button,
    rows: [Pin<Output<PushPull>>; 3],
    cols: [Pin<Output<PushPull>>; 9],
    coordinates: [[(usize, usize); 5]; 5],
}

impl Matrix {
    fn new(ports: hal::gpio::p0::Parts) -> Matrix {
        return Matrix {
            prev: Button::new(ports.p0_17.into_pullup_input().degrade()),
            next: Button::new(ports.p0_26.into_pullup_input().degrade()),
            rows: [
                ports.p0_13.into_push_pull_output(Level::Low).degrade(),
                ports.p0_14.into_push_pull_output(Level::Low).degrade(),
                ports.p0_15.into_push_pull_output(Level::Low).degrade(),
            ],
            cols: [
                ports.p0_04.into_push_pull_output(Level::Low).degrade(),
                ports.p0_05.into_push_pull_output(Level::Low).degrade(),
                ports.p0_06.into_push_pull_output(Level::Low).degrade(),
                ports.p0_07.into_push_pull_output(Level::Low).degrade(),
                ports.p0_08.into_push_pull_output(Level::Low).degrade(),
                ports.p0_09.into_push_pull_output(Level::Low).degrade(),
                ports.p0_10.into_push_pull_output(Level::Low).degrade(),
                ports.p0_11.into_push_pull_output(Level::Low).degrade(),
                ports.p0_12.into_push_pull_output(Level::Low).degrade(),
            ],
            coordinates: [
                [(0, 0), (1, 3), (0, 1), (1, 4), (0, 2)],
                [(2, 3), (2, 4), (2, 5), (2, 6), (2, 7)],
                [(1, 1), (0, 8), (1, 2), (2, 8), (1, 0)],
                [(0, 7), (0, 6), (0, 5), (0, 4), (0, 3)],
                [(2, 2), (1, 6), (2, 0), (1, 5), (2, 1)],
            ],
        };
    }

    fn on(&mut self, x: usize, y: usize) {
        let (r, c) = self.coordinates[x][y];
        self.rows[r].set_high().unwrap();
        self.cols[c].set_low().unwrap();
    }

    fn off(&mut self, x: usize, y: usize) {
        let (r, c) = self.coordinates[x][y];
        self.rows[r].set_low().unwrap();
        self.cols[c].set_high().unwrap();
    }
}

struct Button {
    pin: Pin<Input<PullUp>>,
    debouncer: u32,
}

impl Button {
    fn new(p: Pin<Input<PullUp>>) -> Button {
        Button {
            pin: p,
            debouncer: 0,
        }
    }

    fn is_pressed(&mut self) -> bool {
        if self.pin.is_low().unwrap() {
            self.debouncer += 1;
            if self.debouncer >= 1250 {
                self.debouncer = 0;
                return true;
            }
        } else {
            self.debouncer = 0;
        }
        return false;
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let p = hal::pac::Peripherals::take().unwrap();
    let port0 = hal::gpio::p0::Parts::new(p.GPIO);

    let mut matrix = Matrix::new(port0);

    rprintln!("Example started");

    let mut row = 0;
    let mut col = 0;
    loop {
        if matrix.prev.is_pressed() {
            if col <= 0 {
                col = 5;
                if row <= 0 {
                    row = 5;
                }
                row -= 1;
            }
            col -= 1;
        }

        if matrix.next.is_pressed() {
            col += 1;
            if col >= 5 {
                col = 0;
                row += 1;
                if row >= 5 {
                    row = 0;
                }
            }
        }

        for i in 0..5 {
            for j in 0..5 {
                if i == row && j == col {
                    matrix.on(i, j);
                } else {
                    matrix.off(i, j);
                }
            }
        }
    }
}
