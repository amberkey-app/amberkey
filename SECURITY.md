# Security Policy

## Reporting a vulnerability

Email **security@amberkey.app**. You should hear back within 72 hours.

Please include reproduction steps and impact. We ask for up to 90 days of
coordinated disclosure; we will credit you (or keep you anonymous, your call)
in the release notes that ship the fix.

## Scope

Everything AmberKey ships — the open recovery components in the public repo
and the hosted service behind amberkey.app — with special interest in:

- the SLIP-39 implementation (`crates/core/src/slip39/`) — any deviation from
  the spec or the Trezor test vectors is a critical bug
- the continuity bundle format and its age usage (`crates/core/src/bundle.rs`)
- the offline recovery tool (`web/recovery-tool/`) — it must make zero network
  requests and be reproducible
- the liveness and release-ceremony logic — anything that could release a
  bundle early, or block a legitimate release
- shareholder link endpoints (auth-less by design; rate limited)

## What the server can never do

The server holds only ciphertext, contact routing, and schedules. There is no
key escrow. A full server compromise leaks metadata (who, when, how many
shareholders) — documented in `spec/threat-model.md` — but no vault contents.

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
