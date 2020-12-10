#![no_std]
#![no_main]

use nrf51_hal as hal;
use panic_halt as _;

extern crate cortex_m;
use core::cell::Cell;
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m::peripheral;
use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use hal::gpio::{Level, Output, Pin, PushPull};
use hal::pac::interrupt;
use hal::rtc::{Rtc, RtcInterrupt};
use log::LevelFilter;
use rtt_logger::RTTLogger;
use rtt_target::rtt_init_print;

static COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
static RTC: Mutex<RefCell<Option<Rtc<hal::pac::RTC0>>>> = Mutex::new(RefCell::new(None));
static LED: Mutex<RefCell<Option<Pin<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static LOGGER: RTTLogger = RTTLogger::new(LevelFilter::Debug);

#[entry]
fn main() -> ! {
    rtt_init_print!();

    unsafe {
        log::set_logger_racy(&LOGGER).unwrap();
    }
    log::set_max_level(log::LevelFilter::Debug);
    let mut cp = peripheral::Peripherals::take().unwrap();
    let p = hal::pac::Peripherals::take().unwrap();
    let port0 = hal::gpio::p0::Parts::new(p.GPIO);

    let led = port0.p0_13.into_push_pull_output(Level::Low).degrade();
    let _ = port0.p0_04.into_push_pull_output(Level::Low);

    let clocks = hal::clocks::Clocks::new(p.CLOCK).enable_ext_hfosc();
    let _clocks = clocks.start_lfclk();

    let mut rtc = Rtc::new(p.RTC0, 4095).unwrap();
    rtc.enable_event(RtcInterrupt::Tick);
    rtc.enable_counter();
    rtc.enable_interrupt(RtcInterrupt::Tick, Some(&mut cp.NVIC));
    cortex_m::interrupt::free(|cs| {
        RTC.borrow(cs).replace(Some(rtc));
        LED.borrow(cs).replace(Some(led));
    });

    unsafe {
        cortex_m::interrupt::enable();
    }

    log::info!("Started application");

    loop {}
}

#[interrupt]
fn RTC0() {
    cortex_m::interrupt::free(|cs| {
        let rtc = RTC.borrow(cs).borrow();
        rtc.as_ref().unwrap().reset_event(RtcInterrupt::Tick);
        rtc.as_ref().unwrap().clear_counter();

        let mut led = LED.borrow(cs).borrow_mut();
        if COUNTER.borrow(cs).get() % 2 == 0 {
            led.as_mut().unwrap().set_high().unwrap();
        } else {
            led.as_mut().unwrap().set_low().unwrap();
        }
        COUNTER.borrow(cs).set(COUNTER.borrow(cs).get() + 1)
    });
}
