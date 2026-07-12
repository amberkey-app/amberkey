# CISA Secure by Design pledge — goal-by-goal evidence

Last reviewed: 2026-07-11 · Status: **evidence ready; pledge not yet signed.**
The pledge is a public commitment by the company (Tarnover, LLC) — signing is a
company action (see the email draft at the bottom). This page will change to
"signed" only when CISA lists us.

The pledge asks software manufacturers to show measurable progress on seven
goals within a year. AmberKey's position on each, today:

| # | Goal | AmberKey evidence | Status |
|---|---|---|---|
| 1 | **Increase MFA use** | Authentication is passkey-first and account creation is passkey-only — every account has phishing-resistant MFA from its first second. TOTP exists only as an additional fallback factor. 100% of users, by construction. | ✅ exceeds |
| 2 | **Reduce default passwords** | No passwords exist anywhere in the product — none to default. Deployment images ship with no default credentials; admin access is an explicit env allowlist that defaults to *disabled*. | ✅ by construction |
| 3 | **Reduce entire classes of vulnerability** | Memory-unsafety: all crypto/server code is Rust. SQL injection: parameterized queries exclusively (rusqlite). XSS: React auto-escaping + strict CSPs on all three surfaces, zero third-party scripts. Credential phishing: passkeys are verifier-bound. | ✅ ongoing |
| 4 | **Increase customer installation of patches** | The service is hosted — customers are always on the current version. The one customer-held artifact (the offline recovery tool) is versioned, signed, and mirrored, and the printed cards pin its hash so survivors can verify what they run. | ✅ |
| 5 | **Publish a vulnerability disclosure policy** | `SECURITY.md`: scope, 72-hour acknowledgment, 90-day coordinated disclosure, researcher credit, and an explicit **safe harbor** authorizing good-faith research (added 2026-07-11). | ✅ |
| 6 | **Transparency in vulnerability reporting (CVEs)** | Commitment in `SECURITY.md` (added 2026-07-11): CVEs requested promptly for high/critical issues in released open components, with accurate CWE and affected-version data. None issued yet (none found yet). | ✅ committed |
| 7 | **Evidence of intrusions** | Every account has an append-only audit log the owner can read in-app (logins, check-ins, changes, admin actions), giving customers first-party forensic evidence. Server-side centralized logging is a tracked gap (`ops/SOC2-PHASE0.md`). | ✅ / gap noted |

*Signing is a company action via the current process at
cisa.gov/securebydesign/pledge; this page tracks the evidence, not the paperwork.*
