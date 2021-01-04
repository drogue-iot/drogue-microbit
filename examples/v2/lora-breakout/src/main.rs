//! Example showing the use of RTIC in combination with RTC and Led Matrix on the micro:bit.
#![no_main]
#![no_std]

use panic_halt as _;

use log::LevelFilter;
use rtic::app;
use rtt_logger::RTTLogger;
use rtt_target::rtt_init_print;

use nrf52833_hal as hal;

static LOGGER: RTTLogger = RTTLogger::new(LevelFilter::Debug);

#[app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        count: usize,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        rtt_init_print!();
        log::set_logger(&LOGGER).unwrap();
        log::set_max_level(log::LevelFilter::Debug);

        let port0 = hal::gpio::p0::Parts::new(ctx.device.P0);

        let clocks = hal::clocks::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        let _clocks = clocks.start_lfclk();

        log::info!("Started application");

        init::LateResources { count: 0 }
    }
};
