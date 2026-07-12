//! Split/reconstruct round-trips across group configurations, plus negative tests.

use amberkey_core::slip39::{combine_mnemonics, generate_mnemonics, GroupSpec, Share};
use amberkey_core::Error;

fn g(threshold: u8, count: u8) -> GroupSpec {
    GroupSpec { threshold, count }
}

fn secret(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i * 37 + 11) as u8).collect()
}

#[test]
fn roundtrip_across_group_configs() {
    // (group_threshold, groups) — includes the AmberKey default template
    // (spouse 1-of-1, kids N-of-N, professional 1-of-1, 2 groups needed)
    // and no-spouse / friends-only / self-share variants.
    let configs: Vec<(u8, Vec<GroupSpec>)> = vec![
        (1, vec![g(1, 1)]),                            // self-share single card
        (1, vec![g(2, 3)]),                            // single group 2-of-3
        (1, vec![g(3, 5)]),                            // single group 3-of-5
        (2, vec![g(1, 1), g(2, 2), g(1, 1)]),          // default template, 2 kids
        (2, vec![g(1, 1), g(2, 3), g(1, 1)]),          // default template, 3 kids (N-1 of N)
        (2, vec![g(2, 3), g(3, 5)]),                   // friends-only variant
        (3, vec![g(1, 1), g(2, 2), g(2, 3), g(1, 1)]), // 3-of-4 groups
        (2, vec![g(16, 16), g(1, 1)]),                 // max member count
    ];
    for extendable in [false, true] {
        for len in [16, 32] {
            let ms = secret(len);
            for (gt, groups) in &configs {
                let sets = generate_mnemonics(*gt, groups, &ms, b"", 0, extendable).unwrap();
                // Exact-threshold reconstruction from the first `gt` groups.
                let mut quorum: Vec<String> = Vec::new();
                for (set, spec) in sets.iter().zip(groups).take(*gt as usize) {
                    quorum.extend(set.iter().take(spec.threshold as usize).cloned());
                }
                assert_eq!(combine_mnemonics(&quorum, b"").unwrap(), ms, "config {gt:?}/{groups:?}");

                // Over-provision: all shares at once also works.
                let all: Vec<String> = sets.iter().flatten().cloned().collect();
                assert_eq!(combine_mnemonics(&all, b"").unwrap(), ms);
            }
        }
    }
}

#[test]
fn passphrase_changes_output() {
    let ms = secret(16);
    let sets = generate_mnemonics(1, &[g(2, 3)], &ms, b"correct horse", 0, true).unwrap();
    let quorum = &sets[0][..2];
    assert_eq!(combine_mnemonics(quorum, b"correct horse").unwrap(), ms);
    // Wrong passphrase yields a *different* secret, not an error (SLIP-39 design).
    assert_ne!(combine_mnemonics(quorum, b"wrong").unwrap(), ms);
}

#[test]
fn below_threshold_fails() {
    let ms = secret(32);
    let sets = generate_mnemonics(2, &[g(1, 1), g(2, 2), g(1, 1)], &ms, b"", 0, true).unwrap();
    // One complete group only.
    let err = combine_mnemonics(&sets[0], b"").unwrap_err();
    assert!(matches!(err, Error::InsufficientGroups { .. }), "{err}");
    // Two groups but one incomplete (1 of 2 kids).
    let partial = vec![sets[0][0].clone(), sets[1][0].clone()];
    let err = combine_mnemonics(&partial, b"").unwrap_err();
    assert!(matches!(err, Error::InsufficientGroups { .. }), "{err}");
}

#[test]
fn corrupted_word_rejected_by_checksum() {
    let ms = secret(16);
    let sets = generate_mnemonics(1, &[g(2, 3)], &ms, b"", 0, true).unwrap();
    let mut words: Vec<&str> = sets[0][0].split(' ').collect();
    let replacement = if words[5] == "academic" { "acid" } else { "academic" };
    words[5] = replacement;
    let corrupted = words.join(" ");
    let err = combine_mnemonics(&[corrupted, sets[0][1].clone()], b"").unwrap_err();
    assert!(matches!(err, Error::InvalidChecksum), "{err}");
}

#[test]
fn tampered_value_rejected_by_digest() {
    let ms = secret(16);
    let sets = generate_mnemonics(1, &[g(2, 3)], &ms, b"", 0, true).unwrap();
    // Flip a value bit and re-encode with a valid checksum: digest must catch it.
    let mut share = Share::from_mnemonic(&sets[0][0]).unwrap();
    share.value[3] ^= 0x40;
    let forged = share.to_mnemonic();
    let err = combine_mnemonics(&[forged, sets[0][1].clone()], b"").unwrap_err();
    assert!(matches!(err, Error::InvalidDigest), "{err}");
}

#[test]
fn mixed_share_sets_rejected() {
    let ms = secret(16);
    let a = generate_mnemonics(1, &[g(2, 3)], &ms, b"", 0, true).unwrap();
    let b = generate_mnemonics(1, &[g(2, 3)], &ms, b"", 0, true).unwrap();
    let err = combine_mnemonics(&[a[0][0].clone(), b[0][1].clone()], b"").unwrap_err();
    assert!(matches!(err, Error::MismatchedShareSet(_)), "{err}");
}

#[test]
fn invalid_generate_params_rejected() {
    let ms = secret(16);
    assert!(generate_mnemonics(2, &[g(2, 3)], &ms, b"", 0, true).is_err()); // gt > groups
    assert!(generate_mnemonics(1, &[g(3, 2)], &ms, b"", 0, true).is_err()); // t > n
    assert!(generate_mnemonics(1, &[g(1, 2)], &ms, b"", 0, true).is_err()); // 1-of-n, n>1
    assert!(generate_mnemonics(1, &[g(1, 1)], &secret(15), b"", 0, true).is_err()); // odd len
    assert!(generate_mnemonics(1, &[g(1, 1)], &secret(8), b"", 0, true).is_err()); // too short
    assert!(generate_mnemonics(1, &[g(1, 1)], &ms, &[7u8], 0, true).is_err()); // bad passphrase
}
