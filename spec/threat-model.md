# AmberKey Threat Model

**Status:** living document. This is the canonical threat model; the marketing
site's [security page](../site/src/pages/security.astro) renders a summary of
this matrix and must be kept consistent with it. Per seedplan §10, every
feature PR that touches ceremony or liveness logic updates this file.

## System summary (what an attacker faces)

- The **vault master key** is an age X25519 identity generated client-side.
  Its raw 32-byte scalar is split with SLIP-39 (group thresholds) across the
  owner's recovery circle as printed cards. See `spec/bundle-format.md` for
  the exact encoding and parameters.
- The **continuity bundle** is a single age-v1-encrypted tar containing the
  executor packet (Layer 1), bearer secrets (Layer 2), the circle directory,
  and a playbook snapshot.
- The **server is blind**: it stores only (a) bundle ciphertext, (b) liveness
  state and schedules, (c) circle contact routing (names as entered, email/
  phone, card IDs, group assignments), (d) ceremony state and an append-only
  audit log. It never receives keys, shares, share words, or plaintext.
- **Liveness:** `ACTIVE → NUDGE(7d) → SMS(7d) → TRUSTED_CONTACT_QUERY(7d) →
  QUORUM_ELIGIBLE`, base check-in interval 30/60/90 days (owner choice). Any
  owner check-in in any state returns to ACTIVE. QUORUM_ELIGIBLE only permits
  ceremony *initiation*; it releases nothing.
- **Ceremony:** any holder may initiate when eligible (or immediately with a
  death-certificate claim, unverified in MVP). All parties including the
  owner are notified; a veto window of 3–14 days (default 7) follows; owner
  check-in or veto cancels and resets liveness. After quorum confirmation by
  group thresholds, the server releases the **encrypted bundle only**.
  Reconstruction happens offline with the printed cards and `recover.html`
  (or `amberkey-recover`).

## Security goals

1. **Confidentiality until death:** no party — including AmberKey — can read
   Layer 2 (or the bundle at all) without a quorum of physical cards.
2. **Availability after death:** a real death eventually yields recovery,
   even if AmberKey no longer exists.
3. **Owner supremacy while alive:** a living, reachable owner can always halt
   a release.
4. **Graceful provider failure:** loss of AmberKey degrades recovery to
   slower/manual, never to impossible.

These goals tension against each other (1 vs 2 especially); each threat entry
below states which trade-off was taken and what risk remains. A threat model
without residual risks is marketing.

---

## T1. Premature or coerced reconstruction

**Attack.** Circle members collude — or are coerced by an outsider — to open
the vault while the owner is alive. Variants: a single malicious holder; a
malicious household; kidnapping/duress against the owner.

**Mitigations.**

- SLIP-39 group thresholds: no single holder, and (if the correlated-death
  check is heeded) no single household, holds a quorum.
- Every ceremony initiation notifies the owner on all channels and all
  holders; there are no silent ceremonies.
- Veto window (3–14 days, default 7): one owner check-in cancels the ceremony
  and resets liveness to ACTIVE.
- Safety code (duress): an alternate check-in code that succeeds normally but
  silently flags the account and notifies the professional-group holder if one
  exists. It is entered through a deliberately innocuous "Trouble checking in?"
  affordance, not a labelled control, so it does not advertise that a duress
  mechanism exists; a wrong or blank code is an ordinary check-in, so it can't
  be probed. (Documented limits: this deters and signals, it does not prevent.)
- Nudge/SMS reminders carry a check-in deep link. It lands on the check-in
  surface but still requires the owner's passkey/TOTP: email or SMS possession
  alone can never reset the switch, so an adversary who compromises a
  notification channel cannot suppress the dead-man's switch to keep the vault
  sealed. On the owner's own logged-in device the link is effectively one tap.
- The attempt is permanently visible in the owner-visible audit log and to
  all holders — collusion cannot be quiet.

**Residual risk.** A full quorum acting jointly against an owner who cannot
respond within the veto window succeeds. Secret sharing distributes trust; it
cannot create it. Coercion of the *owner* (rubber-hose) is out of scope for
any cryptographic design; the safety code is a tripwire, not a shield. The
death-certificate fast path is unverified in MVP and shortens the ladder (not
the veto window); this is an accepted MVP weakness, mitigated by notification
+ veto, and flagged for post-MVP verification work.

## T2. Share loss or theft

**Attack.** A printed card is lost, destroyed, stolen, or photographed —
including "helpful" cloud photo backups by holders. Aggregate variant: an
attacker collects multiple cards toward a quorum.

**Mitigations.**

- Below-threshold shares reveal nothing about the secret
  (information-theoretic; SLIP-39 §security). One card is useless.
- Cards are anonymous: no owner name, no holder name, no other holders'
  identities or count (`spec/share-card.md`). A found card cannot be routed
  to its vault or its peers.
- Quarterly health-check attestations (card ID + first/last word over a
  signed one-time link) detect loss/drift early; two consecutive misses on a
  share prompts a re-share suggestion.
