//! RS1024 checksum from SLIP-0039.

const GEN: [u32; 10] = [
    0xE0E040, 0x1C1C080, 0x3838100, 0x7070200, 0xE0E0009, 0x1C0C2412, 0x38086C24, 0x3090FC48,
    0x21B1F890, 0x3F3F120,
];

pub const CUSTOMIZATION_STRING: &[u8] = b"shamir";
pub const CUSTOMIZATION_STRING_EXTENDABLE: &[u8] = b"shamir_extendable";

fn polymod(values: impl Iterator<Item = u32>) -> u32 {
    let mut chk: u32 = 1;
    for v in values {
        let b = chk >> 20;
        chk = ((chk & 0xFFFFF) << 10) ^ v;
        for (i, g) in GEN.iter().enumerate() {
            if (b >> i) & 1 != 0 {
                chk ^= g;
            }
        }
    }
    chk
}

fn cs_values(customization: &[u8]) -> impl Iterator<Item = u32> + '_ {
    customization.iter().map(|b| *b as u32)
}

pub fn create_checksum(customization: &[u8], data: &[u16]) -> [u16; 3] {
    let values = cs_values(customization)
        .chain(data.iter().map(|v| *v as u32))
        .chain([0u32; 3]);
    let pm = polymod(values) ^ 1;
    [
        ((pm >> 20) & 1023) as u16,
        ((pm >> 10) & 1023) as u16,
        (pm & 1023) as u16,
    ]
}

pub fn verify_checksum(customization: &[u8], data: &[u16]) -> bool {
    polymod(cs_values(customization).chain(data.iter().map(|v| *v as u32))) == 1
}
