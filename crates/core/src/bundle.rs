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

// ---- archive key: a second age identity for encrypting arbitrary export files
// (ChatGPT/Claude/etc.) to a recipient the recovery circle can decrypt. Plain
// age of raw bytes (no tar wrapper) so a stock `age -d` opens it too. See
// spec/archive-key.md.

/// Derive the public age recipient (`age1...`) from an age secret-key string.
pub fn recipient_from_secret(secret: &str) -> Result<String> {
    let id = age::x25519::Identity::from_str(secret).map_err(|e| Error::BadKey(e.to_string()))?;
    Ok(id.to_public().to_string())
}

/// Encrypt raw bytes to `recipient` as a bare age file (no tar), suitable for a
/// single sidecar/archive export.
pub fn encrypt_file(recipient: &str, bytes: &[u8]) -> Result<Vec<u8>> {
    let recipient =
        age::x25519::Recipient::from_str(recipient).map_err(|e| Error::BadKey(e.to_string()))?;
    let encryptor =
        age::Encryptor::with_recipients(std::iter::once(&recipient as &dyn age::Recipient))
            .map_err(|e| Error::Age(e.to_string()))?;
    let mut out = Vec::new();
    let mut writer = encryptor.wrap_output(&mut out).map_err(|e| Error::Age(e.to_string()))?;
    writer.write_all(bytes)?;
    writer.finish().map_err(|e| Error::Age(e.to_string()))?;
    Ok(out)
}

