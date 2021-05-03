#![cfg_attr(not(test), no_std)]

/// The payload to convey over LoRaWAN
pub struct LoRaWANPayload {
    pub temperature: i16,
    pub pressure: u32,
    pub humidity: u32,
    pub gas_resistance: u32,
}

impl LoRaWANPayload {
    /// Return the structure as big endian bytes
    /// ```
    /// use applib::LoRaWANPayload;
    /// let payload = LoRaWANPayload { temperature: -2, pressure: 0, humidity: 99, gas_resistance: 1 };
    /// assert_eq!(&payload.as_be_bytes(), &[0xff, 0xfe, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x63, 0x00, 0x00, 0x00, 0x01]);
    /// ```
    pub fn as_be_bytes(&self) -> [u8; 14] {
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