- Proactive re-share regenerates all cards from the same master secret and
  revokes the old set (extendable SLIP-39 backup flag; old and new sets are
  incompatible, and the tool rejects mixed case numbers with a clear error).
- Kit documentation explicitly warns against photographing cards.

**Residual risk.** Theft of a quorum's worth of cards — which requires knowing
who holds them and where — plus possession of any bundle ciphertext defeats
the scheme entirely; physical security of holders' homes is inherited, not
provided. Conversely, silent loss of too many cards between health checks can
make recovery impossible until the owner re-shares; the health-check cadence
(quarterly) bounds this window. Photographed shares are as good as stolen and
undetectable by attestation (the holder still "has" the card).

## T3. Provider compromise (AmberKey infrastructure breached)

**Attack.** An attacker gains full control of AmberKey servers: database,
blob store, notification pipeline, deployment.

**Mitigations.**

- Blind server: no keys, no shares, no plaintext exist server-side to steal.
  All cryptography is client-side in the open-source core crate.
- Bundle blobs are ciphertext, content-addressed and versioned; tampering
  with them yields decryption failure, not silent corruption (age is
  authenticated encryption).
- App CSP is locked down with no external script origins; the served SPA is
  the primary code-injection surface and is treated as such (see T6 for the
  limits of this).
- Shareholder links are single-use with expiry and rate-limited; a server
  attacker can mint links but cannot conjure shares through them — health
  checks never transmit mnemonics.

**Residual risk.** Full metadata exposure (see T8). An attacker controlling
the server can: send false notifications, suppress real ones, manipulate
liveness timers toward QUORUM_ELIGIBLE, and serve a malicious web app to
*owners* (the strongest variant — a poisoned client at bundle-edit time reads
plaintext; this is why the recovery path never requires the hosted app, and
why the recovery tool is a separately verifiable artifact). Ceremony
manipulation still cannot decrypt anything without physical cards.

## T4. Provider death or bankruptcy

**Attack.** Not an attacker: AmberKey shuts down, is acquired and gutted,
loses its domains, or vanishes without notice.

**Mitigations.**

- Recovery requires no AmberKey infrastructure: printed cards + any bundle
  copy + `recover.html` (single file, works from `file://`, zero network).
- Local-first: clients hold a full encrypted bundle copy; holders may hold
  USB copies; export is nagged annually and after changes.
- Open formats exclusively (SLIP-39, age v1, ustar tar): any conforming
  third-party implementations suffice (`spec/bundle-format.md`,
  `spec/recovery.md`). Nothing proprietary rides *inside* standard artifacts.
- Tool + spec mirrored beyond our control: GitHub (primary), Codeberg,
  Software Heritage, Internet Archive; mirror list printed on every
  instruction sheet in priority order.
- Reproducible builds + minisign signatures let anyone verify or rebuild the
  tool without trusting surviving infrastructure (fingerprint printed on
  cards).
- ToS wind-down commitment: 12 months' notice, bundles downloadable
  throughout, final release converts the product to a fully offline flow.

**Residual risk.** The coordination layer dies: no automated liveness, no
notifications, no ceremony choreography — survivors must discover the death,
find holders (sealed circle directory in each instruction sheet covers this),
and self-organize. An owner who ignored export nags may leave a stale bundle;
staleness is bounded by the last export, and the playbook snapshot inside is
timestamped. Domain loss also kills the hosted-tool URL printed on cards —
hence the mirror list on paper, not just online.

## T5. False triggers

**Attack.** Again not malice, usually: hospitalization, incarceration,
off-grid travel, a lost phone + changed email make a living owner appear
dead. Malicious variant: an attacker who controls the owner's channels
suppresses nudges to *induce* escalation (see also T6).

**Mitigations.**

- Slow multi-channel ladder: 30/60/90-day base interval, then email, SMS,
  and a trusted-contact query, each with 7-day grace — minimum ~7 weeks of
  silence before the circle can even initiate.
- QUORUM_ELIGIBLE gates initiation only; the ceremony adds notification to
  all channels plus the 3–14 day veto window.
- Any check-in at any stage — including during the veto window — resets to
  ACTIVE. Check-in is deliberately low-friction (one tap).
- The trusted-contact query inserts a human ("have you heard from them?")
  before eligibility.
