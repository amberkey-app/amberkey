//! Generate a test fixture: shares + encrypted bundle, for CLI smoke tests and
//! the recovery-tool Playwright tests. Usage: cargo run -p amberkey-core --example make_fixture -- <outdir>

use amberkey_core::bundle::{self, BundleFile, Manifest};
use amberkey_core::slip39::{self, GroupSpec};

fn main() {
    let out = std::env::args().nth(1).expect("usage: make_fixture <outdir>");
    std::fs::create_dir_all(&out).unwrap();

    let identity = bundle::generate_identity();
    let scalar = bundle::identity_to_bytes(&identity).unwrap();

    // Default template: spouse 1-of-1, kids 2-of-2, professional 1-of-1, any 2 groups.
    let groups = [
        GroupSpec { threshold: 1, count: 1 },
        GroupSpec { threshold: 2, count: 2 },
        GroupSpec { threshold: 1, count: 1 },
    ];
    let mnemonics = slip39::generate_mnemonics(2, &groups, &scalar, b"", 1, true).unwrap();
    let flat: Vec<String> = mnemonics.iter().flatten().cloned().collect();
    std::fs::write(format!("{out}/shares.txt"), flat.join("\n") + "\n").unwrap();
    std::fs::write(
        format!("{out}/shares.json"),
        serde_json::to_string_pretty(&mnemonics).unwrap(),
    )
    .unwrap();

    let files = vec![
        BundleFile {
            path: "manifest.json".into(),
            data: serde_json::to_vec_pretty(&Manifest::new("Alex Fixture", "2026-07-06T00:00:00Z")).unwrap(),
        },
        BundleFile {
            path: "packet/executor-checklist.md".into(),
            data: b"# Executor checklist\n\n**Do not cancel the phone number** until every account is settled.\n\n1. Read the coverage summary.\n2. Work through each account card in packet/cards/.\n".to_vec(),
        },
        BundleFile {
            path: "packet/cards/google.json".into(),
            data: br#"{"id":"11111111-1111-1111-1111-111111111111","service":"google","label":"Personal Gmail","strategy":"native","metadata":{"institution":"","last4":"","notes_md":""},"native_config":{"mechanism":"Inactive Account Manager","designees":["spouse"],"delay":"3 months","last_attested":"2026-07-01"},"layer2_refs":[],"playbook_ref":"playbooks/google.md","executor_instructions_md":"Wait for the IAM email, or use the deceased-user request form."}"#.to_vec(),
        },
        BundleFile {
            path: "secrets/wallet-seed.json".into(),
            data: br#"{"id":"22222222-2222-2222-2222-222222222222","label":"Hardware wallet seed","kind":"crypto_seed","value":"test test test junk","notes_md":"Sweep immediately on recovery."}"#.to_vec(),
        },
        BundleFile {
            path: "circle.json".into(),
            data: br#"{"group_threshold":2,"groups":[{"name":"Partner","member_threshold":1,"members":[{"name":"Sam","relationship":"spouse","email":"sam@example.com","phone":"","card_id":"FIXTUR-G1-M1"}]},{"name":"Kids","member_threshold":2,"members":[{"name":"Kid A","relationship":"child","email":"","phone":"","card_id":"FIXTUR-G2-M1"},{"name":"Kid B","relationship":"child","email":"","phone":"","card_id":"FIXTUR-G2-M2"}]},{"name":"Professional","member_threshold":1,"members":[{"name":"Pat Attorney","relationship":"attorney","email":"pat@example.com","phone":"","card_id":"FIXTUR-G3-M1"}]}]}"#.to_vec(),
        },
        BundleFile {
            path: "playbook-snapshot/google.md".into(),
            data: b"---\nverified: 2026-07-06\n---\n# Google (snapshot)\nFrozen copy of the playbook at export time.\n".to_vec(),
        },
    ];
    let ct = bundle::encrypt_bundle(&identity.to_public().to_string(), &files).unwrap();
    std::fs::write(format!("{out}/bundle.age"), &ct).unwrap();
    println!("fixture written to {out}: shares.txt shares.json bundle.age");
}
