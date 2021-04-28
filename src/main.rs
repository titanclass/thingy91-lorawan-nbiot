#![cfg_attr(not(test), no_std)]
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

#[entry]
fn main() -> ! {
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
