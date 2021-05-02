#![no_std]
#![no_main]

use bsp::hal::{twim, Delay, Twim};

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

// The payload to convey over LoRaWAN
#[repr(C)]
struct LoRaWANPayload {
    temperature: i16,
    pressure: u32,
    humidity: u32,
    gas_resistance: u32,
}

impl LoRaWANPayload {
    fn as_bytes(&self) -> &[u8] {
        // This is actually safe given the underlying fields we're slicing
        unsafe {
            core::slice::from_raw_parts(
                &self as *const _ as *const u8,
                core::mem::size_of::<LoRaWANPayload>(),
            )
        }
    }
}

// FIXME: Select a Network ID that your LoRaWAN Network Server accepts connections for
const NET_ID: u32 = 0x13_u32;

// FIXME: Replace these network and app session key string literals with ones that your
// LoRaWAN Network Server will recognise. Note that we're using ABP, hence the declaration
// of session keys.

const NWK_SKEY: &'static str = "EE508F76B0492985BFACBACE0B2754C2";
const APP_SKEY: &'static str = "BA357A0A743BD19BD4509B9667C87658";

#[entry]
fn main() -> ! {
    // FIXME: Take care of the unwrap() calls

    // Initialize device

    let board = bsp::Board::take().unwrap();

    let scl = board.pins.P0_12.degrade();
    let sda = board.pins.P0_11.degrade();

    let pins = twim::Pins { scl, sda };

    let i2c = Twim::new(board.TWIM0_NS, pins, twim::Frequency::K400);

    let delayer = Delay::new(board.SYST);

    let mut dev = Bme680::init(i2c, delayer, I2CAddress::Primary).unwrap();
    let settings = SettingsBuilder::new()
        .with_humidity_oversampling(OversamplingSetting::OS2x)
        .with_pressure_oversampling(OversamplingSetting::OS4x)
        .with_temperature_oversampling(OversamplingSetting::OS8x)
        .with_temperature_filter(IIRFilterSize::Size3)
        .with_gas_measurement(Duration::from_millis(1500), 320, 25)
        .with_run_gas(true)
        .build();
    dev.set_sensor_settings(settings).unwrap();

    // Read sensor data
    dev.set_sensor_mode(PowerMode::ForcedMode).unwrap();
    let (data, _state) = dev.get_sensor_data().unwrap();

    println!("Temperature {}°C", data.temperature_celsius());
    println!("Pressure {}hPa", data.pressure_hpa());
    println!("Humidity {}%", data.humidity_percent());
    println!("Gas Resistence {}Ω", data.gas_resistance_ohm());

    // Construct a LoRaWAN packet from the data. We'll lay the packet out
    // as follows, and using an FPort of 1:
    //
    // Start |   End | Description
    //     0 |     1 | Temperature (C) * 100
    //     2 |     5 | Pressure (hPA) * 100
    //     6 |     9 | Humidity (%) * 1000
    //    10 |    13 | Gas Resistence

    let mut phy = lorawan_encoding::creator::DataPayloadCreator::new();
    let dev_eui = "923453256784434561".parse::<u64>().unwrap(); // FIXME: use ICCID returned via modem - "AT+ICCID" ???
    let dev_addr = nwk_addr(dev_eui, NET_ID);

    let payload = LoRaWANPayload {
        temperature: unsafe { (data.temperature_celsius() * 100f32).to_int_unchecked() },
        pressure: unsafe { (data.pressure_hpa() * 100f32).to_int_unchecked() },
        humidity: unsafe { (data.humidity_percent() * 1000f32).to_int_unchecked() },
        gas_resistance: data.gas_resistance_ohm(),
    };

    let nwk_skey =
        lorawan_encoding::keys::AES128(u128::from_str_radix(NWK_SKEY, 16).unwrap().to_le_bytes());
    let app_skey =
        lorawan_encoding::keys::AES128(u128::from_str_radix(APP_SKEY, 16).unwrap().to_le_bytes());

    phy.set_confirmed(false)
        .set_uplink(true)
        .set_f_port(1)
        .set_dev_addr(&dev_addr.to_le_bytes())
        .set_fcnt(0); // FIXME: Update the fcnt
    let _payload_bytes = phy
        .build(payload.as_bytes(), &[], &nwk_skey, &app_skey)
        .unwrap();

    loop {}
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
