use bsp::hal::{uarte::Instance, Timer, Uarte};
use core::fmt::Write;
use menu::{Item, ItemType, Menu, Parameter, Runner};
use nrf9160_hal::pac::NVMC_NS;
use nrf_hal_common::{nvmc::Nvmc, pac::TIMER0_NS};
use thingy_91_nrf9160_bsp::hal::uarte;

use crate::config::Config;

pub struct Console<'a, T>
where
    T: Instance,
{
    config: &'a mut Config,
    nvmc: &'a mut Nvmc<NVMC_NS>,
    timer: &'a mut Timer<TIMER0_NS>,
    uarte: &'a mut Uarte<T>,
}

impl<'a, T> Console<'a, T>
where
    T: Instance,
{
    pub fn with(
        config: &'a mut Config,
        nvmc: &'a mut Nvmc<NVMC_NS>,
        timer: &'a mut Timer<TIMER0_NS>,
        uarte: &'a mut Uarte<T>,
    ) -> Self {
        Console {
            config,
            nvmc,
            timer,
            uarte,
        }
    }
}

impl<'a, T> Write for Console<'a, T>
where
    T: Instance,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.uarte.write_str(s)
    }
}

fn set_net_id<'a, T>(
    _menu: &Menu<Console<'a, T>>,
    _item: &Item<Console<'a, T>>,
    args: &[&str],
    context: &mut Console<'a, T>,
) where
    T: Instance,
{
    match u32::from_str_radix(args[0].trim_start_matches("0x"), 16) {
        Ok(v) => context.config.net_id = v,
        Err(_) => writeln!(context, "Invalid").unwrap(),
    };
}

fn set_nwk_skey<'a, T>(
    _menu: &Menu<Console<'a, T>>,
    _item: &Item<Console<'a, T>>,
    args: &[&str],
    context: &mut Console<'a, T>,
) where
    T: Instance,
{
    match (
        u128::from_str_radix(args[0].trim_start_matches("0x"), 16),
        args[0].len() == 32,
    ) {
        (Ok(v), true) => context.config.nwkskey = Some(v),
        _ => writeln!(context, "Invalid").unwrap(),
    };
}

fn set_app_skey<'a, T>(
    _menu: &Menu<Console<'a, T>>,
    _item: &Item<Console<'a, T>>,
    args: &[&str],
    context: &mut Console<'a, T>,
) where
    T: Instance,
{
    match (
        u128::from_str_radix(args[0].trim_start_matches("0x"), 16),
        args[0].len() == 32,
    ) {
        (Ok(v), true) => context.config.appskey = Some(v),
        _ => writeln!(context, "Invalid").unwrap(),
    };
}

fn set_iccid<'a, T>(
    _menu: &Menu<Console<'a, T>>,
    _item: &Item<Console<'a, T>>,
    args: &[&str],
    context: &mut Console<'a, T>,
) where
    T: Instance,
{
    match args[0].parse::<u64>() {
        Ok(v) => context.config.iccid = Some(v),
        Err(_) => writeln!(context, "Invalid").unwrap(),
    };
}

fn set_send_freq_ms<'a, T>(
    _menu: &Menu<Console<'a, T>>,
    _item: &Item<Console<'a, T>>,
    args: &[&str],
    context: &mut Console<'a, T>,
) where
    T: Instance,
{
    match args[0].parse::<u32>() {
        Ok(v) => context.config.send_frequency_ms = v,
        Err(_) => writeln!(context, "Invalid").unwrap(),
    };
}

fn set_network_server_host<'a, T>(
    _menu: &Menu<Console<'a, T>>,
    _item: &Item<Console<'a, T>>,
    args: &[&str],
    context: &mut Console<'a, T>,
) where
    T: Instance,
{
    let mut iter = args[0].split('.');
    let mut result = [None; 4];
    for r in &mut result {
        if let Some(v) = iter.next() {
            if let Ok(v) = v.parse::<u8>() {
                *r = Some(v);
            }
        } else {
            break;
        }
    }
    match result {
        [Some(b0), Some(b1), Some(b2), Some(b3)] => {
            context.config.network_server_host = Some([b0, b1, b2, b3]);
        }
        _ => writeln!(context, "Invalid").unwrap(),
    }
}

fn set_network_server_port<'a, T>(
    _menu: &Menu<Console<'a, T>>,
    _item: &Item<Console<'a, T>>,
    args: &[&str],
    context: &mut Console<'a, T>,
) where
    T: Instance,
{
    match args[0].parse::<u16>() {
        Ok(v) => context.config.network_server_port = v,
        Err(_) => writeln!(context, "Invalid").unwrap(),
    };
}

fn save<'a, T>(
    _menu: &Menu<Console<'a, T>>,
    _item: &Item<Console<'a, T>>,
    _args: &[&str],
    context: &mut Console<'a, T>,
) where
    T: Instance,
{
    write!(context, "Saving to flash... ").unwrap();
    match context.config.save(&mut context.nvmc) {
        Ok(_) => writeln!(context, "saved.").unwrap(),
        Err(_) => writeln!(context, "there was a problem saving.").unwrap(),
    };
}

