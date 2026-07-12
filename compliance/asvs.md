# OWASP ASVS 4.0.3 — Level 2 self-assessment

Last reviewed: 2026-07-11 · Status: **chapters V2, V3, V4 itemized below** (80
L2-applicable requirements verified against the code); remaining chapters (V1,
V5, V7–V14) pending. No overall ASVS claim is made until all chapters are
complete — but the verified chapters, including every gap found, are published
as we go. Gaps are the point of doing this.

**Fixes applied same-day from this pass (2026-07-11):**

- `FreshAdmin` gate reordered — admin-route existence no longer leaks to
  non-admins via a step-up response (V4 finding below).
- Session cookies renamed to the `__Host-` prefix (3.4.4 → now Met).
- TOTP enrollment now requires step-up re-auth (mitigates the 2.5.5 finding — a
  hijacked month-old session can no longer silently swap the fallback factor).

**Second remediation round (2026-07-12):** email notifications now fire on
authenticator and contact changes (2.2.3/2.5.5) · beta invites expire after 30
days (2.3.1) · sessions carry a hard 90-day ceiling regardless of rolling
(bounds 3.3.2) · "Sign out everywhere" revokes all sessions (3.3.4 partial).

**Third round (2026-07-12): TOTP seeds sealed at rest** — ChaCha20-Poly1305
with a key held only in the environment (SSM SecureString in prod), bound to
the owning user id, legacy rows migrated at startup (2.8.2 → Met; closes the
assessment's last outright gap).

**Still open (tracked):** a per-session management list; revoking other
sessions automatically on a factor change; a stronger KDF for the duress code;
requiring a passkey (not the TOTP fallback) for admin sessions; and reducing
capability-link exposure in request logs. Each is a hardening item, not a known
exploit — stated so buyers can see the roadmap.

---

## V2 Authentication

| ID | Requirement (condensed) | Verdict | Evidence |
|---|---|---|---|
| 2.1.1–2.1.12 | Password security (length, complexity, breach checks, paste, rotation, strength meter) | ➖ N/A | No passwords exist anywhere; authentication is passkey (WebAuthn) with a TOTP fallback — there is no password field to attack |
| 2.2.1 | Anti-automation against credential brute force is effective | ✅ Met | TOTP sign-in and re-auth are rate-limited per account with a fixed window; passkeys are inherently unguessable; codes are single-use. An edge WAF adds volumetric rate limiting in production |
| 2.2.2 | Weak authenticators (SMS/email) restricted to secondary use only | ✅ Met | SMS/email are used for notifications only, never accepted as an authentication factor |
| 2.2.3 | Secure notifications sent after authentication-detail changes | ✅ Met | **Fixed 2026-07-12**: email to the account address on TOTP enrol/replacement and on phone change/removal; email changes already warned the old inbox |
| 2.3.1 | Initial activation codes random, expire quickly | ✅ Met | 256-bit CSPRNG, hashed, single-use, and — **fixed 2026-07-12** — valid only 30 days from mint |
| 2.3.2 | Enrollment/use of user-provided authentication devices supported | ⚠️ Partial | Passkeys (FIDO2) are the primary authenticator, but only one is enrolled at registration — no route exists to add a second passkey/device later, though the `credentials` schema supports it |
| 2.3.3 | Timely renewal instructions for expiring authenticators | ➖ N/A | Passkeys and TOTP secrets do not expire |
| 2.4.1–2.4.5 | Password storage: bcrypt/argon2/PBKDF2 parameters, salting, peppering | ➖ N/A | No passwords stored (see duress-code caveat in gaps below) |
| 2.5.1 | Initial/recovery secret not sent in clear text | ✅ Met | No recovery secrets exist; invite links carry a registration gate token only, and the TOTP secret is returned once over the authenticated session |
| 2.5.2 | No password hints or knowledge-based answers | ✅ Met | No hints/KBA anywhere in the codebase |
| 2.5.3 | Credential recovery does not reveal current credential | ✅ Met | No credential-recovery flow exists; nothing to reveal |
| 2.5.4 | No shared or default accounts (root/admin) | ✅ Met | No seeded accounts; admin is an env allowlist over ordinary accounts, disabled when `AMBERKEY_ADMIN_EMAILS` is empty |
| 2.5.5 | Notification when authentication factor changed or replaced | ✅ Met | Step-up required (2026-07-11) **and** the account email is notified on every factor change (2026-07-12) |
| 2.5.6 | Forgotten-credential recovery uses secure mechanism | ➖ N/A | Deliberately no recovery flow (server-blind design); losing passkey + TOTP is permanent lockout |
| 2.5.7 | Lost OTP/MFA replacement requires equivalent identity proofing | ➖ N/A | No lost-factor replacement flow exists |
| 2.6.1–2.6.3 | Look-up secrets (recovery codes) single-use, random, resistant to offline attack | ➖ N/A | Look-up secrets not used |
| 2.7.1–2.7.6 | Out-of-band verifiers (SMS/push) requirements | ➖ N/A | SMS/email never used as authentication verifiers, notification only |
| 2.8.1 | OTPs have a defined, short lifetime | ✅ Met | 30-second TOTP steps, ±1 step skew only |
| 2.8.2 | Symmetric OTP verification keys highly protected | ✅ Met | **Fixed 2026-07-12**: seeds sealed with ChaCha20-Poly1305 under an environment-held key (SSM SecureString in prod), AEAD-bound to the user id; legacy rows migrated at startup. Stated caveat: a compromised live process can still verify codes — inherent to TOTP, and why passkeys are the primary factor |
| 2.8.3 | Approved crypto for OTP generation/verification | ✅ Met | RFC 6238 TOTP via `totp_lite` (SHA-1, 6 digits, 30s) — the approved TOTP construction |
| 2.8.4 | Time-based OTP usable only once within validity period | ✅ Met | `totp_last_step` replay rejection: a matched step must exceed the last accepted step |
| 2.8.5 | OTP reuse rejected, logged, and holder notified | ⚠️ Partial | Reuse is rejected but the rejection is neither audit-logged nor notified to the holder |
| 2.8.6 | Physical OTP generator revocable on loss/theft | ➖ N/A | No hardware OTP tokens; TOTP is app-based (note: no owner-facing TOTP unenroll endpoint exists, only overwrite via re-enrol) |
| 2.9.1 | Cryptographic-device keys protected against disclosure | ✅ Met | Passkey private keys never leave the client authenticator; server stores public-key material only |
| 2.9.2 | Challenge nonce ≥64 bits, statistically unique | ✅ Met | Challenges generated by `webauthn-rs`; challenge state server-side, single-use, 10-min expiry |
| 2.9.3 | Approved cryptographic algorithms for verification | ✅ Met | `webauthn-rs` default suite (ES256 et al.), origin + RP ID enforced |
| 2.10.1 | Intra-service secrets don't rely on unchanging credentials | ✅ Met | Stripe/SMTP/Twilio keys injected via env at deploy, rotatable without code change |
| 2.10.2 | Service accounts not default credentials | ✅ Met | No defaults; features disable themselves when env unset |
| 2.10.3 | Service passwords stored resisting offline recovery | ✅ Met | Production: SSM Parameter Store SecureString (KMS-encrypted), injected at task start; nothing secret on disk |
| 2.10.4 | Secrets/keys/seeds not in source, managed via secret store | ✅ Met | Nothing hardcoded (verified); operational secrets in SSM SecureString; TOTP seeds now sealed at rest (2.8.2) |

**Notable gaps (V2):**

- No security notifications at all: enrolling/replacing TOTP, changing phone,
  or new sign-ins never email the owner (2.2.3, 2.5.5). Highest-value quick win.
- The duress code is a short memorized secret; hardening its at-rest storage
  (a slow KDF) is queued. Impact is bounded by design — the code only raises a
  silent flag, it unlocks nothing.
- Beta invites never expire (2.3.1); no way to enrol a second passkey or remove
  TOTP after registration (2.3.2 / 2.8.6 note).
- Application rate limiting is per-instance; the production edge WAF provides
  the durable outer limit.

## V3 Session Management

| ID | Requirement (condensed) | Verdict | Evidence |
|---|---|---|---|
| 3.1.1 | Session tokens never revealed in URL parameters | ⚠️ Partial | Owner sessions are cookie-only. Shareholder access uses single-purpose capability links (necessarily carried in a link), stored only as a hash, expiring, single-use for attestation, and rate-limited |
| 3.2.1 | Fresh session token generated on every authentication | ✅ Met | Every login/registration mints a new 256-bit token; no pre-auth session exists to fixate; pending-challenge cookie is separate and removed |
| 3.2.2 | Session tokens possess ≥64 bits entropy | ✅ Met | 256-bit tokens from the OS CSPRNG |
| 3.2.3 | Tokens stored in browser via secure methods | ✅ Met | HttpOnly, Secure, SameSite=Strict, Path=/ cookie; server stores only SHA-256 of the token |
| 3.2.4 | Tokens generated with approved cryptographic algorithms | ✅ Met | `getrandom` (OS CSPRNG) |
| 3.3.1 | Logout and expiration invalidate token server-side | ✅ Met | Logout deletes the DB row; every request checks `expires_at > now`; expired rows purged on session creation |
| 3.3.2 | Re-authentication after idle/absolute period (L2: 12h or 30min idle) | ⚠️ Partial (documented deviation) | 30-day rolling session — deliberate product decision (missed check-ins are the product's dangerous failure mode), documented in `nist-800-63b.md`, compensated by the 10-min step-up window. **Since 2026-07-12** a hard 90-day ceiling ends any session regardless of activity; still far beyond the L2 figure, hence Partial, on purpose |
| 3.3.3 | Terminate all other sessions after credential change | ⚠️ Partial | Admin disable kills all of a user's sessions immediately; but TOTP enrol/replacement terminates nothing, and there is no user "log out everywhere" |
| 3.3.4 | Users can view and terminate their active sessions/devices | ⚠️ Partial | **"Sign out everywhere"** (Settings) revokes every session, audited (2026-07-12); a per-session list with selective revocation is still to come |
| 3.4.1 | Cookie-based tokens set Secure | ✅ Met | Always, including dev (localhost exemption noted in code) |
| 3.4.2 | Cookie-based tokens set HttpOnly | ✅ Met | HttpOnly set on the session cookie |
| 3.4.3 | Cookie-based tokens use SameSite | ✅ Met | `SameSite=Strict` |
| 3.4.4 | Cookies use the "__Host-" name prefix | ✅ Met | **Fixed 2026-07-11**: cookies renamed `__Host-ak_session` / `__Host-ak_pending` (attributes already qualified) |
| 3.4.5 | Precise path scoping when sharing a domain with other apps | ➖ N/A | The app is the sole application on its host; Path=/ is correct |
| 3.5.1 | Users can revoke OAuth/SSO trust relationships | ➖ N/A | No OAuth/SSO |
| 3.5.2 | Session tokens used rather than static API secrets | ✅ Met | Cookie sessions for owners; shareholder links are expiring, hashed, purpose-scoped capabilities; Stripe webhook is HMAC-signed with a 5-min replay window |
| 3.5.3 | Stateless tokens use signatures/encryption against tampering | ➖ N/A | Sessions are stateful (DB-backed, hash-verified); no stateless tokens issued |
| 3.7.1 | Full/fresh authentication required for sensitive transactions | ✅ Met | Sensitive changes (recovery circle, liveness schedule, duress code, contact details, authenticator enrollment, and all destructive admin actions) require a recent re-authentication; the re-auth challenge is bound to the session's user |

**Notable gaps (V3):**

- The rolling session (3.3.2) is now bounded by a 90-day absolute ceiling —
  the remaining deviation from the L2 figure is deliberate and documented.
- "Sign out everywhere" exists; the per-session list (3.3.4) does not yet.
- One-time-link tokens in URL paths (3.1.1) appear in reverse-proxy access
  logs; consider logging path prefixes only for `/api/link/` at the proxy.

## V4 Access Control

| ID | Requirement (condensed) | Verdict | Evidence |
|---|---|---|---|
| 4.1.1 | Access control enforced on a trusted (server) layer | ✅ Met | All authorization lives in server extractors: `Owner`, `FreshOwner`, `Admin`/`FreshAdmin`; the client `is_admin` flag is display-only |
| 4.1.2 | Access-control attributes not manipulable by end users | ✅ Met | Identity derives solely from the hashed session token; admin role from the env allowlist with no in-app grant surface; disabled accounts rejected in the session query itself |
| 4.1.3 | Least privilege; deny by default | ✅ Met | Every handler takes an extractor; the only unauthenticated surfaces are the signed Stripe webhook, the rate-limited newsletter form, and token-capability link endpoints |
| 4.1.5 | Access controls fail securely, including on exceptions | ✅ Met | Extractor failure returns 401/404 before the handler runs; DB no-rows maps to NotFound, other errors to an opaque 500 |
| 4.2.1 | Sensitive data protected against IDOR | ✅ Met | Every data query is scoped to the session's owner id (blobs, audit, circle, liveness); blob fetch requires owner_id AND hash; link endpoints scope through the token's member→owner join; bundle download additionally requires released-ceremony participation |
| 4.2.2 | CSRF protection on authenticated functionality | ✅ Met | `SameSite=Strict` on all cookies blocks cross-site sends; state changes are JSON-body fetches with same-origin credentials; WebAuthn origin binding protects auth ceremonies |
| 4.3.1 | Administrative interfaces use appropriate multi-factor authentication | ⚠️ Partial | Admin access requires an allowlisted account (others get 404) and a recent re-authentication for any state-changing action. Mandating the passkey factor (rather than allowing the TOTP fallback) for admin sessions is a queued hardening item |
| 4.3.2 | Directory browsing disabled; no file metadata leakage | ✅ Met | Static serving has no listings; no `.git`/metadata exposure paths in the API |
| 4.3.3 | Step-up authorization / segregation of duties | ✅ Met | Two-tier admin gate: reads `Admin`, destructive/financial `FreshAdmin` + audited; owner-side sensitive actions step-up-gated |

**Notable gaps (V4):**

- ~~An admin-only route could reveal its existence to a non-admin holding a
  stale session (a step-up prompt instead of a 404)~~ — **fixed 2026-07-11**;
  non-admins now always receive 404.
- Admin over TOTP-only sign-in (4.3.1): requiring a passkey-authenticated
  session for the admin surface is queued.
- The health-check action (sends emails, mints attestation links) runs with a
  normal authenticated session while editing the same circle needs step-up;
  non-destructive, but a deliberate decision worth revisiting.

## Summary

| Chapter | ✅ Met | ➖ N/A | ⚠️ Partial | ❌ Gap | Total (L2-applicable) |
|---|---|---|---|---|---|
| V2 Authentication | 20 | 30 | 2 | 0 | 52 |
| V3 Session Management | 11 | 3 | 4 | 0 | 18 |
| V4 Access Control | 8 | 0 | 1 | 0 | 9 |
| **Total** | **39** | **33** | **7** | **0** | **79** |

(Counts reflect the three same-day fixes; 4.1.4 excluded as deleted in 4.0.3.)
