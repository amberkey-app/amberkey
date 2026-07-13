//! Continuity bundle: an age-encrypted tar archive. See spec/bundle-format.md.
//!
//! The vault master key is an age X25519 identity; its raw 32-byte scalar is
//! what SLIP-39 splits (see spec/bundle-format.md for the exact encoding).

use crate::error::{Error, Result};
use age::secrecy::ExposeSecret;
use bech32::{Bech32, Hrp};
use std::io::{Read, Write};
use std::str::FromStr;

pub const BUNDLE_SCHEMA_VERSION: u32 = 1;
pub const BUNDLE_FORMAT: &str = "amberkey-bundle";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manifest {
    pub format: String,
    pub schema_version: u32,
    /// RFC 3339; supplied by the caller (core has no clock on WASM).
    pub created_at: String,
    pub owner_name: String,
}

impl Manifest {
    pub fn new(owner_name: &str, created_at: &str) -> Self {
        Manifest {
            format: BUNDLE_FORMAT.into(),
            schema_version: BUNDLE_SCHEMA_VERSION,
            created_at: created_at.into(),
            owner_name: owner_name.into(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BundleFile {
    pub path: String,
    #[serde(with = "b64")]
    pub data: Vec<u8>,
}

mod b64 {
    use base64::Engine;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(data: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&base64::engine::general_purpose::STANDARD.encode(data))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(d)?;
        base64::engine::general_purpose::STANDARD
            .decode(s)
            .map_err(serde::de::Error::custom)
    }
}

/// Generate a fresh vault master key (age X25519 identity).
pub fn generate_identity() -> age::x25519::Identity {
    age::x25519::Identity::generate()
}

const AGE_HRP: &str = "age-secret-key-";

/// The raw 32-byte scalar of an age identity — the SLIP-39 master secret.
pub fn identity_to_bytes(identity: &age::x25519::Identity) -> Result<[u8; 32]> {
    let s = identity.to_string();
    let (hrp, data) = bech32::decode(s.expose_secret()).map_err(|e| Error::BadKey(e.to_string()))?;
    if hrp.to_lowercase() != AGE_HRP {
        return Err(Error::BadKey("not an age secret key".into()));
    }
    data.try_into().map_err(|_| Error::BadKey("bad scalar length".into()))
}

/// Rebuild the age identity from the 32-byte scalar recovered via SLIP-39.
pub fn identity_from_bytes(scalar: &[u8]) -> Result<age::x25519::Identity> {
    if scalar.len() != 32 {
        return Err(Error::BadKey(format!("expected 32 bytes, got {}", scalar.len())));
    }
    let hrp = Hrp::parse(AGE_HRP).expect("valid hrp");
    let encoded = bech32::encode::<Bech32>(hrp, scalar)
        .map_err(|e| Error::BadKey(e.to_string()))?
        .to_uppercase();
    age::x25519::Identity::from_str(&encoded).map_err(|e| Error::BadKey(e.to_string()))
}

/// Tar the files and encrypt to `recipient` (the vault public key).
pub fn encrypt_bundle(recipient: &str, files: &[BundleFile]) -> Result<Vec<u8>> {
    let recipient =
        age::x25519::Recipient::from_str(recipient).map_err(|e| Error::BadKey(e.to_string()))?;

    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        for f in files {
            let mut header = tar::Header::new_ustar();
            header.set_size(f.data.len() as u64);
            header.set_mode(0o644);
            header.set_mtime(0); // fixed for reproducibility
            builder
                .append_data(&mut header, &f.path, f.data.as_slice())
                .map_err(|e| Error::Bundle(e.to_string()))?;
        }
        builder.finish().map_err(|e| Error::Bundle(e.to_string()))?;
    }

    let encryptor = age::Encryptor::with_recipients(std::iter::once(&recipient as &dyn age::Recipient))
        .map_err(|e| Error::Age(e.to_string()))?;
    let mut out = Vec::new();
    let mut writer = encryptor.wrap_output(&mut out).map_err(|e| Error::Age(e.to_string()))?;
    writer.write_all(&tar_bytes)?;
    writer.finish().map_err(|e| Error::Age(e.to_string()))?;
    Ok(out)
}

/// Decrypt a bundle with the vault identity and unpack the tar.
pub fn decrypt_bundle(identity: &age::x25519::Identity, data: &[u8]) -> Result<Vec<BundleFile>> {
    let decryptor = age::Decryptor::new(data).map_err(|e| Error::Age(e.to_string()))?;
    let mut reader = decryptor
        .decrypt(std::iter::once(identity as &dyn age::Identity))
        .map_err(|e| Error::Age(e.to_string()))?;
    let mut tar_bytes = Vec::new();
    reader.read_to_end(&mut tar_bytes)?;

    let mut archive = tar::Archive::new(tar_bytes.as_slice());
    let mut files = Vec::new();
    for entry in archive.entries().map_err(|e| Error::Bundle(e.to_string()))? {
        let mut entry = entry.map_err(|e| Error::Bundle(e.to_string()))?;
        let path = entry
            .path()
            .map_err(|e| Error::Bundle(e.to_string()))?
            .to_string_lossy()
            .into_owned();
        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        files.push(BundleFile { path, data });
    }
    Ok(files)
}

/// Find and parse the manifest in a decrypted bundle.
pub fn read_manifest(files: &[BundleFile]) -> Result<Manifest> {
    let f = files
        .iter()
        .find(|f| f.path == "manifest.json")
        .ok_or_else(|| Error::Bundle("manifest.json missing".into()))?;
    let m: Manifest =
        serde_json::from_slice(&f.data).map_err(|e| Error::Bundle(format!("bad manifest: {e}")))?;
    if m.format != BUNDLE_FORMAT {
        return Err(Error::Bundle(format!("unknown bundle format {:?}", m.format)));
    }
    // Decryption of any schema version must remain supported forever; version
    // gates parsing of newer optional sections, never rejects.
    Ok(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_scalar_roundtrip() {
        let id = generate_identity();
        let bytes = identity_to_bytes(&id).unwrap();
        let id2 = identity_from_bytes(&bytes).unwrap();
        assert_eq!(
            id.to_string().expose_secret(),
            id2.to_string().expose_secret()
        );
        assert_eq!(id.to_public().to_string(), id2.to_public().to_string());
    }

    /// Forward compatibility (seedplan §11): a bundle written by a NEWER
    /// schema — higher version, unknown manifest fields, unknown files —
    /// must still open, parse, and expose its packet in today's reader.
    #[test]
    fn old_reader_opens_newer_bundle() {
        let id = generate_identity();
        let futuristic_manifest = serde_json::json!({
            "format": "amberkey-bundle",
            "schema_version": 99,
            "created_at": "2040-01-01T00:00:00Z",
            "owner_name": "Future Owner",
            "brand_new_field": { "nested": true },
            "another_unknown": [1, 2, 3],
        });
        let files = vec![
            BundleFile { path: "manifest.json".into(), data: serde_json::to_vec(&futuristic_manifest).unwrap() },
            BundleFile { path: "packet/executor-checklist.md".into(), data: b"# Checklist".to_vec() },
            BundleFile { path: "packet/new-section/hologram.bin".into(), data: vec![0xAA; 64] },
            BundleFile { path: "quantum-extras.json".into(), data: b"{}".to_vec() },
        ];
        let ct = encrypt_bundle(&id.to_public().to_string(), &files).unwrap();
        let out = decrypt_bundle(&id, &ct).unwrap();
        assert_eq!(out.len(), 4, "unknown files must be preserved, not rejected");
        let m = read_manifest(&out).unwrap();
        assert_eq!(m.schema_version, 99);
        assert_eq!(m.owner_name, "Future Owner");
        assert!(out.iter().any(|f| f.path == "packet/executor-checklist.md"));
    }

    #[test]
    fn bundle_roundtrip() {
        let id = generate_identity();
        let files = vec![
            BundleFile {
                path: "manifest.json".into(),
                data: serde_json::to_vec(&Manifest::new("Test Owner", "2026-07-06T00:00:00Z")).unwrap(),
            },
            BundleFile { path: "packet/cards/google.json".into(), data: b"{}".to_vec() },
            BundleFile { path: "secrets/wallet-seed.json".into(), data: vec![0u8; 4096] },
        ];
        let ct = encrypt_bundle(&id.to_public().to_string(), &files).unwrap();
        let out = decrypt_bundle(&id, &ct).unwrap();
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].path, "manifest.json");
        assert_eq!(out[2].data, vec![0u8; 4096]);
        let m = read_manifest(&out).unwrap();
        assert_eq!(m.owner_name, "Test Owner");
        assert_eq!(m.schema_version, BUNDLE_SCHEMA_VERSION);

        // wrong key must fail
        let other = generate_identity();
        assert!(decrypt_bundle(&other, &ct).is_err());
    }
}
