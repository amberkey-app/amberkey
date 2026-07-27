#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid mnemonic word: {0}")]
    InvalidWord(String),
    #[error("invalid mnemonic length")]
    InvalidLength,
    #[error("invalid mnemonic checksum")]
    InvalidChecksum,
    #[error("invalid mnemonic padding")]
    InvalidPadding,
    #[error("invalid master secret length (must be >= 16 bytes and even)")]
    InvalidSecretLength,
    #[error("invalid group parameters: {0}")]
    InvalidGroupParams(String),
    #[error("mnemonics do not belong to the same share set ({0} differs)")]
    MismatchedShareSet(&'static str),
    #[error("mismatching member thresholds within a group")]
    MismatchedMemberThreshold,
    #[error("two shares with the same member index carry different values")]
    DuplicateMemberIndex,
    #[error("insufficient shares: {got} complete group(s), {needed} needed")]
    InsufficientGroups { got: usize, needed: usize },
    #[error("share digest verification failed (a share is corrupted or from a different set)")]
    InvalidDigest,
    #[error("passphrase must be printable ASCII")]
    InvalidPassphrase,
    #[error("duplicate share x-coordinate")]
    DuplicateShareIndex,
    #[error("age encryption error: {0}")]
    Age(String),
    #[error("invalid age identity or recipient: {0}")]
    BadKey(String),
    #[error("bundle format error: {0}")]
    Bundle(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
