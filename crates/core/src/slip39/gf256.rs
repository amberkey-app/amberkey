//! GF(256) arithmetic and Lagrange interpolation for SLIP-39 Shamir sharing.
//! Field: x^8 + x^4 + x^3 + x + 1 (0x11B), log/exp tables over generator 3,
//! exactly as in the SLIP-0039 reference implementation.

use crate::error::{Error, Result};

const fn tables() -> ([u8; 255], [u8; 256]) {
    let mut exp = [0u8; 255];
    let mut log = [0u8; 256];
    let mut poly: u16 = 1;
    let mut i = 0;
    while i < 255 {
        exp[i] = poly as u8;
        log[poly as usize] = i as u8;
        // multiply by generator 3 = x + 1
        poly = (poly << 1) ^ poly;
        if poly & 0x100 != 0 {
            poly ^= 0x11B;
        }
        i += 1;
    }
    (exp, log)
}

const TABLES: ([u8; 255], [u8; 256]) = tables();
const EXP: [u8; 255] = TABLES.0;
const LOG: [u8; 256] = TABLES.1;

/// Evaluate the polynomial determined by `shares` (points (x, y-vector)) at `x`.
pub fn interpolate(shares: &[(u8, Vec<u8>)], x: u8) -> Result<Vec<u8>> {
    let len = shares
        .first()
        .ok_or(Error::InsufficientGroups { got: 0, needed: 1 })?
        .1
        .len();
    if shares.iter().any(|(_, y)| y.len() != len) {
        return Err(Error::MismatchedShareSet("share length"));
    }
    let mut seen = [false; 256];
    for (xi, _) in shares {
        if seen[*xi as usize] {
            return Err(Error::DuplicateShareIndex);
        }
        seen[*xi as usize] = true;
    }
    if let Some((_, y)) = shares.iter().find(|(xi, _)| *xi == x) {
        return Ok(y.clone());
    }

    // Logarithm of the product of (x_i - x) for i = 1..m.
    let log_prod: i64 = shares.iter().map(|(xi, _)| LOG[(xi ^ x) as usize] as i64).sum();

    let mut result = vec![0u8; len];
    for (xi, yi) in shares {
        // Sum of logs of (x_i - x_j) for all j (self term is LOG[0] = 0).
        let log_denom: i64 = shares
            .iter()
            .map(|(xj, _)| LOG[(xi ^ xj) as usize] as i64)
            .sum();
        let log_basis = (log_prod - LOG[(xi ^ x) as usize] as i64 - log_denom).rem_euclid(255);
        for (r, y) in result.iter_mut().zip(yi) {
            if *y != 0 {
                *r ^= EXP[((LOG[*y as usize] as i64 + log_basis) % 255) as usize];
            }
        }
    }
    Ok(result)
}
