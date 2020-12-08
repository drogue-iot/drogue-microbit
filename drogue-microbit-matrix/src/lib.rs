#![no_std, no_main]

use embedded_hal::digital::v2::OutputPin;
use hal::gpio::{Level, Output, Pin, PushPull};

#[cfg(feature = "51")]
use nrf51_hal as hal;

#[cfg(feature = "52833")]
use nrf52833_hal as hal;

pub struct LedMatrix {
    rows: [Pin<Output<PushPull>>; 3],
    cols: [Pin<Output<PushPull>>; 9],
    coordinates: [[(usize, usize); 5]; 5],
}

impl LedMatrix {
    pub fn new(ports: hal::gpio::p0::Parts) -> LedMatrix {
        let mut m = LedMatrix {
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
        m.clear();
        m
    }

    pub fn clear(&mut self) {
        for row in self.rows.iter_mut() {
            row.set_low().unwrap();
        }
        for col in self.cols.iter_mut() {
            col.set_high().unwrap();
        }
    }

    pub fn on(&mut self, x: usize, y: usize) {
        let (r, c) = self.coordinates[x][y];
        self.rows[r].set_high().unwrap();
        self.cols[c].set_low().unwrap();
    }

    pub fn off(&mut self, x: usize, y: usize) {
        let (r, c) = self.coordinates[x][y];
        self.rows[r].set_low().unwrap();
        self.cols[c].set_high().unwrap();
    }
}

mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
