//! SLIP-0039 Shamir's Secret-Sharing for Mnemonic Codes, implemented to spec.
//! Acceptance gate: the full official Trezor test vector suite (see tests/).

mod cipher;
mod gf256;
mod rs1024;
mod share;
mod wordlist;

pub use share::Share;
pub use wordlist::WORDS;

use crate::error::{Error, Result};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::BTreeMap;

const DIGEST_INDEX: u8 = 254;
const SECRET_INDEX: u8 = 255;
const DIGEST_LEN: usize = 4;
const MAX_SHARE_COUNT: u8 = 16;

/// AmberKey defaults: extendable backup flag on (re-share friendly), e=1 => 20000 PBKDF2 iterations.
pub const DEFAULT_ITERATION_EXPONENT: u8 = 1;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct GroupSpec {
    pub threshold: u8,
    pub count: u8,
}

fn share_digest(random: &[u8], secret: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(random).expect("hmac accepts any key length");
    mac.update(secret);
    mac.finalize().into_bytes()[..DIGEST_LEN].to_vec()
}

fn random_bytes(n: usize) -> Vec<u8> {
    let mut buf = vec![0u8; n];
    getrandom::getrandom(&mut buf).expect("OS randomness unavailable");
    buf
}

/// Split `secret` into `count` Shamir shares with the given `threshold`.
fn split_secret(threshold: u8, count: u8, secret: &[u8]) -> Result<Vec<(u8, Vec<u8>)>> {
    if threshold == 0 || threshold > count || count > MAX_SHARE_COUNT {
        return Err(Error::InvalidGroupParams(format!(
            "invalid threshold {threshold} of {count}"
        )));
    }
    if threshold == 1 {
        return Ok((0..count).map(|i| (i, secret.to_vec())).collect());
    }
    if secret.len() < share::MIN_SECRET_LEN {
        return Err(Error::InvalidSecretLength);
    }

    let random_share_count = threshold as usize - 2;
    let mut shares: Vec<(u8, Vec<u8>)> = (0..random_share_count as u8)
        .map(|i| (i, random_bytes(secret.len())))
        .collect();

    let random_part = random_bytes(secret.len() - DIGEST_LEN);
    let mut digest_share = share_digest(&random_part, secret);
    digest_share.extend_from_slice(&random_part);

    let mut base = shares.clone();
    base.push((DIGEST_INDEX, digest_share));
    base.push((SECRET_INDEX, secret.to_vec()));

    for i in random_share_count as u8..count {
        shares.push((i, gf256::interpolate(&base, i)?));
    }
    Ok(shares)
}

/// Recover a secret from exactly-`threshold` shares, verifying the embedded digest.
fn recover_secret(threshold: u8, shares: &[(u8, Vec<u8>)]) -> Result<Vec<u8>> {
    if threshold == 1 {
        return Ok(shares[0].1.clone());
    }
    let secret = gf256::interpolate(shares, SECRET_INDEX)?;
    let digest_share = gf256::interpolate(shares, DIGEST_INDEX)?;
    let (digest, random_part) = digest_share.split_at(DIGEST_LEN);
    if digest != share_digest(random_part, &secret) {
        return Err(Error::InvalidDigest);
    }
    Ok(secret)
}

fn check_passphrase(passphrase: &[u8]) -> Result<()> {
    if passphrase.iter().all(|c| (32..=126).contains(c)) {
        Ok(())
    } else {
        Err(Error::InvalidPassphrase)
    }
}

