//! SLIP-0039 share <-> mnemonic encoding.
//!
//! Word layout (10 bits per word):
//!   id(15) | ext(1) | e(4) | GI(4) | Gt-1(4) | g-1(4) | I(4) | t-1(4) | value | checksum(30)

use super::{rs1024, wordlist};
use crate::error::{Error, Result};

pub const ID_BITS: u16 = 15;
const METADATA_WORDS: usize = 7; // 4 header words + 3 checksum words
const MIN_MNEMONIC_WORDS: usize = 20; // 128-bit secret
pub const MIN_SECRET_LEN: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Share {
    pub identifier: u16,
    pub extendable: bool,
    pub iteration_exponent: u8,
    pub group_index: u8,
    pub group_threshold: u8, // 1-based
    pub group_count: u8,     // 1-based
    pub member_index: u8,
    pub member_threshold: u8, // 1-based
    pub value: Vec<u8>,
}

impl Share {
    fn customization(&self) -> &'static [u8] {
        if self.extendable {
            rs1024::CUSTOMIZATION_STRING_EXTENDABLE
        } else {
            rs1024::CUSTOMIZATION_STRING
        }
    }

    pub fn to_indices(&self) -> Vec<u16> {
        let mut w = Vec::new();
        w.push(self.identifier >> 5);
        w.push(
            ((self.identifier & 0x1F) << 5)
                | ((self.extendable as u16) << 4)
                | self.iteration_exponent as u16,
        );
        let gt = (self.group_threshold - 1) as u16;
        let gc = (self.group_count - 1) as u16;
        w.push(((self.group_index as u16) << 6) | (gt << 2) | (gc >> 2));
        w.push(((gc & 3) << 8) | ((self.member_index as u16) << 4) | (self.member_threshold - 1) as u16);

        // Value: big-endian bit string, left-padded with zeros to a multiple of 10 bits.
        let value_words = (self.value.len() * 8).div_ceil(10);
        let pad_bits = value_words * 10 - self.value.len() * 8;
        let mut acc: u32 = 0;
        let mut nbits = pad_bits;
        for byte in &self.value {
            acc = (acc << 8) | *byte as u32;
            nbits += 8;
            while nbits >= 10 {
                w.push(((acc >> (nbits - 10)) & 1023) as u16);
                nbits -= 10;
                acc &= (1 << nbits) - 1;
            }
        }
        debug_assert_eq!(nbits, 0);

        let checksum = rs1024::create_checksum(self.customization(), &w);
        w.extend_from_slice(&checksum);
        w
    }

    pub fn to_mnemonic(&self) -> String {
        self.to_indices()
            .iter()
            .map(|i| wordlist::WORDS[*i as usize])
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn from_indices(w: &[u16]) -> Result<Self> {
        if w.len() < MIN_MNEMONIC_WORDS {
            return Err(Error::InvalidLength);
        }
        let value_words = w.len() - METADATA_WORDS;
        let pad_bits = (10 * value_words) % 16;
        if pad_bits > 8 {
            return Err(Error::InvalidLength);
        }

        let identifier = (w[0] << 5) | (w[1] >> 5);
        let extendable = (w[1] >> 4) & 1 == 1;
        let customization = if extendable {
            rs1024::CUSTOMIZATION_STRING_EXTENDABLE
        } else {
            rs1024::CUSTOMIZATION_STRING
        };
        if !rs1024::verify_checksum(customization, w) {
            return Err(Error::InvalidChecksum);
        }

        let iteration_exponent = (w[1] & 0xF) as u8;
        let group_index = (w[2] >> 6) as u8;
        let group_threshold = ((w[2] >> 2) & 0xF) as u8 + 1;
        let group_count = (((w[2] & 3) << 2) | (w[3] >> 8)) as u8 + 1;
        let member_index = ((w[3] >> 4) & 0xF) as u8;
        let member_threshold = (w[3] & 0xF) as u8 + 1;
        if group_threshold > group_count {
            return Err(Error::InvalidGroupParams(format!(
                "group threshold {group_threshold} exceeds group count {group_count}"
            )));
        }

        // Decode value bits; the pad_bits high bits must be zero.
        let value_byte_count = (10 * value_words - pad_bits) / 8;
        let mut value = Vec::with_capacity(value_byte_count);
        let mut acc: u32 = 0;
        let mut nbits = 0usize;
        for (i, word) in w[4..4 + value_words].iter().enumerate() {
            if i == 0 && pad_bits > 0 && (*word >> (10 - pad_bits)) != 0 {
                return Err(Error::InvalidPadding);
            }
            acc = (acc << 10) | *word as u32;
            nbits += 10;
            if i == 0 {
                nbits -= pad_bits;
                acc &= (1 << nbits) - 1;
            }
            while nbits >= 8 {
                value.push(((acc >> (nbits - 8)) & 0xFF) as u8);
                nbits -= 8;
                acc &= (1 << nbits) - 1;
            }
        }
        debug_assert_eq!(value.len(), value_byte_count);
        if value.len() < MIN_SECRET_LEN {
            return Err(Error::InvalidSecretLength);
        }

        Ok(Share {
            identifier,
            extendable,
            iteration_exponent,
            group_index,
            group_threshold,
            group_count,
            member_index,
            member_threshold,
            value,
        })
    }

    pub fn from_mnemonic(mnemonic: &str) -> Result<Self> {
        let indices = mnemonic
            .split_whitespace()
            .map(|word| {
                let lower = word.to_ascii_lowercase();
                wordlist::word_index(&lower).ok_or_else(|| Error::InvalidWord(word.to_string()))
            })
            .collect::<Result<Vec<u16>>>()?;
        Self::from_indices(&indices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indices_roundtrip() {
        let share = Share {
            identifier: 0x1234,
            extendable: true,
            iteration_exponent: 1,
            group_index: 2,
            group_threshold: 2,
            group_count: 3,
            member_index: 4,
            member_threshold: 3,
            value: vec![0xAB; 32],
        };
        let decoded = Share::from_indices(&share.to_indices()).unwrap();
        assert_eq!(share, decoded);
        let decoded = Share::from_mnemonic(&share.to_mnemonic()).unwrap();
        assert_eq!(share, decoded);
    }
}
