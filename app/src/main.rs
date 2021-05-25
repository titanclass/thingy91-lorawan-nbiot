#![no_std]
#![no_main]

extern crate thingy_91_nrf9160_bsp as bsp;
extern crate tinyrlibc;

use applib::{data_up_unconfirmed, nwk_addr, EnvironmentalPayload};
use bme680::{Bme680, I2CAddress, IIRFilterSize, OversamplingSetting, PowerMode, SettingsBuilder};
use bsp::{
    hal::{pwm, timer, twim, Delay, Twim},
    pac::{interrupt, NVIC},
    prelude::U32Ext,
    Board,
};
use core::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use cortex_m::{asm, prelude::_embedded_hal_timer_CountDown};
use cortex_m_rt::entry;
use embedded_hal::{prelude::_embedded_hal_blocking_delay_DelayMs, Pwm};
use nrfxlib::udp::UdpSocket;

// pick a panicking behavior
#[cfg(debug_assertions)]
use panic_halt as _;

// release profile: minimize the binary size of the application
#[cfg(not(debug_assertions))]
use panic_reset as _;

// TODO: Select a Network ID that your LoRaWAN Network Server accepts connections for
const NET_ID: u32 = 0x13_u32;

// TODO: Replace these network and app session key string literals with ones that your
// LoRaWAN Network Server will recognise. Note that we're using ABP, hence the declaration
// of session keys.

const NWK_SKEY: &'static str = "EE508F76B0492985BFACBACE0B2754C2";
const APP_SKEY: &'static str = "BA357A0A743BD19BD4509B9667C87658";

// TODO: Replace with the ICCID of your SIM card so we can attain something unique
const ICCID: &'static str = "923453256784434561";

// TODO: Replace with how often you would like environmental telemetry to be sent.
const SEND_FREQUENCY_MS: u32 = 60 * 60 * 1000; // 1 hour

// TODO: Replace the host address accordingly.
const NETWORK_SERVER_HOST: &str = "";

// TODO: Replace the host port accordingly.
const NETWORK_SERVER_PORT: u16 = 1694u16;

const TIMER_EXPIRED: AtomicBool = AtomicBool::new(false);

// Interrupt handlers for LTE related hardware. Defers straight to the library.

#[interrupt]
fn EGU1() {
    nrfxlib::application_irq_handler();
    cortex_m::asm::sev();
}

#[interrupt]
fn EGU2() {
    nrfxlib::trace_irq_handler();
    cortex_m::asm::sev();
}

#[interrupt]
fn IPC() {
    nrfxlib::ipc_irq_handler();
    cortex_m::asm::sev();
}

// Our timer handler

#[interrupt]
fn TIMER0() {
    TIMER_EXPIRED.store(true, Ordering::Relaxed);
}

// Setup required for the modem

fn init_modem(board: &mut Board) {
    unsafe {
        NVIC::unmask(bsp::pac::Interrupt::EGU1);
        NVIC::unmask(bsp::pac::Interrupt::EGU2);
        NVIC::unmask(bsp::pac::Interrupt::IPC);

        // Only use top three bits, so shift by up by 8 - 3 = 5 bits

        board.NVIC.set_priority(bsp::pac::Interrupt::EGU2, 4 << 5);
        board.NVIC.set_priority(bsp::pac::Interrupt::EGU1, 4 << 5);
        board.NVIC.set_priority(bsp::pac::Interrupt::IPC, 0 << 5);

        // nRF9160 Engineering A Errata - [17] Debug and Trace: LTE modem stops when debugging through SWD interface
        // https://infocenter.nordicsemi.com/index.jsp?topic=%2Ferrata_nRF9160_EngA%2FERR%2FnRF9160%2FEngineeringA%2Flatest%2Ferr_160.html

        core::ptr::write_volatile(0x4000_5C04 as *mut u32, 0x02);
    }

    nrfxlib::init().unwrap();
}

#[entry]
fn main() -> ! {
    // Initialize device

    let mut board = Board::take().unwrap();

    // Initialise our network connectivity

    init_modem(&mut board);

    let udp_socket = UdpSocket::new().unwrap();
    udp_socket
        .connect(NETWORK_SERVER_HOST, NETWORK_SERVER_PORT)
        .unwrap();

    // Setup LoRaWAN info

    let dev_eui = ICCID.parse::<u64>().unwrap();
    let dev_addr = nwk_addr(dev_eui, NET_ID);

    let nwk_skey = u128::from_str_radix(NWK_SKEY, 16).unwrap();
    let app_skey = u128::from_str_radix(APP_SKEY, 16).unwrap();

    // Setup the environmental sensor

    let scl = board.pins.P0_12.into_floating_input().degrade();
    let sda = board.pins.P0_11.into_floating_input().degrade();

    let pins = twim::Pins { scl, sda };

    let i2c = Twim::new(board.TWIM2_NS, pins, twim::Frequency::K400);

    let mut delayer = Delay::new(board.SYST);

    let mut dev = Bme680::init(i2c, &mut delayer, I2CAddress::Primary).unwrap();
    let settings = SettingsBuilder::new()
        .with_humidity_oversampling(OversamplingSetting::OS2x)
        .with_pressure_oversampling(OversamplingSetting::OS4x)
        .with_temperature_oversampling(OversamplingSetting::OS8x)
        .with_temperature_filter(IIRFilterSize::Size3)
        .with_gas_measurement(Duration::from_millis(1500), 320, 25)
        .with_run_gas(true)
        .build();
    dev.set_sensor_settings(&mut delayer, settings).unwrap();
    dev.set_sensor_mode(&mut delayer, PowerMode::ForcedMode)
        .unwrap();

    // Our main loop where we read our sensors, send data and then sleep

    let mut fcnt = 0; // frame counter for LoRaWAN

    // Set up our LED

    let rgb_pwm = board.leds.rgb_led_1.pwm;
    rgb_pwm.set_period(500u32.hz());
    rgb_pwm.set_duty_on_common(rgb_pwm.get_max_duty());

    // Setup our timer so we can wake up to do our work periodically

    let mut timer = timer::Timer::periodic(board.TIMER0_NS);
    timer.enable_interrupt();
    // timer.start(count); // FIXME

    loop {
        if TIMER_EXPIRED.compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed)
            == Ok(true)
        {
            // Show we're doing something

            rgb_pwm.next_step();
            rgb_pwm.set_duty_on(pwm::Channel::C1, 0);

            // Read  data from the environmental sensor

            let (data, _) = dev.get_sensor_data(&mut delayer).unwrap();

            // Construct a LoRaWAN packet from the data.

            let payload = EnvironmentalPayload {
                temperature: unsafe { (data.temperature_celsius() * 100f32).to_int_unchecked() },
                pressure: unsafe { (data.pressure_hpa() * 100f32).to_int_unchecked() },
                humidity: unsafe { (data.humidity_percent() * 1000f32).to_int_unchecked() },
                gas_resistance: data.gas_resistance_ohm(),
            };

            let payload_bytes = data_up_unconfirmed(dev_addr, fcnt, &payload, nwk_skey, app_skey);

            // Send the data. There's nothing we can do about transmissions failing.
            // Everything is best-effort in IoT.

            let _ = udp_socket.write(&payload_bytes);

            fcnt += 1;

            // All done. Time to sleep.

            rgb_pwm.next_step();
            rgb_pwm.set_duty_on_common(rgb_pwm.get_max_duty());
        }

        asm::wfi();
    }
}
