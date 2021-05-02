#![cfg_attr(not(test), no_std)]

/// Form a device address given a Dev EUI and a LoRaWAN network id
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

#[cfg(test)]
mod nwk_addr_tests {
    use super::*;

    #[test]
    fn test_all() {
        assert_eq!(nwk_addr(0xffffffffe00c00fe, 0x00000003), 0x000c00fe);
        assert_eq!(nwk_addr(0xffffffffe00c00ff, 0x00000003), 0x000c00ff);
        assert_eq!(nwk_addr(0x00000000e00c00ff, 0x00fc0000), 0x0000007f);
    }
}
