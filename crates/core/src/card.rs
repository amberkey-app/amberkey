//! Shared data models: account cards (packet), circle directory, printed share cards.
//! Serialized shapes are part of the bundle format spec — change with care.

use serde::{Deserialize, Serialize};

/// Account card, one per declared account (seedplan section 7.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountCard {
    pub id: String,
    pub service: Service,
    pub label: String,
    pub strategy: Strategy,
    #[serde(default)]
    pub metadata: AccountMetadata,
    #[serde(default)]
    pub native_config: NativeConfig,
    #[serde(default)]
    pub layer2_refs: Vec<String>,
    #[serde(default)]
    pub playbook_ref: String,
    #[serde(default)]
    pub executor_instructions_md: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Service {
    Google,
    Apple,
    Bitwarden,
    #[serde(rename = "1password")]
    OnePassword,
    #[serde(rename = "keepass")]
    KeePass,
    #[serde(rename = "lastpass")]
    LastPass,
    BrowserPasswords,
    Bank,
    CreditCard,
    CryptoWallet,
    Photos,
    Email,
    PhoneCarrier,
    Generic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    Native,
    Layer2Secret,
    LegalProcess,
    DeviceMediated,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountMetadata {
    #[serde(default)]
    pub institution: String,
    #[serde(default)]
    pub last4: String,
    #[serde(default)]
    pub notes_md: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NativeConfig {
    #[serde(default)]
    pub mechanism: String,
    #[serde(default)]
    pub designees: Vec<String>,
    #[serde(default)]
    pub delay: String,
    #[serde(default)]
    pub last_attested: String,
}

/// A Layer 2 secret item. Lives only inside the encrypted bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretItem {
    pub id: String,
    pub label: String,
    /// e.g. "crypto_seed", "master_password", "apple_access_key", "device_passcode", "port_pin"
    pub kind: String,
    pub value: String,
    #[serde(default)]
    pub notes_md: String,
}

/// circle.json — the sealed-envelope shareholder directory inside the bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircleDirectory {
    pub group_threshold: u8,
    pub groups: Vec<CircleGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircleGroup {
    pub name: String,
    pub member_threshold: u8,
    pub members: Vec<CircleMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircleMember {
    pub name: String,
    #[serde(default)]
    pub relationship: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub phone: String,
    pub card_id: String,
}

/// Data printed on one physical share card (spec/share-card.md).
/// Deliberately excludes owner name, holder name, and other holders' identities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareCard {
    /// e.g. "AK1-G2-M3": case number, group index, member index. Case number is
    /// random and shared by all cards of one share set.
    pub card_id: String,
    pub case_number: String,
    pub mnemonic: String,
    pub recovery_url: String,
    pub tool_hash: String,
    pub minisign_fingerprint: String,
}

impl ShareCard {
    pub fn card_id(case_number: &str, group_index: u8, member_index: u8) -> String {
        format!("{case_number}-G{}-M{}", group_index + 1, member_index + 1)
    }
}
