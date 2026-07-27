# AmberKey Share Card Data Format

What is printed on each physical card of the kit, and what is deliberately
left off. Model: `ShareCard` in `crates/core/src/card.rs`.

## Card front

| Field | Content |
|---|---|
| Words | the SLIP-39 mnemonic, numbered, 4 columns |
| QR | the mnemonic as plain space-separated lowercase text (nothing else) |
| Card ID | `<case>-G<group>-M<member>`, e.g. `AK7F3K-G2-M1` (1-based indices) |
| Case number | random 6-char base32 (no 0/1/8/9-lookalikes), same on every card of one share set; changes on re-share |
| Recovery URL | `https://recover.amberkey.app` |
| Tool hash | SHA-256 of the `recover.html` release current at print time |
| Minisign key | the release-signing public key fingerprint |

## Deliberately absent

- Owner name
- Holder name
- Other holders' identities or count
- Anything linking the card to AmberKey account metadata

A found card alone identifies nobody and unlocks nothing.

## Instruction sheet (one per holder)

1. Plain-language "if you are reading this" opener.
2. The offline recovery procedure (condensed from `spec/recovery.md`).
3. Mirror list in priority order.
4. **Sealed section** (fold-and-tape page): the circle directory — who the
   other holders are and how to reach them. The holder is asked not to open
   it unless the recovery is real.

## Card ID semantics

`G<g>-M<m>` maps to SLIP-39 group index `g-1`, member index `m-1`. Health
checks identify a card by ID + first and last mnemonic word; the server
stores only the card ID and attestation timestamps, never words.

## QR content rationale

The QR encodes exactly the mnemonic text so any generic scanner + any
SLIP-39 tool can consume it. Bundle-on-QR is deferred from MVP; the
instruction sheet directs holders to the USB copy for the bundle.
