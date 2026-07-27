# Security Policy

Web version: <https://amberkey.app/security/disclosure> ·
Machine-readable: <https://amberkey.app/.well-known/security.txt>

## Reporting a vulnerability

Email **security@amberkey.app**. You should hear back within 72 hours.

PGP is optional. Key: <https://amberkey.app/.well-known/pgp-key.txt>, fingerprint
`0716 9DA7 82C4 FA41 D1EB  DFCA 1F4F 6870 6D45 AE05` (ed25519, valid to
2029-07-25). Verify the fingerprint before trusting a copy from anywhere else.

Please include reproduction steps and impact. We ask for up to 90 days of
coordinated disclosure; we will credit you (or keep you anonymous, your call)
in the release notes that ship the fix.

## Scope

Everything AmberKey ships — the open recovery components in the public repo
and the hosted service behind amberkey.app — with special interest in:

- the SLIP-39 implementation (`crates/core/src/slip39/`) — any deviation from
  the spec or the Trezor test vectors is a critical bug
- the family backup format (the encrypted `bundle`) and its age usage (`crates/core/src/bundle.rs`)
- the offline recovery tool (`web/recovery-tool/`) — it must make zero network
  requests and be reproducible
- the liveness and release-ceremony logic — anything that could release a
  bundle early, or block a legitimate release
- shareholder link endpoints (auth-less by design; rate limited)

## What the server can never do

The server holds only ciphertext, contact routing, and schedules. There is no
key escrow. A full server compromise leaks routing metadata (contact
email/phone, timestamps, shareholder count) — but never member names, keys,
or vault contents. The exact leak set is documented in `spec/threat-model.md`.

**Pro console (advisors).** When an owner consents, an estate planner can read a
**read-only metadata projection** — the allowlisted "advisor read surface" in
`spec/threat-model.md` (T10) is the *maximal external read surface*: liveness,
coverage, share counts, bundle age/hash, completeness flags. Never plan contents,
secrets, keys, or member names. Consent is client-revocable immediately; advisor
tokens are a distinct audience from owner sessions; the advisor console ships
with no cryptographic code. Later phases (provisioning, platform API) inherit
this same surface and add nothing beyond read-only ceremony records.

## Out of scope

Volume-metric denial of service, social engineering of shareholders (mitigated
by design: cards alone unlock nothing), and issues in third-party dependencies
already fixed upstream (report those upstream; tell us so we bump).

## Safe harbor

We authorize good-faith security research on AmberKey and will not pursue or
support legal action (including under the CFAA or DMCA §1201) against
researchers who:

- test against **their own accounts** or the dev instance
  (`dev.amberkey.app` / `app.dev.amberkey.app`), never other people's data —
  if you encounter someone else's data, stop and report immediately;
- avoid degrading the service for others (no volumetric DoS);
- give us a reasonable window to fix (the 90-day coordinated disclosure above)
  before publishing.

Research consistent with this policy is considered authorized. If in doubt
about scope, ask first: security@amberkey.app.

## CVEs

For vulnerabilities of high or critical severity in the released open
components, we will request a CVE ID promptly once a fix is available, with
accurate CWE classification and affected-version (CPE) data, and credit the
reporter in the advisory unless they prefer otherwise.
