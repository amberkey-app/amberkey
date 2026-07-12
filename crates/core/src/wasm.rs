//! wasm-bindgen surface. JSON strings in/out; bytes as Uint8Array.
//! This is the ONLY crypto entry point for the web app and recovery tool.

use crate::error::Error;
use crate::{bundle, slip39};
use age::secrecy::ExposeSecret;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

fn err(e: Error) -> JsError {
    JsError::new(&e.to_string())
}

#[derive(serde::Serialize)]
struct KeyPair {
    secret: String,
    public: String,
}

fn keypair_json(id: &age::x25519::Identity) -> String {
    serde_json::to_string(&KeyPair {
        secret: id.to_string().expose_secret().to_string(),
        public: id.to_public().to_string(),
    })
    .expect("serializable")
}

/// Generate a fresh vault master key. Returns {"secret","public"}.
#[wasm_bindgen]
pub fn generate_identity() -> String {
    keypair_json(&bundle::generate_identity())
}

/// Split an age identity into SLIP-39 mnemonics.
/// `groups_json`: [{"threshold":N,"count":M}, ...]. Returns [[mnemonic]].
#[wasm_bindgen]
pub fn split_identity(
    secret: &str,
    group_threshold: u8,
    groups_json: &str,
) -> Result<String, JsError> {
    let identity =
        age::x25519::Identity::from_str(secret).map_err(|e| JsError::new(&e.to_string()))?;
    let groups: Vec<slip39::GroupSpec> =
        serde_json::from_str(groups_json).map_err(|e| JsError::new(&e.to_string()))?;
    let scalar = bundle::identity_to_bytes(&identity).map_err(err)?;
    let mnemonics = slip39::generate_mnemonics(
        group_threshold,
        &groups,
        &scalar,
        b"",
        slip39::DEFAULT_ITERATION_EXPONENT,
        true,
    )
    .map_err(err)?;
    Ok(serde_json::to_string(&mnemonics).expect("serializable"))
}

/// Combine SLIP-39 mnemonics (JSON array of strings) back into the vault key.
/// Returns {"secret","public"}.
#[wasm_bindgen]
pub fn combine_shares(mnemonics_json: &str) -> Result<String, JsError> {
    let mnemonics: Vec<String> =
        serde_json::from_str(mnemonics_json).map_err(|e| JsError::new(&e.to_string()))?;
    let scalar = slip39::combine_mnemonics(&mnemonics, b"").map_err(err)?;
    let identity = bundle::identity_from_bytes(&scalar).map_err(err)?;
    Ok(keypair_json(&identity))
}

/// Validate a single mnemonic; returns its card-relevant metadata or throws.
#[wasm_bindgen]
pub fn inspect_share(mnemonic: &str) -> Result<String, JsError> {
    let s = slip39::Share::from_mnemonic(mnemonic).map_err(err)?;
    Ok(serde_json::json!({
        "identifier": s.identifier,
        "group_index": s.group_index,
        "group_threshold": s.group_threshold,
        "group_count": s.group_count,
        "member_index": s.member_index,
        "member_threshold": s.member_threshold,
    })
    .to_string())
}

/// Encrypt a continuity bundle. `files_json`: [{"path","data"(base64)}].
#[wasm_bindgen]
pub fn encrypt_bundle(recipient: &str, files_json: &str) -> Result<Vec<u8>, JsError> {
    let files: Vec<bundle::BundleFile> =
        serde_json::from_str(files_json).map_err(|e| JsError::new(&e.to_string()))?;
    bundle::encrypt_bundle(recipient, &files).map_err(err)
}

/// Decrypt a continuity bundle with the identity secret string.
/// Returns [{"path","data"(base64)}].
#[wasm_bindgen]
pub fn decrypt_bundle(secret: &str, data: &[u8]) -> Result<String, JsError> {
    let identity =
        age::x25519::Identity::from_str(secret).map_err(|e| JsError::new(&e.to_string()))?;
    let files = bundle::decrypt_bundle(&identity, data).map_err(err)?;
    Ok(serde_json::to_string(&files).expect("serializable"))
}

/// Generate the print-at-home kit PDF from a KitInput JSON (see kit.rs).
/// Runs entirely client-side; mnemonics never leave the device.
/// Kit-feature only: the owner console needs it, the recovery tool does not.
/// QR code as inline SVG (used by TOTP enrollment).
#[cfg(feature = "kit")]
#[wasm_bindgen]
pub fn qr_svg(data: &str) -> Result<String, JsError> {
    crate::kit::qr_svg(data).map_err(err)
}

#[cfg(feature = "kit")]
#[wasm_bindgen]
pub fn generate_kit(kit_input_json: &str) -> Result<Vec<u8>, JsError> {
    let input: crate::kit::KitInput =
        serde_json::from_str(kit_input_json).map_err(|e| JsError::new(&e.to_string()))?;
    crate::kit::generate_kit_pdf(&input).map_err(err)
}

/// One-step recovery: mnemonics (JSON array) + encrypted bundle -> files JSON.
#[wasm_bindgen]
pub fn recover_bundle(mnemonics_json: &str, data: &[u8]) -> Result<String, JsError> {
    let mnemonics: Vec<String> =
        serde_json::from_str(mnemonics_json).map_err(|e| JsError::new(&e.to_string()))?;
    let scalar = slip39::combine_mnemonics(&mnemonics, b"").map_err(err)?;
    let identity = bundle::identity_from_bytes(&scalar).map_err(err)?;
    let files = bundle::decrypt_bundle(&identity, data).map_err(err)?;
    Ok(serde_json::to_string(&files).expect("serializable"))
}
