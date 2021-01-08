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

use core::pin::Pin;
use drogue_rak811 as rak811;
use hal::gpio::Level;
use hal::pac::{TIMER0, UARTE0};
use hal::timer::Timer;
use hal::uarte::*;

static LOGGER: RTTLogger = RTTLogger::new(LevelFilter::Trace);

#[app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        count: usize,
        driver: rak811::Rak811Driver<UarteTx<UARTE0>, UarteRx<UARTE0>>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        rtt_init_print!();
        log::set_logger(&LOGGER).unwrap();
        log::set_max_level(log::LevelFilter::Trace);

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

        let (uarte_tx, uarte_rx) = uarte.split();

        let driver = rak811::Rak811Driver::new(uarte_tx, uarte_rx);

        log::info!("Started application");

        init::LateResources { count: 0, driver }
    }

    #[idle(resources=[driver])]
    fn idle(ctx: idle::Context) -> ! {
        let idle::Resources { driver } = ctx.resources;

        driver
            .send_command(rak811::Command::QueryFirmwareInfo)
            .map_err(|e| log::error!("ERROR: {:?}", e));
        driver
            .send_command(rak811::Command::GetBand)
            .map_err(|e| log::error!("ERROR: {:?}", e));

        driver
            .set_mode(rak811::LoraMode::WAN)
            .map_err(|e| log::error!("ERROR: {:?}", e));

        driver
            .set_device_address(&[0x00, 0x11, 0x22, 0x33])
            .map_err(|e| log::error!("ERROR: {:?}", e));

        driver
            .set_device_eui(&[0x00, 0xBB, 0x7C, 0x95, 0xAD, 0xB5, 0x30, 0xB9])
            .map_err(|e| log::error!("ERROR: {:?}", e));
        driver
            .set_app_eui(&[0x70, 0xB3, 0xD5, 0x7E, 0xD0, 0x03, 0xB1, 0x84])
            .map_err(|e| log::error!("ERROR: {:?}", e));

        driver
            .set_app_key(&[
                0xE2, 0xB5, 0x25, 0xB6, 0x86, 0xB8, 0xE2, 0xE6, 0xFE, 0x22, 0x27, 0x51, 0xAF, 0x35,
                0xCD, 0x22,
            ])
            .map_err(|e| log::error!("ERROR: {:?}", e));

        driver
            .join(rak811::ConnectMode::OTAA)
            .map_err(|e| log::error!("ERROR: {:?}", e));

        driver
            .send(rak811::QoS::Unconfirmed, 1, b"hello, world")
            .map_err(|e| log::error!("ERROR: {:?}", e));

        log::info!("Data sent!");

        loop {
            compiler_fence(Ordering::SeqCst);
        }
    }
};
