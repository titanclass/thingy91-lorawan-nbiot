#![cfg_attr(not(test), no_std)]

use lorawan_encoding::keys;

/// Return a LoRaWAN data-up-confirmed payload We'll lay the packet out
// as follows, and using an FPort of 1:
///
/// Start |   End | Description
///     0 |     1 | Temperature (C) * 100
///     2 |     5 | Pressure (hPA) * 100
///     6 |     9 | Humidity (%) * 1000
///    10 |    13 | Gas Resistence
///
/// ```
/// use app::EnvironmentalPayload;
/// let bytes = app::data_up_unconfirmed(0, 0, &EnvironmentalPayload { temperature: 0, pressure: 1, humidity: 2, gas_resistance: 3 }, 0_u128, 0_u128);
/// ```
pub fn data_up_unconfirmed(
    dev_addr: u32,
    fcnt: u32,
    payload: &EnvironmentalPayload,
    nwk_skey: u128,
    app_skey: u128,
) -> [u8; 27] {
    let mut phy = lorawan_encoding::creator::DataPayloadCreator::new();
    phy.set_confirmed(false)
        .set_uplink(true)
        .set_f_port(1)
        .set_dev_addr(&dev_addr.to_le_bytes())
        .set_fcnt(fcnt);
    let bytes_ref = phy
        .build(
            &payload.to_be_bytes(),
            &[],
            &keys::AES128(nwk_skey.to_le_bytes()),
            &keys::AES128(app_skey.to_le_bytes()),
        )
        .unwrap();
    let mut bytes = [0_u8; 27];
    bytes.copy_from_slice(bytes_ref);
    bytes
}

/// The payload to convey over LoRaWAN
pub struct EnvironmentalPayload {
    pub temperature: i16,
    pub pressure: u32,
    pub humidity: u32,
    pub gas_resistance: u32,
}

impl EnvironmentalPayload {
    /// Return the structure as big endian bytes
    /// ```
    /// use app::EnvironmentalPayload;
    /// let payload = EnvironmentalPayload { temperature: -2, pressure: 0, humidity: 99, gas_resistance: 1 };
    /// assert_eq!(&payload.to_be_bytes(), &[0xff, 0xfe, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x00, 0x00, 0x00, 0x01]);
    /// ```
    pub fn to_be_bytes(&self) -> [u8; 14] {
        [
            ((self.temperature as u16 & 0xff00) >> 8) as u8,
            (self.temperature as u16 & 0x00ff) as u8,
            ((self.pressure as u32 & 0xff000000) >> 24) as u8,
            ((self.pressure as u32 & 0x00ff0000) >> 16) as u8,
            ((self.pressure as u32 & 0x0000ff00) >> 8) as u8,
            (self.pressure as u32 & 0x000000ff) as u8,
            ((self.humidity as u32 & 0xff000000) >> 24) as u8,
            ((self.humidity as u32 & 0x00ff0000) >> 16) as u8,
            ((self.humidity as u32 & 0x0000ff00) >> 8) as u8,
            (self.humidity as u32 & 0x000000ff) as u8,
            ((self.gas_resistance as u32 & 0xff000000) >> 24) as u8,
            ((self.gas_resistance as u32 & 0x00ff0000) >> 16) as u8,
            ((self.gas_resistance as u32 & 0x0000ff00) >> 8) as u8,
            (self.gas_resistance as u32 & 0x000000ff) as u8,
        ]
    }
}

/// Form a device address given a Dev EUI and a LoRaWAN network id.
/// '''
/// assert_eq!(nwk_addr(0xffffffffe00c00fe, 0x00000003), 0x000c00fe);
/// assert_eq!(nwk_addr(0xffffffffe00c00ff, 0x00000003), 0x000c00ff);
/// assert_eq!(nwk_addr(0x00000000e00c00ff, 0x00fc0000), 0x0000007f);
/// ```
pub fn nwk_addr(dev_eui: u64, net_id: u32) -> u32 {
    let dev_eui_lower = (dev_eui & 0x00000000ffffffff) as u32;
    match net_id & 0x00e00000 {
        0x00e00000 => dev_eui_lower & 0x0000007f,
        0x00c00000 => dev_eui_lower & 0x000003ff,
        0x00a00000 => dev_eui_lower & 0x00001fff,
        0x00800000 => dev_eui_lower & 0x0000ffff,
        0x00600000 => dev_eui_lower & 0x0003ffff,
        0x00400000 => dev_eui_lower & 0x000fffff,
        0x00200000 => dev_eui_lower & 0x00ffffff,
        _ => dev_eui_lower & 0x01ffffff,
    }
}
