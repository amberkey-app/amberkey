# Pro Console Foundation (A+B) — design spec

**Status:** approved design, 2026-07-18. Branch `pro-planner-ui`.
**Scope:** sub-projects A (firm account + consented planner↔client link) and B
(read-only oversight dashboard), specified together because the dashboard is the
consent model made visible and splitting them would let the metadata contract
drift.

This is the first phase of the **Pro product** (estate planners, family offices,
and 3rd-party platforms). The full phase stack, the crypto boundary, and the
release-gate principle live in the Pro product direction note; this spec builds
the foundation the later phases (C provisioning, D bulk kits, E platform API)
inherit. The self-service consumer product ships to the public first, so A+B
must also serve as a **sales demo** against an otherwise empty world.

## 1. Non-negotiable invariants

These are stated up front because they are the load-bearing claims; the rest of
the spec is how we uphold them.

1. **The pro console artifact contains no cryptographic code.** The pro console
   is a distinct build artifact with no WASM module, no key handling, no import
   edge to `web/app/src/wasm` or any crypto path. This is verifiable ("grep the
   bundle") and CI-enforced (§7). Trust-page material.
2. **The metadata contract is the maximal external read surface.** Every read an
   advisor (A/B), and later a platform API (E), can ever perform is the
   allowlist in §4. Default-deny everything else. C may add only *read-only*
   ceremony records to this surface, never write access through it.
3. **Advisor tokens are a distinct audience from owner sessions.** An advisor
   token can never call an owner endpoint, and vice versa. This mirrors, on the
   client, the separate-origin isolation in §6.
4. **Consent is client-granted and client-revocable at any time.** Revocation is
   immediate and logged. There is no advisor-side veto on revocation.
5. **The advisor is never a required release gate.** (Inherited principle; not
   exercised in A+B, but the schema and roles must never make an advisor a
   precondition for a legitimate release.)
6. **The blind server is unchanged.** No plaintext, no Layer 2, no key material
   is exposed to any advisor surface. A+B adds only new metadata projections
   over data the server already holds.

## 2. Goals / non-goals

**Goals**
- A firm/advisor account with its own auth and token audience.
- A consented, client-revocable advisor↔owner relationship (both invite
  directions).
- A read-only dashboard that is a pure projection of the metadata contract.
- An append-only audit trail of advisor reads, from the first request.
- Server-side demo seeding so the console demos before real clients exist.

**Non-goals (deferred to later phases, but schema/room reserved)**
- Any Layer 1 write / provisioning (C).
- Consent scopes beyond `status` (schema enumerates them; only `status` ships).
- Bulk kits / co-brand (D). Platform API + webhooks (E).
- Billing/seat metering beyond what a firm needs to exist (kept minimal).

## 3. Entities & data model

New server tables (SQLite, axum). Names indicative; follow existing migration
style in `crates/server/src/db.rs`.

- **`firm`** — the pro account. `id`, `name`, `brand` (nullable JSON slot for D),
  `created_at`. A firm has one or more advisors.
- **`advisor`** — a seat under a firm. `id`, `firm_id`, `email`, auth material
  (passkey credentials, same primitives as owners but a distinct audience),
  `role` (admin/member of the firm), `created_at`, `revoked`. `email` is unique
  **per firm**, not globally: one person may hold a seat at several firms, so
  sign-in offers every live seat's passkeys for the address and the credential
  used resolves which firm the session belongs to.
- **`engagement`** — the advisor↔owner link. `id`, `firm_id`, `advisor_id`
  (nullable: an engagement can belong to the firm before a specific seat claims
  it), `owner_id`, `consent_scope` (§5), `status` (pending/active/revoked),
  `created_at`, `revoked_at`, and **`ceremony_ref` (nullable, reserved)** — the
  foreign-key surface C's ceremony records will land on, added now so the schema
  never migrates for it.
- **`consent`** — the grant record. `id`, `engagement_id`, `scope`,
  `granted_at`, `revoked_at`, `initiated_by` (owner/advisor). Immutable rows;
  a revoke writes `revoked_at`, a re-grant is a new row.
- **`audit_event`** — append-only. `id`, `firm_id`, `advisor_id`, `owner_id`,
  `action` (e.g. `status.read`, `roster.read`, `invite.sent`, `consent.granted`,
  `consent.revoked`), `at`, minimal context. Never contains client secrets.

All engagement/consent/audit queries are scoped by `firm_id` + `owner_id`
(the same tenant-isolation pattern the owner side already uses).

## 4. The metadata contract (the deliverable)

The enumerated, allowlisted set of fields an advisor token can read at `status`
scope. **This is the actual deliverable of A+B** — the dashboard is just its
projection. It is copied verbatim into `spec/threat-model.md` as the
"advisor read surface" and scopes `SECURITY.md`, because it is the answer to
"what does a compromised advisor account leak."

| Field | Exposed at `status` | Notes |
|---|---|---|
| Client label | ✅ | owner-set string on the engagement; blank unless the owner chose one, and settable/clearable at any time from the owner app (`POST /api/pro/label`, scoped to the owner's own engagement). Not server-inferred PII — the client picks what (if anything) the advisor sees to tell connections apart |
| Liveness state | ✅ | enum only |
| Last-check timestamp | ✅ | time only |
| Coverage / plan-health % | ✅ | derived, no contents |
| Share distribution: confirmed count | ✅ | **count only** |
| Share distribution: pending count | ✅ | **count only** |
| Shareholder identities / contacts | ❌ | requires `status+contacts` (not in B) |
| Bundle age (last export/sync time) | ✅ | metadata |
| Bundle ciphertext hash | ✅ | see non-secret note below |
| Executor-packet completeness flag | ✅ | boolean |
| Playbook-assignment completeness flag | ✅ | boolean |
| Ceremony status | ✅ | enum |
| Advisor's own card still held (if cardholder) | ✅ | boolean |
| Plan contents, letters, card values, any Layer 2 | ❌ | default-deny |
| Member names | ❌ | server never holds them (blind-server property) |

**Why the ciphertext hash is non-secret** (the field a reviewer will squint at):
it is a hash of already-encrypted bundle ciphertext, so it reveals nothing about
plaintext; it is exposed because it lets an advisor later verify a ceremony
certificate against server state (does the bundle the client attests to match
what the server holds) without ever seeing content.

Contract endpoint: a single read projection keyed by engagement, returning only
the ✅ fields for the consented `owner_id`. The projection is the *only* code
path an advisor read can take; there is no generic owner-record accessor on the
advisor audience.

## 5. Consent scope ladder

Enumerated in full in A so the consent schema never migrates when C arrives.
**B implements `status` only**; the rest are defined, reserved, and rejected if
requested.

`status` → `status+contacts` → `configure` (Layer 1 write, C) →
`ceremony-coordinator` (C) → `cardholder` (opt-in per engagement).

Grant/revoke semantics, designed once here:
- Consent is granted by the **client (owner)**, explicitly, per scope.
- **Client-revocable at any time; revocation is immediate** (the next advisor
  read fails closed) **and logged** as an `audit_event`.
- **No advisor-side veto** on revocation.
- `cardholder` is always optional per engagement (some firms prohibit it on
  liability grounds).

## 6. Auth, token audience, and front-end architecture

**Separate build artifact, shared workspace.** Not a `/pro` route inside
`web/app`. Concretely:
- A second entry point / `web/pro` package in the same npm workspace, with its
  own `index.html` and its own bundle.
- A shared component library (design tokens, table/badge/layout primitives)
  consumed by both consoles.
- **No dependency edge from `web/pro` to `web/app/src/wasm` or any crypto path.**
- Served at **`pro.amberkey.app`** — a fourth static surface (one more Caddy
  stanza in dev; one more S3+CloudFront surface + ACM SAN + Route53 record in
  the AWS stack, following the existing `for_each` over surfaces).

**Why separate origin, not just a route:** the owner console's defining property
is that all crypto runs through the WASM core; the pro console's defining
property is that it contains none. That is only a verifiable claim if the pro
console is a distinct artifact. A separate origin additionally gives
browser-level cookie/session isolation between the two audiences (not just token
separation), a clean home for a stricter pro session policy and an eventual SSO
redirect flow without touching owner-console auth, and a URL sales can put on a
slide.

**Advisor auth:** same primitives as owners (passkeys), but tokens carry a
distinct audience claim; the server rejects an advisor token on owner endpoints
and an owner token on advisor endpoints.

**Provisioning:** firms and every advisor seat are created by AmberKey staff in
`/admin` (billing is arranged manually). The advisor console has no firm-creation
path; a seat activates by consuming a single-use, staff-issued invite bound to a
pre-created, credential-less `advisors` row. `advisors.role` is `admin` |
`member`, recorded at provisioning time and not yet permission-bearing.

## 7. Dashboard (B), demo seeding, and the CI boundary

**Dashboard** — a read-only roster of consented clients whose columns are
exactly the `status`-scope contract fields (§4). No filter, sort, or detail view
may surface a field the contract does not name. Health states (e.g. liveness
color, bundle-stale flag, coverage %) are rendered from the contract only.

**Demo seeding** — server-side demo firm + synthetic clients spanning health
states (healthy, stale bundle, liveness-escalating, incomplete packet, pending
shares), plus a demo advisor login. Non-production gated. Because seeding is
server-side, the demo is simply "visit `pro.amberkey.app`, log in as the demo
advisor"; it does not need to live inside the owner SPA to be demoable.

**CI no-crypto-import check** — a lint/CI rule that fails the `web/pro` build if
anything under `wasm/` or the crypto paths is imported. One line, and it makes
invariant §1.1 self-enforcing rather than aspirational — the front-end twin of
"advisor tokens can't call owner endpoints."

## 8. Audit log

Every advisor read of client status is an `audit_event`, **from the first
request** — built into the engagement primitive from day one because it is
nearly impossible to retrofit honestly ("our audit trail starts three months
after launch" is a bad sentence for an RIA compliance officer). Events cover
reads, invites, and consent grant/revoke. Surfaced to the firm; a later
decision covers whether/how it is mirrored to the client (the owner-side
`audit_log` already exists as a precedent).

## 9. Security / threat-model updates (part of this work)

- Add the §4 contract table to `spec/threat-model.md` as the "advisor read
  surface," with a **"compromised advisor account"** entry: it leaks only the
  contract fields for *consented* clients, cannot write, cannot read Layer 2 or
  keys, cannot act as an owner, and cannot block a release.
- Scope note in `SECURITY.md` referencing the contract as the maximal external
  read surface.
- Keep the security page summary consistent (per the threat-model's own rule).

## 10. Downstream inheritance (captured, not built here)

- **C (provisioning ceremony):** parked in `spec/drafts/setup-ceremony.md`,
  marked *blocked on A+B*. `engagement.ceremony_ref` is its landing spot.
- **D (bulk kits / co-brand):** co-brand kit templating is an open-core
  (Apache-2.0, `kit` feature in `crates/core`) vs proprietary-wrapper licensing
  boundary decision, not just a billing feature — a paragraph for D's spec.
- **E (platform API, e.g. Summitas):** exposes exactly this metadata contract
  plus webhook events, and from C nothing but read-only ceremony records. The
  "maximal external read surface" line (§1.2) exists so E cannot scope-creep.

## 11. Open decisions (not blocking the plan)

- Firm↔seat multiplicity: RESOLVED — many seats per firm, all staff-provisioned;
  firm-admin self-service seat management is deferred to a later phase.
- Billing/seat metering: kept minimal for A+B; real per-seat pricing is a C-era
  concern.

## 12. Testing strategy

- **Contract allowlist test:** the advisor read projection returns only §4 ✅
  fields; a default-deny assertion fails if any non-listed field appears.
- **Token audience test:** advisor token rejected on owner endpoints and owner
  token rejected on advisor endpoints.
- **Consent revocation test:** a revoked consent fails the next advisor read
  closed, immediately, and writes an `audit_event`.
- **Audit emission test:** every advisor read emits exactly one event.
- **CI no-crypto-import check:** the `web/pro` build fails if a crypto/wasm
  import is introduced.