/// Decrypt a bare age file (raw bytes, no tar) with an age secret-key string.
pub fn decrypt_file(secret: &str, data: &[u8]) -> Result<Vec<u8>> {
    let identity = age::x25519::Identity::from_str(secret).map_err(|e| Error::BadKey(e.to_string()))?;
    let decryptor = age::Decryptor::new(data).map_err(|e| Error::Age(e.to_string()))?;
    let mut reader = decryptor
        .decrypt(std::iter::once(&identity as &dyn age::Identity))
        .map_err(|e| Error::Age(e.to_string()))?;
    let mut out = Vec::new();
    reader.read_to_end(&mut out)?;
    Ok(out)
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
    fn archive_file_roundtrip_and_recipient() {
        // The archive key is an ordinary age identity; deriving its recipient
        // matches age's own to_public, and a bare-file round-trip works. This is
        // the whole crypto contract of the archive-key feature.
        let id = generate_identity();
        let secret = id.to_string();
        let recipient = recipient_from_secret(secret.expose_secret()).unwrap();
        assert_eq!(recipient, id.to_public().to_string());

        let msg = b"an exported conversations.json";
        let ct = encrypt_file(&recipient, msg).unwrap();
        assert_ne!(&ct[..], &msg[..], "must actually be encrypted");
        let pt = decrypt_file(secret.expose_secret(), &ct).unwrap();
        assert_eq!(pt, msg, "must round-trip");

        // Wrong identity cannot open it.
        let other = generate_identity();
        assert!(decrypt_file(other.to_string().expose_secret(), &ct).is_err());
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

// ------------------------------------------------------------- streaming ----

/// A `Write` sink that hands whatever age produces to a buffer the caller can
/// drain between chunks. `Rc<RefCell<_>>` rather than a channel because WASM is
/// single-threaded and this never crosses a thread.
#[derive(Clone, Default)]
struct SharedSink(std::rc::Rc<std::cell::RefCell<Vec<u8>>>);

impl Write for SharedSink {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl SharedSink {
    fn drain(&self) -> Vec<u8> {
        std::mem::take(&mut *self.0.borrow_mut())
    }
}

/// Incremental age encryption, for files too large to hold in memory.
///
/// `encrypt_file` takes the whole plaintext and returns the whole ciphertext, so
/// at peak it holds several copies of the file at once against a 32-bit WASM
/// address space. That is fine for a few hundred MB and fails somewhere past a
/// gigabyte, which is exactly the size of the AI-export ZIPs the playbooks tell
/// people to encrypt. This feeds age one chunk at a time and lets the caller
/// write each piece of ciphertext straight to disk, so peak memory is the chunk
/// size rather than the file size.
///
/// The output is an ordinary age file, byte-for-byte what `encrypt_file` would
/// have produced for the same input: `age -d` and the recovery tool both read it.
pub struct StreamingEncryptor {
    writer: Option<age::stream::StreamWriter<SharedSink>>,
    sink: SharedSink,
}

impl StreamingEncryptor {
    pub fn new(recipient: &str) -> Result<Self> {
        let recipient =
            age::x25519::Recipient::from_str(recipient).map_err(|e| Error::BadKey(e.to_string()))?;
        let encryptor =
            age::Encryptor::with_recipients(std::iter::once(&recipient as &dyn age::Recipient))
                .map_err(|e| Error::Age(e.to_string()))?;
        let sink = SharedSink::default();
        let writer = encryptor
            .wrap_output(sink.clone())
            .map_err(|e| Error::Age(e.to_string()))?;
        Ok(Self { writer: Some(writer), sink })
    }

    /// Feed one chunk of plaintext; returns whatever ciphertext that produced.
    /// May be empty (age buffers into 64 KiB payload chunks), which is normal.
    pub fn update(&mut self, chunk: &[u8]) -> Result<Vec<u8>> {
        let w = self.writer.as_mut().ok_or_else(|| Error::Age("encryptor already finished".into()))?;
        w.write_all(chunk)?;
        Ok(self.sink.drain())
    }

    /// Finish the file. Returns the trailing ciphertext, which must be appended
    /// or the file will not decrypt.
    pub fn finish(&mut self) -> Result<Vec<u8>> {
        let w = self.writer.take().ok_or_else(|| Error::Age("encryptor already finished".into()))?;
        w.finish().map_err(|e| Error::Age(e.to_string()))?;
        Ok(self.sink.drain())
    }
}

/// Incremental age decryption: the ciphertext is supplied up front, the
/// plaintext comes back a chunk at a time.
///
/// This is deliberately less capable than `StreamingEncryptor`, and the reason
/// is worth writing down. age decryption is pull-based and synchronous: the
/// reader asks its source for more bytes. A WASM call cannot suspend to await
/// the next chunk from JavaScript, so the ciphertext has to be resident before
/// decryption starts. What this DOES remove is the second copy: `decrypt_file`
/// accumulates the entire plaintext in a `Vec` as well, so peak memory is
/// ciphertext + plaintext. Here it is ciphertext + one chunk, which roughly
/// halves the ceiling and is the difference between a 1 GB archive opening and
/// failing.
///
/// Going further needs the ciphertext out of linear memory entirely (an OPFS
/// sync handle read from a Worker), which is a different piece of work.
pub struct StreamingDecryptor {
    reader: age::stream::StreamReader<std::io::Cursor<Vec<u8>>>,
}

impl StreamingDecryptor {
    /// Takes ownership of the ciphertext so nothing is copied a second time.
    pub fn new(secret: &str, data: Vec<u8>) -> Result<Self> {
        let identity =
            age::x25519::Identity::from_str(secret).map_err(|e| Error::BadKey(e.to_string()))?;
        let decryptor = age::Decryptor::new(std::io::Cursor::new(data))
            .map_err(|e| Error::Age(e.to_string()))?;
        let reader = decryptor
            .decrypt(std::iter::once(&identity as &dyn age::Identity))
            .map_err(|e| Error::Age(e.to_string()))?;
        Ok(Self { reader })
    }

    /// Next piece of plaintext, at most `max` bytes. An empty return means the
    /// file is finished; anything else means keep calling.
    pub fn read_chunk(&mut self, max: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; max];
        let mut filled = 0;
        while filled < max {
            match self.reader.read(&mut buf[filled..]) {
                Ok(0) => break,
                Ok(n) => filled += n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.into()),
            }
        }
        buf.truncate(filled);
        Ok(buf)
    }
}
