//! Example showing the use of RTIC in combination with RTC and Led Matrix on the micro:bit.
#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_semihosting;

use drogue_microbit_matrix::LedMatrix;

use hal::rtc::{Rtc, RtcInterrupt};
use rtic::app;

use nrf51_hal as hal;

#[app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        rtc: Rtc<hal::pac::RTC0>,
        led: LedMatrix,
        count: usize,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        let port0 = hal::gpio::p0::Parts::new(ctx.device.GPIO);

        let led = LedMatrix::new(port0);

        let clocks = hal::clocks::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        let _clocks = clocks.start_lfclk();

        let mut rtc = Rtc::new(ctx.device.RTC0, 4095).unwrap();
        rtc.enable_event(RtcInterrupt::Tick);
        rtc.enable_counter();
        rtc.enable_interrupt(RtcInterrupt::Tick, None);

        init::LateResources {
            rtc: rtc,
            led: led,
            count: 0,
        }
    }

    #[task(binds = RTC0, resources = [rtc, led, count])]
    fn rtc0(ctx: rtc0::Context) {
        let rtc: &mut Rtc<hal::pac::RTC0> = ctx.resources.rtc;
        let count: &mut usize = ctx.resources.count;
        let led: &mut LedMatrix = ctx.resources.led;

        rtc.reset_event(RtcInterrupt::Tick);
        rtc.clear_counter();

        let row = (*count / 5) % 5;
        let col = *count % 5;
        led.clear();
        led.on(row, col);
        *count += 1;
    }
};
