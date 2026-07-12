//! Acceptance gate: the full official Trezor SLIP-39 test vector suite.
//! Source: https://github.com/trezor/python-shamir-mnemonic/blob/master/vectors.json
//! Each vector: [description, mnemonics, master_secret_hex, bip32_xprv].
//! We assert on the master secret (the SLIP-39 output); the xprv column is a
//! downstream BIP32 derivation outside SLIP-39 scope.

use amberkey_core::slip39::combine_mnemonics;

fn hex_decode(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn full_trezor_vector_suite() {
    let raw = include_str!("data/trezor_vectors.json");
    let vectors: Vec<(String, Vec<String>, String, String)> = serde_json::from_str(raw).unwrap();
    assert!(vectors.len() >= 45, "expected the full suite, got {}", vectors.len());

    for (desc, mnemonics, secret_hex, _xprv) in &vectors {
        let result = combine_mnemonics(mnemonics, b"TREZOR");
        if secret_hex.is_empty() {
            assert!(result.is_err(), "vector must fail but succeeded: {desc}");
        } else {
            let secret = result.unwrap_or_else(|e| panic!("vector failed: {desc}: {e}"));
            assert_eq!(secret, hex_decode(secret_hex), "wrong master secret: {desc}");
        }
    }
}
