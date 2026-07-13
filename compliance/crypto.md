# Cryptography conformance

Last reviewed: 2026-07-11

The most meaningful compliance claims AmberKey makes are specification
conformance claims about its cryptography, because they are testable and the
tests are public. Everything below is checkable in this repository.

## What we conform to

| Primitive / format | Specification | Where | Verification |
|---|---|---|---|
| Vault encryption | **age v1** file format | `crates/core` (via the `age` crate) | round-trip + recovery tests in CI; format documented in `spec/bundle-format.md` |
| Key agreement | **X25519** — RFC 7748 | inside age | upstream `age`/`x25519-dalek` test suites |
| Payload encryption | **ChaCha20-Poly1305** — RFC 8439 | inside age | upstream test suites |
| Passphrase KDF (age scrypt recipients, unused by default) | **scrypt** — RFC 7914 | inside age | upstream |
| Key splitting | **SLIP-39** (Shamir's Secret-Sharing for Mnemonic Codes) | `crates/core/src/slip39*` — in-house implementation | **the complete official Trezor test-vector suite runs in CI on every commit**; `cargo test --workspace --features kit` |
| Content addressing | **SHA-256** — FIPS 180-4 | bundle/blob hashing, session tokens | upstream `sha2` crate |
| Release signatures | **Ed25519** (minisign format) | release artifacts, `recover.html` | verifiable with the published minisign key; fingerprint printed on every share card |
| Sign-in | **WebAuthn Level 2 / FIDO2** | `webauthn-rs` server-side | see [nist-800-63b.md](nist-800-63b.md) |

The SLIP-39 claim is the strongest one: it is an independent implementation
gated on the reference vectors, so "conforms to SLIP-39" means any standard
SLIP-39 tool can read AmberKey's cards — which is the product's whole exit
guarantee. The recovery tool is additionally built reproducibly (CI builds it
twice and compares hashes) and its SHA-256 is printed on the physical cards.

## What we deliberately do not claim

- **Not "FIPS validated" and not claimed to be.** FIPS 140-3 validation applies
  to specific tested modules under CMVP; no AmberKey component has been
  submitted. Moreover, X25519 and ChaCha20-Poly1305 are **IETF-standardized**
  algorithms, not FIPS-approved ones — so the honest description of AmberKey's
  cryptography is *"IETF-standardized cryptography with public test vectors"*,
  and we skip FIPS language entirely.
- **No home-rolled primitives.** The only in-house cryptographic code is the
  SLIP-39 share arithmetic (which has a public reference test suite — see
  above); ciphers, curves, and hashes come from widely-reviewed Rust crates.
- **No third-party cryptographic audit yet.** A professional review of
  `crates/core` is the planned escalation once revenue justifies it; until then
  these are self-assessed conformance claims backed by public CI.
