//! Example of a BLE thermometer exposed using ESS (Environmental Sensing Service)
#![no_main]
#![no_std]

#[allow(unused_imports)]
use panic_halt;

use drogue_microbit_ess::{EnvironmentSensingService, ESS_UUID};

use nrf51_hal as hal;

use core::sync::atomic::{compiler_fence, Ordering};
use hal::rtc::{Rtc, RtcCompareReg, RtcInterrupt};
use log::LevelFilter;
use rtt_logger::RTTLogger;
use rtt_target::rtt_init_print;

use rubble::l2cap::{BleChannelMap, L2CAPState};
use rubble::link::queue::{PacketQueue, SimpleQueue};
use rubble::link::{
    ad_structure::{AdStructure, ServiceUuids},
    LinkLayer, Responder, MIN_PDU_BUF,
};
use rubble::time::{Duration, Timer};
use rubble::{config::Config, security::NoSecurity};
use rubble_nrf5x::radio::{BleRadio, PacketBuffer};
use rubble_nrf5x::{timer::BleTimer, utils::get_device_address};

static LOGGER: RTTLogger = RTTLogger::new(LevelFilter::Debug);

use rtic::app;

pub enum AppConfig {}

impl Config for AppConfig {
    type Timer = BleTimer<hal::pac::TIMER0>;
    type Transmitter = BleRadio;
    type ChannelMapper = BleChannelMap<EnvironmentSensingService, NoSecurity>;
    type PacketQueue = &'static mut SimpleQueue;
}

#[app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        // Temperature sensing
        thermometer: hal::Temp,
        rtc: Rtc<hal::pac::RTC0>,
        #[init(0)]
        timer_count: i8,

        #[init([0; MIN_PDU_BUF])]
        ble_tx_buf: PacketBuffer,
        #[init([0; MIN_PDU_BUF])]
        ble_rx_buf: PacketBuffer,
        #[init(SimpleQueue::new())]
        tx_queue: SimpleQueue,
        #[init(SimpleQueue::new())]
        rx_queue: SimpleQueue,
        ble_ll: LinkLayer<AppConfig>,
        ble_r: Responder<AppConfig>,
        radio: BleRadio,
    }

    #[init(resources = [ble_tx_buf, ble_rx_buf, tx_queue, rx_queue])]
    fn init(ctx: init::Context) -> init::LateResources {
        rtt_init_print!();
        log::set_max_level(log::LevelFilter::Debug);
        unsafe {
            log::set_logger_racy(&LOGGER).unwrap();
        }

        let _port0 = hal::gpio::p0::Parts::new(ctx.device.GPIO);

        let clocks = hal::clocks::Clocks::new(ctx.device.CLOCK).enable_ext_hfosc();
        let _clocks = clocks.start_lfclk();

        let thermometer = hal::Temp::new(ctx.device.TEMP);

        let ble_timer = BleTimer::init(ctx.device.TIMER0);

        let mut rtc = Rtc::new(ctx.device.RTC0, 4095).unwrap();
        rtc.enable_event(RtcInterrupt::Compare0);
        rtc.enable_counter();
        let _ = rtc.set_compare(RtcCompareReg::Compare0, 10);
        rtc.enable_interrupt(RtcInterrupt::Compare0, None);

        let device_address = get_device_address();

        log::info!("Starting to advertise with address {:?}", device_address);

        let mut radio = BleRadio::new(
            ctx.device.RADIO,
            &ctx.device.FICR,
            ctx.resources.ble_tx_buf,
            ctx.resources.ble_rx_buf,
        );

        // Create TX/RX queues
        let (tx, tx_cons) = ctx.resources.tx_queue.split();
        let (rx_prod, rx) = ctx.resources.rx_queue.split();

        // Create the actual BLE stack objects
        let mut ble_ll = LinkLayer::<AppConfig>::new(device_address, ble_timer);

        let ess: EnvironmentSensingService = EnvironmentSensingService::new();

        let ble_r = Responder::new(tx, rx, L2CAPState::new(BleChannelMap::with_attributes(ess)));

        let next_update = ble_ll
            .start_advertise(
                Duration::from_millis(100),
                &[
                    AdStructure::CompleteLocalName("Drogue IoT micro:bit"),
                    AdStructure::ServiceUuids16(ServiceUuids::from_uuids(true, &[ESS_UUID])),
                ],
                &mut radio,
                tx_cons,
                rx_prod,
            )
            .unwrap();

        ble_ll.timer().configure_interrupt(next_update);

        init::LateResources {
            radio: radio,
            ble_ll: ble_ll,
            ble_r: ble_r,
            thermometer: thermometer,
            rtc: rtc,
        }
    }

    #[task(binds = RADIO, resources = [radio, ble_ll], spawn = [ble_worker], priority = 3)]
    fn radio(ctx: radio::Context) {
        let ble_ll: &mut LinkLayer<AppConfig> = ctx.resources.ble_ll;
        if let Some(cmd) = ctx
            .resources
            .radio
            .recv_interrupt(ble_ll.timer().now(), ble_ll)
        {
            ctx.resources.radio.configure_receiver(cmd.radio);
            ble_ll.timer().configure_interrupt(cmd.next_update);

            if cmd.queued_work {
                // If there's any lower-priority work to be done, ensure that happens.
                // If we fail to spawn the task, it's already scheduled.
                ctx.spawn.ble_worker().ok();
            }
        }
    }

    #[task(binds = TIMER0, resources = [radio, ble_ll], spawn = [ble_worker], priority = 3)]
    fn timer0(ctx: timer0::Context) {
        let timer = ctx.resources.ble_ll.timer();
        if !timer.is_interrupt_pending() {
            return;
        }
        timer.clear_interrupt();

        let cmd = ctx.resources.ble_ll.update_timer(ctx.resources.radio);
        ctx.resources.radio.configure_receiver(cmd.radio);

        ctx.resources
            .ble_ll
            .timer()
            .configure_interrupt(cmd.next_update);

        if cmd.queued_work {
            // If there's any lower-priority work to be done, ensure that happens.
            // If we fail to spawn the task, it's already scheduled.
            ctx.spawn.ble_worker().ok();
        }
    }

    #[task(binds = RTC0, resources = [rtc, thermometer, timer_count, ble_r], priority = 1)]
    fn rtc0(ctx: rtc0::Context) {
        let rtc0::Resources {
            rtc,
            thermometer,
            timer_count,
            mut ble_r,
        } = ctx.resources;
        rtc.reset_event(RtcInterrupt::Compare0);
        rtc.clear_counter();
        if *timer_count % 2 == 0 {
            thermometer.start_measurement();
        } else {
            let value = thermometer.read();
            value.map_or_else(
                |_| {},
                |value| {
                    let f = value.to_num::<u32>() - 4;
                    ble_r.lock(|ble_r| {
                        let l2cap = &mut *(ble_r.l2cap());
                        let provider: &mut EnvironmentSensingService =
                            l2cap.channel_mapper().attribute_provider();
                        provider.set_temperature(f);
                    });
                },
            );
            thermometer.stop_measurement();
        }
        *timer_count += 1;
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        log::info!("Drogue IoT micro:bit started!");
        loop {
            compiler_fence(Ordering::SeqCst);
        }
    }

    #[task(resources = [ble_r], priority = 2)]
    fn ble_worker(ctx: ble_worker::Context) {
        let ble_worker::Resources { ble_r } = ctx.resources;

        // Fully drain the packet queue
        while ble_r.has_work() {
            ble_r.process_one().unwrap();
        }
    }

    extern "C" {
        fn WDT();
    }
};
