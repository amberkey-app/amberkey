# DRAFT: Supervised setup ceremony (Pro phase C)

**Status: BLOCKED on A+B** (`spec/pro-console-foundation.md`). Parked so the
decisions do not evaporate; not a buildable spec yet. This is the Phase-2
centerpiece of the Pro product: an advisor-facilitated, client-device setup
ceremony that provisions a client's plan without ever exposing Layer 2 or key
material to the advisor.

## Why this exists

Estate-planning clients paid specifically to not do this themselves and will not
self-serve through share generation, kit printing, and shareholder selection.
The advisor's real-world job is "I make sure this actually gets done," slotting
in right after the will/trust signing. So provisioning, not oversight, is the
commercial spine — but it must stay inside the blind-server model.

## The boundary (= the existing Layer 1 / Layer 2 boundary)

- **Advisor can write:** account shell + configuration, circle definition
  (names, contacts, threshold), executor packet + playbook assignments,
  order/prepare kit materials, schedule the ceremony, manage liveness cadence,
  coordinate the release ceremony.
- **Only the client's device can:** generate the vault key, encrypt the bundle,
  produce the SLIP-39 shares. One supervised session (~20–30 min), clean-kiosk
  mode, everything client-side through the WASM core, nothing persisted but
  ciphertext.

Trust-page sentence: "Your advisor manages everything about your plan except the
secrets. Those never exist anywhere but your device and your shareholders'
cards."

## Load-bearing decisions to preserve (from prior design work)

- **Invariants:** the crypto runs only on the client device; the advisor console
  (per A+B §1.1) contains no crypto and cannot be the ceremony's crypto host.
- **All-shares-in-one-room threat:** the ceremony physically colocates the owner
  and (potentially) shareholders/printer; treat "all shares generated and
  visible in one room at one time" as a threat to bound (e.g. immediate print +
  hand-off, no aggregation, no screen retention).
- **USB-printer kit:** printing shares to a local/USB printer in kiosk mode, not
  over any network path.
- **Abort-voids-everything:** aborting the ceremony invalidates all partial
  state; nothing half-generated persists or is resumable.
- **Non-resumability:** a ceremony is single-session; it cannot be paused and
  resumed later (prevents partial-key exposure windows).
- **Ephemeral ceremony key:** the ceremony uses an ephemeral key that does not
  survive the session.
- **Device-custody flag:** record whose device ran the ceremony (client laptop
  vs firm kiosk iPad) as part of the ceremony record.
- **Ceremony certificate:** the session emits a verifiable certificate the
  advisor can later check against server state (the bundle ciphertext hash from
  the A+B contract is what makes this checkable without seeing content).

## Schema hooks already reserved in A+B

- `engagement.ceremony_ref` (nullable) is the foreign-key surface ceremony
  records will land on. The `configure` and `ceremony-coordinator` consent
  scopes are already enumerated in the A+B consent ladder.

## Hard line to hold (release gate)

Firms will ask for required advisor sign-off on the *release* ceremony ("we ran
the signing, we should confirm the death"). **Refuse.** The advisor is
coordinator and attester, never a required gate: blocking a legitimate release is
a critical bug in our own security policy, and a defunct advisor firm must never
strand a beneficiary.

## Before this becomes buildable

Needs its own brainstorming → spec → plan cycle once A+B lands, covering: the
kiosk/clean-device mode, the ephemeral-key ceremony state machine, the printer
path, the certificate format, and the `configure`/`ceremony-coordinator` consent
write paths.
