//! AmberKey crypto core.
//!
//! Single source of truth for all AmberKey cryptography:
//! - SLIP-39 secret sharing (in-house, gated on the official Trezor test vectors)
//! - age (v1) continuity bundle encryption
//! - share card and packet data models
//!
//! Compiled to WASM for the browser (web app + offline recovery tool) and
//! native for `amberkey-recover`. No crypto exists outside this crate.

pub mod bundle;
pub mod card;
pub mod error;
#[cfg(feature = "kit")]
pub mod kit;
pub mod slip39;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use error::Error;

/// Long-term recovery tool URL printed on share cards. One constant; change here only.
pub const RECOVERY_URL: &str = "https://recover.amberkey.app";
/// Marketing / product domain.
pub const MARKETING_URL: &str = "https://amberkey.app";
