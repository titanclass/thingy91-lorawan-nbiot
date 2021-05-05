#![no_std]
#![no_main]

use bsp::hal::{twim, Delay, Twim};

use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
// pick a panicking behavior
#[cfg(debug_assertions)]
use panic_halt as _;

// release profile: minimize the binary size of the application
#[cfg(not(debug_assertions))]
use panic_reset as _;

use cortex_m_rt::entry;

use bme680::*;

extern crate nrf9160_dk_bsp as bsp;

use core::time::Duration;

use applib::*;

// FIXME: Select a Network ID that your LoRaWAN Network Server accepts connections for
const NET_ID: u32 = 0x13_u32;

// FIXME: Replace these network and app session key string literals with ones that your
// LoRaWAN Network Server will recognise. Note that we're using ABP, hence the declaration
// of session keys.

const NWK_SKEY: &'static str = "EE508F76B0492985BFACBACE0B2754C2";
const APP_SKEY: &'static str = "BA357A0A743BD19BD4509B9667C87658";

// FIXME: Replace with the ICCID of your SIM card so we can attain something unique
const ICCID: &'static str = "923453256784434561";

// FIXME: Replace with how often you would like environmental telemetry to be sent.
const SEND_FREQUENCY_MS: u32 = 60 * 60 * 1000; // 1 hour

#[entry]
fn main() -> ! {
    // Initialize device

    let mut board = bsp::Board::take().unwrap();

    *GLOBAL_UART.lock() = Some(board.cdc_uart);

    // Setup LoRaWAN info

    let dev_eui = ICCID.parse::<u64>().unwrap();
    let dev_addr = nwk_addr(dev_eui, NET_ID);

    let nwk_skey = u128::from_str_radix(NWK_SKEY, 16).unwrap();
    let app_skey = u128::from_str_radix(APP_SKEY, 16).unwrap();

    // Setup the environmental sensor

    let scl = board.pins.P0_12.degrade();
    let sda = board.pins.P0_11.degrade();

    let pins = twim::Pins { scl, sda };

    let i2c = Twim::new(board.TWIM0_NS, pins, twim::Frequency::K100);

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

    loop {
        // Show we're doing something

        board.leds.led_1.enable();

        // Read  data from the environmental sensor

        let (data, _) = dev.get_sensor_data(&mut delayer).unwrap();
        println!("Temperature {}°C", data.temperature_celsius());
        println!("Pressure {}hPa", data.pressure_hpa());
        println!("Humidity {}%", data.humidity_percent());
        println!("Gas Resistence {}Ω", data.gas_resistance_ohm());

        // Construct a LoRaWAN packet from the data.

        let payload = EnvironmentalPayload {
            temperature: unsafe { (data.temperature_celsius() * 100f32).to_int_unchecked() },
            pressure: unsafe { (data.pressure_hpa() * 100f32).to_int_unchecked() },
            humidity: unsafe { (data.humidity_percent() * 1000f32).to_int_unchecked() },
            gas_resistance: data.gas_resistance_ohm(),
        };

        let _payload_bytes = data_up_unconfirmed(dev_addr, fcnt, &payload, nwk_skey, app_skey);

        // FIXME: Send the data

        fcnt += 1;

        // All done. Time to sleep.

        board.leds.led_1.disable();

        delayer.delay_ms(SEND_FREQUENCY_MS); // We can do better by using a periodic timer as it'll take a few seconds to the above
    }
}

/// A UART we can access from anywhere (with run-time lock checking).
static GLOBAL_UART: spin::Mutex<Option<bsp::hal::uarte::Uarte<bsp::pac::UARTE0_NS>>> =
    spin::Mutex::new(None);

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => {
        {
            use core::fmt::Write as _;
            if let Some(ref mut uart) = *crate::GLOBAL_UART.lock() {
                let _err = writeln!(*uart, $($arg)*);
            }
        }
    };
}