fn show<'a, T>(
    _menu: &Menu<Console<'a, T>>,
    _item: &Item<Console<'a, T>>,
    _args: &[&str],
    context: &mut Console<'a, T>,
) where
    T: Instance,
{
    let config = context.config.clone();
    writeln!(context, "Settings...\n").unwrap();
    writeln!(context, "NET_ID:\t\t\t 0x{:08X}", config.net_id).unwrap();
    if let Some(nwkskey) = config.nwkskey {
        writeln!(context, "NWKSKEY:\t\t 0x{:032X}", nwkskey).unwrap();
    } else {
        writeln!(context, "NWKSKEY:\t\t REQUIRED!").unwrap();
    }
    if let Some(appskey) = config.appskey {
        writeln!(context, "APPSKEY:\t\t 0x{:032X}", appskey).unwrap();
    } else {
        writeln!(context, "APPSKEY:\t\t REQUIRED!").unwrap();
    }
    if let Some(iccid) = config.iccid {
        writeln!(context, "ICCID:\t\t\t {}", iccid).unwrap();
    } else {
        writeln!(context, "ICCID:\t\t\t REQUIRED!").unwrap();
    }
    writeln!(context, "SEND_FREQUENCY_MS:\t {}", config.send_frequency_ms).unwrap();
    if let Some(network_server_host) = config.network_server_host {
        writeln!(
            context,
            "NETWORK_SERVER_HOST:\t {}.{}.{}.{}",
            network_server_host[0],
            network_server_host[1],
            network_server_host[2],
            network_server_host[3]
        )
        .unwrap();
    } else {
        writeln!(context, "NETWORK_SERVER_HOST:\t REQUIRED!").unwrap();
    }
    writeln!(
        context,
        "NETWORK_SERVER_PORT:\t {}",
        config.network_server_port
    )
    .unwrap();
}

pub fn enter<T>(console: Console<T>)
where
    T: Instance,
{
    let menu = Menu {
        label: "root",
        items: &[
            &Item {
                item_type: ItemType::Callback {
                    function: set_net_id,
                    parameters: &[Parameter::Optional {
                        parameter_name: "NET_ID",
                        help: Some(
                            "The Network ID in hex form. Defaults to \"0x13\" for The Things Network",
                        ),
                    }],
                },
                command: "set-net-id",
                help: Some("Sets a LoRaWAN Network ID"),
            },
            &Item {
                item_type: ItemType::Callback {
                    function: set_nwk_skey,
                    parameters: &[Parameter::Mandatory {
                        parameter_name: "NWKSKEY",
                        help: Some("e.g. EE508F76B0492985BFACBACE0B2754C2"),
                    }],
                },
                command: "set-nwkskey",
                help: Some("Sets a LoRaWAN Network Session Key in hex form"),
            },
            &Item {
                item_type: ItemType::Callback {
                    function: set_app_skey,
                    parameters: &[Parameter::Mandatory {
                        parameter_name: "APPSKEY",
                        help: Some("e.g. BA357A0A743BD19BD4509B9667C87658"),
                    }],
                },
                command: "set-appskey",
                help: Some("Sets a LoRaWAN Application Session Key in hex form"),
            },
            &Item {
                item_type: ItemType::Callback {
                    function: set_iccid,
                    parameters: &[Parameter::Mandatory {
                        parameter_name: "ICCID",
                        help: Some("e.g. 923453256784434561 i.e. without the country code!"),
                    }],
                },
                command: "set-iccid",
                help: Some("Sets the device's ICCID"),
            },
            &Item {
                item_type: ItemType::Callback {
                    function: set_send_freq_ms,
                    parameters: &[Parameter::Optional {
                        parameter_name: "SEND_FREQUENCY_MS",
                        help: Some("Defaults to 60 * 60 * 1000 (1 hour)"),
                    }],
                },
                command: "set-send-freq",
                help: Some("Sets the data transmission frequency to flash. Defaults to 3600000ms."),
            },
            &Item {
                item_type: ItemType::Callback {
                    function: set_network_server_host,
                    parameters: &[Parameter::Mandatory {
                        parameter_name: "NETWORK_SERVER_HOST",
                        help: Some("The IP V4 address of the host"),
                    }],
                },
                command: "set-network-host",
                help: Some("Sets the network server host."),
            },
            &Item {
                item_type: ItemType::Callback {
                    function: set_network_server_port,
                    parameters: &[Parameter::Optional {
                        parameter_name: "NETWORK_SERVER_PORT",
                        help: Some("The IP port of the host. Defaults to 1694."),
                    }],
                },
                command: "set-network-port",
                help: Some("Sets the network server port."),
            },
            &Item {
                item_type: ItemType::Callback {
                    function: save,
                    parameters: &[],
                },
                command: "save",
                help: Some("Saves settings to flash."),
            },
            &Item {
                item_type: ItemType::Callback {
                    function: show,
                    parameters: &[],
                },
                command: "show",
                help: Some("Shows settings."),
            },
        ],
        entry: None,
        exit: None,
    };

    let mut buffer = [0u8; 64];
    let mut r = Runner::new(&menu, &mut buffer, console);
    loop {
        let mut rx_buffer = [0u8; 64];
        let rx_buffer = match r
                .context
                .uarte
                .read_timeout(&mut rx_buffer, r.context.timer, 100_000) // Tenth of a second delay given the a 1hz prescaler so we can catch many chars
            {
                Ok(_) => &rx_buffer,
                Err(uarte::Error::Timeout(n)) => &rx_buffer[0..n],
                _ => break,
            };
        for b in rx_buffer {
            match *b {
                b if b as char == '\x1b' => {
                    break;
                }
                b if b as char == '\n' => {
                    r.input_byte(b'\r');
                }
                b => r.input_byte(b),
            }
        }
    }
}