- Release delivers ciphertext only; a wrongly-released bundle without cards
  is still nothing (defense in depth with T1's quorum requirement).

**Residual risk.** An owner unreachable for the full ladder + veto window,
whose circle initiates and confirms in good faith, gets their vault opened
while alive. This is the deliberate trade against goal 2 (a real death must
eventually release). The owner controls the knobs (interval, veto length,
circle choice); the floor (~7 weeks) is a product decision documented here.
Wrongful release is also recoverable-by-rotation: a living owner can re-key
the vault (new age identity, re-share, re-export) after the fact.

## T6. Device compromise

**Attack.** Malware on the owner's device at onboarding, bundle edit, or
export time; malware on the recovering survivor's machine at reconstruction
time; a poisoned browser extension; a compromised copy of the recovery tool.

**Mitigations.**

- Honesty first: client-side encryption cannot defend against a fully
  compromised client, and no marketing claim will pretend otherwise.
- Exposure windows minimized: plaintext exists only during explicit
  operations; the vault key exists as plaintext only at generation, kit
  printing, and reconstruction.
- App CSP locked down; no third-party scripts or analytics in the app; all
  crypto in one auditable Rust/WASM core.
- Recovery tool: single self-contained file, no network use, verifiable by
  printed SHA-256 + minisign fingerprint (`spec/recovery.md`), reproducibly
  built and CI-checked (build twice, compare hashes). Docs direct survivors
  to a clean machine and offer an offline-first procedure.
- PDF kit generation is client-side; card material never transits the server.

**Residual risk.** The dominant practical attack on AmberKey, as on all E2E
systems. A compromised owner device at onboarding captures everything ever
typed into it. A compromised recovery machine captures the reconstructed key
and packet at the exact moment of maximum value. Printed-hash verification
defends the *tool*, not the *machine*. We reduce and document; we do not
eliminate.

## T7. Legal seizure / subpoena

**Attack.** Courts or law enforcement compel AmberKey — or circle members —
to produce material. Jurisdictional variants: gag orders, server seizure.

**Mitigations.**

- Capability, not policy, bounds compelled disclosure against AmberKey: we
  hold ciphertext + routing + timestamps and can produce nothing that
  decrypts (see server-side inventory above).
- No custodial key relationship exists to be compelled; shares are physical
  artifacts in private hands, outside our reach.
- Owner-visible append-only audit log makes covert server-side manipulation
  detectable after the fact.

**Residual risk.** Everything in T8's metadata list is one subpoena away.
Circle members can be individually and lawfully compelled to produce cards —
cryptography does not outrank a court order aimed at humans, and a legal
process that assembles a quorum of cards plus a seized bundle succeeds. The
veto window still applies only to ceremonies run through our service; a
compelled *offline* reconstruction bypasses AmberKey coordination entirely.
Owners with legal-seizure threat profiles should weigh circle composition
(and jurisdiction of holders) accordingly.

## T8. Metadata exposure

**Attack.** What the coordination data itself reveals — to AmberKey staff, a
breach (T3), or legal process (T7).

**What is exposed, exactly** (per `circle_members`, `liveness`, `ceremonies`,
`attestations`, `audit_log` in seedplan §6; kept deliberately minimal):

- That an owner has a vault; bundle sizes and update timestamps.
- Circle size and shape: member email/phone, group assignments, card IDs.
  Names and relationships are NOT sent to the server; they exist only in the
  owner's local plan, the encrypted bundle, and the printed sealed pages.
- Liveness rhythm: check-in times, escalation history, configured intervals.
- Ceremony history and health-check attestation timestamps.
- Attestation word fingerprints: `sha256(first|last mnemonic word)` per
  check. This is brute-forceable (2^20 candidates) and deliberately so — the
  ritual asks for SLIP-39 word #1 (the share-set identifier, non-secret and
  identical on every card of a set) and the final word (RS1024 checksum,
  derived from the share). Full preimage recovery therefore reveals no
  share-value entropy; the fingerprint exists only to detect a holder whose
  physical card changed between checks. Changing *which* words the ritual
  asks for requires revisiting this analysis.

**Mitigations.**

- Field minimization: routing needs contact handles and card IDs; it never
  needs share words, packet contents, or account lists — so they are never
  sent.
- Cards themselves are metadata-free (T2); health checks transmit card ID +
  two words' presence confirmation, not the mnemonic.
- No third-party analytics in the app; self-hosted page counts only on the
  marketing site. No plaintext secrets in logs or error reports (CI-enforced
  policy).
- The `circle.json` inside the bundle (full directory, relationships, notes)
  is sealed; the server-side routing copy is the reduced subset above, and
  the duplication is documented here by design rather than hidden.

**Residual risk.** The social graph of "who would you trust with your death"
is itself sensitive, and we hold it in cleartext because notification routing
requires it. A breach or subpoena exposes it. Traffic analysis of check-in
and ceremony timing is possible for anyone who can read our traffic or logs.
Owners for whom this graph is the crown jewel (e.g., under targeted state
attention) exceed AmberKey's threat model, and this document says so rather
than implying otherwise.

---

## Out-of-scope (declared, not defended)

- Rubber-hose cryptanalysis of the owner or holders.
- Endpoint compromise beyond the reductions in T6.
- A quorum of holders acting in concert against a dead or unreachable owner
  — that is the *product working*, from the protocol's point of view.
- Nation-state targeted attacks on specific individuals.
- Cryptanalytic breaks of X25519/ChaCha20-Poly1305/SHA-256 or the SLIP-39
  construction; we inherit the community's confidence and would inherit any
  break alongside far larger targets. Post-quantum migration is a spec-level
  concern tracked for a future bundle version (the format is versioned for
  exactly this kind of change).

## Reporting

Found a hole in this model — or in the code? `security@amberkey.app`, or see
`SECURITY.md` for the disclosure process. Reports that prove any statement in
this document wrong are the most valuable thing you can send us.
