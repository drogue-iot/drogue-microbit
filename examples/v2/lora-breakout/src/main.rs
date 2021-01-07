//! Example showing the use of a LoRa breakout board using the RAK811 network driver.
#![no_main]
#![no_std]

use panic_halt as _;

use core::sync::atomic::{compiler_fence, Ordering};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::serial::{Read, Write};
use log::LevelFilter;
use rtic::app;
use rtt_logger::RTTLogger;
use rtt_target::rtt_init_print;

use nrf52833_hal as hal;

use drogue_rak811 as rak811;
use hal::gpio::Level;
use hal::pac::{TIMER0, UARTE0};
use hal::timer::Timer;
use hal::uarte::*;

static LOGGER: RTTLogger = RTTLogger::new(LevelFilter::Debug);

#[app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        count: usize,
        #[init([0; 1])]
        tx_buf: [u8; 1],
        #[init([0; 1])]
        rx_buf: [u8; 1],
        driver: rak811::Rak811Driver<UarteTx<'static, UARTE0>, UarteRx<'static, UARTE0>>,
    }

    #[init(resources = [tx_buf, rx_buf])]
    fn init(ctx: init::Context) -> init::LateResources {
        rtt_init_print!();
        log::set_logger(&LOGGER).unwrap();
        log::set_max_level(log::LevelFilter::Debug);

        let port0 = hal::gpio::p0::Parts::new(ctx.device.P0);
        let port1 = hal::gpio::p1::Parts::new(ctx.device.P1);

        let clocks = hal::clocks::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        let _clocks = clocks.start_lfclk();

        let mut rst = port1.p1_02.into_push_pull_output(Level::High).degrade();
        let mut cnt: u64 = 0;
        while cnt < 5000000 {
            cnt += 1;
            compiler_fence(Ordering::SeqCst);
        }
        let _ = rst.set_low();
        cnt = 0;
        while cnt < 5000000 {
            cnt += 1;
            compiler_fence(Ordering::SeqCst);
        }

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

        let (uarte_tx, uarte_rx) = uarte
            .split(ctx.resources.tx_buf, ctx.resources.rx_buf)
            .unwrap();

        let driver = rak811::Rak811Driver::new(uarte_tx, uarte_rx);

        log::info!("Started application");

        init::LateResources { count: 0, driver }
    }

    #[idle(resources=[driver])]
    fn idle(ctx: idle::Context) -> ! {
        let idle::Resources { driver } = ctx.resources;

        send_command(driver, rak811::Command::QueryFirmwareInfo);
        send_command(driver, rak811::Command::GetBand);

        loop {
            compiler_fence(Ordering::SeqCst);
        }
    }
};

fn send_command(
    driver: &mut rak811::Rak811Driver<UarteTx<'static, UARTE0>, UarteRx<'static, UARTE0>>,
    command: rak811::Command,
) {
    log::info!("Sending command: {:?}", command);
    match driver.send(command) {
        Ok(response) => log::info!("Response: {:?}", response),
        Err(e) => {
            log::info!("Command error: {:?}", e);
        }
    }
}
