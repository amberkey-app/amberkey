# Changelog — open components

Covers the published subset of AmberKey: the crypto core (`crates/core`), the
recovery CLI, the offline recovery tool, the format specs (`spec/`), and the
compliance self-assessments (`compliance/`). The hosted service changes on the
same tags but is not documented here.

Format: [Keep a Changelog](https://keepachangelog.com/) · versions are git
tags; container images and provenance are published per tag.

## [Unreleased]

### Added
- Signed build provenance from CI: an in-toto/SLSA v1 statement per release
  naming the image digests, `recover.html` SHA-256, and source commit —
  SSHSIG-signed, verification steps in `compliance/slsa.md`.
- Compliance self-assessments: OWASP ASVS 4.0.3 L2 chapters V2/V3/V4 itemized
  (`compliance/asvs.md`), CIS Controls v8 IG1 (`compliance/cis-ig1.md`).

### Security
- Admin route existence no longer leaks to non-admins holding stale sessions
  (authorization check reordered ahead of the re-auth freshness check).
- Session cookies renamed to the `__Host-` prefix (locks them to the exact
  host; signs everyone out once).
- Enrolling or replacing the TOTP fallback factor now requires step-up
  re-authentication.

## [0.5.0] — 2026-07-11

### Added
- `compliance/` published: language policy (conforms-to / aligned-with /
  never "certified"), cryptography conformance statement, NIST SP 800-63B and
  SSDF 800-218 mappings, CSF 2.0 profile, CISA Secure-by-Design evidence,
  OpenSSF badge answer sheet.
- `SECURITY.md`: explicit safe harbor for good-faith research and a CVE
  commitment for high/critical issues in released open components.
- `CONTRIBUTING.md` now ships in this repo (reporting channels, public-mirror
  PR flow, tests-required policy).

### Changed
- The Apache-2.0 license file is now named `LICENSE` (was `LICENSE-APACHE`).

## [0.4.0] — 2026-07-09

### Added
- Playbooks: LastPass (Emergency Access), first-class KeePass, and a
  "no password manager" fork; strengthened Google and Apple guidance
  (Google Password Manager CSV caveats; iCloud Keychain is not covered by
  Apple's Legacy Contact).

## [0.3.0] — 2026-07-09

### Fixed
- Print kit: the sealed directory page now folds so the sensitive side seals
  correctly.

## [0.2.0] — 2026-07-09

### Added
- Generic email-account service type + playbook.
- Discreet safety-code (duress) check-in path.
- TOTP enrollment QR code.
- Public mirrors: Software Heritage snapshots and the Internet Archive item
  (archive.org/details/amberkey).

### Changed
- Print kit: holder names moved off the printed share sheets (owner-only
  distribution page instead).
- Recovery-tool WASM built without fat LTO — its link-order nondeterminism
  broke reproducible builds; honest reproducibility wording in the spec.

## [0.1.0] — 2026-07-08

### Added
- Crypto core: in-house SLIP-39 implementation gated on the complete official
  Trezor test-vector suite; age v1 bundle format; WASM bindings; `recover-cli`.
- `recover.html`: single-file offline recovery tool (zero network requests,
  reproducible build, minisign-signed).
- Format specs: bundle format, recovery procedure, share card, threat model.
- Print-at-home kit (PDF) and verified playbook content.
