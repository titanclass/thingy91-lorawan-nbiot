///! Configuration is provided such that it has a stable representation
///! for both in-memory storage per flash memory, and when communicated
///! between devices, perhaps over serial communications. The goal is to
///! use one form of serialisation and the method adopted uses Postcard
///! given its generality.
///! In particular, I wish to consider a future capability of bulk-flashing
///! configuration to devices during their manufacturing.
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
use nrf_hal_common::{nvmc::Nvmc, pac::NVMC_NS};
use postcard::{from_bytes, to_slice};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[repr(u32)]
pub enum Version {
    V1 = 1,
    Invalid = 0xffffffff, // Represents erased flash memory
}

pub type Ipv4Addr = [u8; 4];

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub version: Version,
    pub net_id: u32,
    pub nwkskey: Option<u128>,
    pub appskey: Option<u128>,
    pub iccid: Option<u64>,
    pub send_frequency_ms: u32,
    pub network_server_host: Option<Ipv4Addr>,
    pub network_server_port: u16,
}

impl Config {
    pub fn new() -> Self {
        Config {
            version: Version::V1,
            net_id: 0x13_u32,
            nwkskey: None,
            appskey: None,
            iccid: None,
            send_frequency_ms: 60 * 60 * 1000, // 1 hour
            network_server_host: None,
            network_server_port: 1694,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.nwkskey.is_some()
            && self.appskey.is_some()
            && self.iccid.is_some()
            && self.network_server_host.is_some()
    }

    pub fn load(nvmc: &mut Nvmc<NVMC_NS>) -> Result<Self, ConfigError> {
        let mut buf = [0u8; 64];
        match nvmc.try_read(0, &mut buf) {
            Ok(_) => match from_bytes::<Self>(&buf) {
                Ok(c) if c.version != Version::Invalid => Ok(c),
                _ => Ok(Self::new()),
            },
            Err(_) => Err(ConfigError::CannotLoad),
        }
    }

    pub fn save(&self, nvmc: &mut Nvmc<NVMC_NS>) -> Result<(), ConfigError> {
        let mut buf = [0u8; 64];
        match to_slice(&self, &mut buf) {
            Ok(_) => nvmc
                .try_erase(0, 4096)
                .and_then(|_| nvmc.try_write(0, &buf))
                .map_err(|_| ConfigError::CannotSave),
            Err(_) => Err(ConfigError::CannotSave),
        }
    }
}

pub enum ConfigError {
    CannotLoad,
    CannotSave,
}

impl<'a> Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
