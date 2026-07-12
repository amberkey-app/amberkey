//! SLIP-0039 master secret encryption: 4-round Feistel with PBKDF2-HMAC-SHA256.

const ROUND_COUNT: u8 = 4;
const BASE_ITERATION_COUNT: u32 = 10_000;

fn round_function(round: u8, passphrase: &[u8], exponent: u8, salt: &[u8], r: &[u8]) -> Vec<u8> {
    let iterations = (BASE_ITERATION_COUNT << exponent) / ROUND_COUNT as u32;
    let mut password = Vec::with_capacity(1 + passphrase.len());
    password.push(round);
    password.extend_from_slice(passphrase);
    let mut full_salt = Vec::with_capacity(salt.len() + r.len());
    full_salt.extend_from_slice(salt);
    full_salt.extend_from_slice(r);
    let mut out = vec![0u8; r.len()];
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(&password, &full_salt, iterations, &mut out);
    out
}

fn salt(identifier: u16, extendable: bool) -> Vec<u8> {
    if extendable {
        Vec::new()
    } else {
        let mut s = b"shamir".to_vec();
        s.extend_from_slice(&identifier.to_be_bytes());
        s
    }
}

fn feistel(
    input: &[u8],
    passphrase: &[u8],
    exponent: u8,
    identifier: u16,
    extendable: bool,
    rounds: impl Iterator<Item = u8>,
) -> Vec<u8> {
    let half = input.len() / 2;
    let mut l = input[..half].to_vec();
    let mut r = input[half..].to_vec();
    let salt = salt(identifier, extendable);
    for i in rounds {
        let f = round_function(i, passphrase, exponent, &salt, &r);
        let new_r: Vec<u8> = l.iter().zip(&f).map(|(a, b)| a ^ b).collect();
        l = r;
        r = new_r;
    }
    let mut out = r;
    out.extend_from_slice(&l);
    out
}

pub fn encrypt(ms: &[u8], passphrase: &[u8], exponent: u8, identifier: u16, extendable: bool) -> Vec<u8> {
    feistel(ms, passphrase, exponent, identifier, extendable, 0..ROUND_COUNT)
}

pub fn decrypt(ems: &[u8], passphrase: &[u8], exponent: u8, identifier: u16, extendable: bool) -> Vec<u8> {
    feistel(ems, passphrase, exponent, identifier, extendable, (0..ROUND_COUNT).rev())
}