/// Split `master_secret` per SLIP-39 two-level scheme. Returns one Vec of mnemonics per group.
pub fn generate_mnemonics(
    group_threshold: u8,
    groups: &[GroupSpec],
    master_secret: &[u8],
    passphrase: &[u8],
    iteration_exponent: u8,
    extendable: bool,
) -> Result<Vec<Vec<String>>> {
    if master_secret.len() < share::MIN_SECRET_LEN || !master_secret.len().is_multiple_of(2) {
        return Err(Error::InvalidSecretLength);
    }
    check_passphrase(passphrase)?;
    if group_threshold == 0 || group_threshold as usize > groups.len() || groups.len() > MAX_SHARE_COUNT as usize {
        return Err(Error::InvalidGroupParams(format!(
            "invalid group threshold {group_threshold} of {}",
            groups.len()
        )));
    }
    for g in groups {
        if g.threshold == 1 && g.count > 1 {
            return Err(Error::InvalidGroupParams(
                "1-of-n groups with n > 1 are not allowed; use separate groups".into(),
            ));
        }
    }

    let identifier = u16::from_be_bytes(random_bytes(2).try_into().unwrap()) & ((1 << share::ID_BITS) - 1);
    let ems = cipher::encrypt(master_secret, passphrase, iteration_exponent, identifier, extendable);

    let group_shares = split_secret(group_threshold, groups.len() as u8, &ems)?;
    let mut out = Vec::with_capacity(groups.len());
    for ((group_index, group_secret), spec) in group_shares.iter().zip(groups) {
        let member_shares = split_secret(spec.threshold, spec.count, group_secret)?;
        out.push(
            member_shares
                .into_iter()
                .map(|(member_index, value)| {
                    Share {
                        identifier,
                        extendable,
                        iteration_exponent,
                        group_index: *group_index,
                        group_threshold,
                        group_count: groups.len() as u8,
                        member_index,
                        member_threshold: spec.threshold,
                        value,
                    }
                    .to_mnemonic()
                })
                .collect(),
        );
    }
    Ok(out)
}

/// Combine shares back into the master secret. Lenient: extra shares beyond the
/// thresholds are tolerated; the first `threshold` members of each complete group
/// and the first `group_threshold` complete groups are used.
pub fn combine_mnemonics(mnemonics: &[impl AsRef<str>], passphrase: &[u8]) -> Result<Vec<u8>> {
    let shares = mnemonics
        .iter()
        .map(|m| Share::from_mnemonic(m.as_ref()))
        .collect::<Result<Vec<Share>>>()?;
    combine_shares(&shares, passphrase)
}

pub fn combine_shares(shares: &[Share], passphrase: &[u8]) -> Result<Vec<u8>> {
    check_passphrase(passphrase)?;
    let first = shares.first().ok_or(Error::InsufficientGroups { got: 0, needed: 1 })?;
    for s in shares {
        if s.identifier != first.identifier || s.extendable != first.extendable {
            return Err(Error::MismatchedShareSet("identifier"));
        }
        if s.iteration_exponent != first.iteration_exponent {
            return Err(Error::MismatchedShareSet("iteration exponent"));
        }
        if s.group_threshold != first.group_threshold {
            return Err(Error::MismatchedShareSet("group threshold"));
        }
        if s.group_count != first.group_count {
            return Err(Error::MismatchedShareSet("group count"));
        }
        if s.value.len() != first.value.len() {
            return Err(Error::MismatchedShareSet("share length"));
        }
    }

    // group_index -> (member_threshold, member_index -> value)
    let mut groups: BTreeMap<u8, (u8, BTreeMap<u8, Vec<u8>>)> = BTreeMap::new();
    for s in shares {
        let entry = groups
            .entry(s.group_index)
            .or_insert_with(|| (s.member_threshold, BTreeMap::new()));
        if entry.0 != s.member_threshold {
            return Err(Error::MismatchedMemberThreshold);
        }
        if let Some(existing) = entry.1.get(&s.member_index) {
            if *existing != s.value {
                return Err(Error::DuplicateMemberIndex);
            }
        } else {
            entry.1.insert(s.member_index, s.value.clone());
        }
    }

    let needed = first.group_threshold as usize;
    let mut group_shares: Vec<(u8, Vec<u8>)> = Vec::new();
    for (group_index, (member_threshold, members)) in &groups {
        if members.len() < *member_threshold as usize {
            continue; // incomplete group
        }
        let member_shares: Vec<(u8, Vec<u8>)> = members
            .iter()
            .take(*member_threshold as usize)
            .map(|(i, v)| (*i, v.clone()))
            .collect();
        group_shares.push((*group_index, recover_secret(*member_threshold, &member_shares)?));
        if group_shares.len() == needed {
            break;
        }
    }
    if group_shares.len() < needed {
        return Err(Error::InsufficientGroups { got: group_shares.len(), needed });
    }

    let ems = recover_secret(first.group_threshold, &group_shares)?;
    Ok(cipher::decrypt(
        &ems,
        passphrase,
        first.iteration_exponent,
        first.identifier,
        first.extendable,
    ))
}
