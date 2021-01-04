//! Example showing the use of a LoRa breakout board using the RAK811 network driver.
#![no_main]
#![no_std]

use panic_halt as _;

use core::fmt::Write;
use log::LevelFilter;
use rtic::app;
use rtt_logger::RTTLogger;
use rtt_target::rtt_init_print;

use nrf52833_hal as hal;

use hal::gpio::{Input, Level, Output, Pin, PullUp, PushPull};
use hal::pac::{TIMER0, UARTE0};
use hal::timer::Timer;
use hal::uarte::*;

static LOGGER: RTTLogger = RTTLogger::new(LevelFilter::Debug);

#[app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        count: usize,
        uarte: Uarte<UARTE0>,
        uarte_timer: Timer<TIMER0>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        rtt_init_print!();
        log::set_logger(&LOGGER).unwrap();
        log::set_max_level(log::LevelFilter::Debug);

        let port0 = hal::gpio::p0::Parts::new(ctx.device.P0);
        let port1 = hal::gpio::p1::Parts::new(ctx.device.P1);

        let clocks = hal::clocks::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        let _clocks = clocks.start_lfclk();

        let _ = port1.p1_02.into_push_pull_output(Level::High).degrade();
        let uarte = Uarte::new(
            ctx.device.UARTE0,
            Pins {
                txd: port0.p0_01.into_push_pull_output(Level::High).degrade(),
                rxd: port0.p0_13.into_floating_input().degrade(),
                cts: None,
                rts: None,
            },
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );

        log::info!("Started application");

        init::LateResources {
            count: 0,
            uarte,
            uarte_timer: Timer::new(ctx.device.TIMER0),
        }
    }

    #[idle(resources=[uarte, uarte_timer])]
    fn idle(ctx: idle::Context) -> ! {
        let idle::Resources { uarte, uarte_timer } = ctx.resources;

        let result = write!(uarte, "at+get_config=device:status");
        match result {
            Ok(_) => {
                log::info!("Success writing AT command");
            }
            Err(_) => {
                log::info!("Error writing AT command");
            }
        }

        let uarte_rx_buf = &mut [0u8; 64][..];
        loop {
            match uarte.read_timeout(uarte_rx_buf, uarte_timer, 100_000) {
                Ok(_) => {
                    if let Ok(msg) = core::str::from_utf8(&uarte_rx_buf[..]) {
                        log::info!("{}", msg);
                    }
                }
                Err(hal::uarte::Error::Timeout(n)) if n > 0 => {
                    if let Ok(msg) = core::str::from_utf8(&uarte_rx_buf[..n]) {
                        log::info!("{}", msg);
                    }
                }
                _ => {}
            }
        }
    }
};
