# Compliance self-assessments

Self-assessed mappings of AmberKey's controls to public frameworks, published so
anyone can check the claims against the code and CI in this repository. The
human-readable summary lives at <https://amberkey.app/trust>.

## Language policy

Precision is the point. Throughout these documents and the website:

- **"conforms to"** — a testable specification, with the tests public (e.g. the
  SLIP-39 Trezor vectors run in CI).
- **"aligned with"** — a framework we self-assessed against; the mapping is the
  artifact. No third party has audited it and we don't imply one has.
- **"certified" / "validated" / "compliant"** — used **never**, unless an
  accredited third party has actually issued that status (none has; see the
  "deliberately not claimed" list on the trust page).

Every document carries a `Last reviewed` date. If a claim and the code disagree,
the code wins and the document is wrong — please report it (SECURITY.md).

## Contents

| Document | Framework | Claim style |
|---|---|---|
| [crypto.md](crypto.md) | IETF RFCs, SLIP-39, age v1 | conforms to (CI-tested) |
| [nist-800-63b.md](nist-800-63b.md) | NIST SP 800-63B authentication | aligned with (self-assessed) |
| [ssdf.md](ssdf.md) | NIST SP 800-218 SSDF | aligned with (self-assessed) |
| [nist-csf.md](nist-csf.md) | NIST CSF 2.0 profile | aligned with (self-assessed) |
| [cis-ig1.md](cis-ig1.md) | CIS Controls v8 IG1 | aligned with (self-assessed, gaps named) |
| [slsa.md](slsa.md) | SLSA v1 provenance | SLSA-informed (signed, self-hosted builder) |
| [cisa-sbd.md](cisa-sbd.md) | CISA Secure by Design pledge | evidence ready; signature pending |
| [openssf-badge.md](openssf-badge.md) | OpenSSF Best Practices badge | answers ready; registration pending |
| [asvs.md](asvs.md) | OWASP ASVS 4.0 Level 2 | in progress (V2/V3/V4 done) |
