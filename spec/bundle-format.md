# AmberKey Continuity Bundle Format

**Status:** v1 (schema_version 1). Decryption of every published schema version
MUST remain supported forever. Version gates parsing of newer optional
sections; it never rejects old bundles.

## Outer layer

A continuity bundle is a single file:

```
bundle = age_encrypt(recipient = vault public key, payload = tar)
```

- **Encryption:** [age](https://age-encryption.org/v1) version 1, X25519
  recipient stanza only. No passphrase stanza, no armor (binary format).
- **Payload:** a POSIX ustar tar archive. For reproducibility every entry is
  written with `mtime = 0`, `mode = 0644`, no user/group names.

The recipient is the **vault master key**: an age X25519 identity generated
client-side at onboarding. The server only ever sees the ciphertext.

## Master secret encoding (what SLIP-39 splits)

The SLIP-39 master secret is the **raw 32-byte X25519 scalar** of the vault
identity, obtained by Bech32-decoding the age secret key string
(`AGE-SECRET-KEY-1...`, HRP `age-secret-key-`) as specified by age v1.
Reconstruction re-encodes the 32 bytes with the same HRP (uppercase) and
parses it as an age identity.

SLIP-39 parameters used by AmberKey:

- extendable backup flag: **set** (allows proactive re-share without
  invalidating the bundle; the KDF salt does not bind the share-set identifier)
- iteration exponent: 1 (20 000 PBKDF2 iterations)
- passphrase: empty (`""`)

Any conforming SLIP-39 implementation can reconstruct the scalar; any
conforming age implementation can then decrypt the bundle. Nothing
proprietary is required.

## Tar contents

| Path | Required | Content |
|---|---|---|
| `manifest.json` | yes | see below |
| `packet/` | yes | Layer 1: executor packet |
| `packet/executor-checklist.md` | yes | first page a survivor reads |
| `packet/cards/<id>.json` | yes | account cards (schema: seedplan 7.3 / `crates/core/src/card.rs`) |
| `packet/letters/*` | no | letters, attachments (markdown or binary) |
| `secrets/<id>.json` | no | Layer 2 secret items (`SecretItem` model) |
| `circle.json` | yes | shareholder directory (`CircleDirectory` model) — the sealed-envelope content |
| `playbook-snapshot/*.md` | yes | frozen copies of playbooks current at export time, timestamped |

## manifest.json

```json
{
  "format": "amberkey-bundle",
  "schema_version": 1,
  "created_at": "2026-07-06T12:00:00Z",
  "owner_name": "display name"
}
```

`format` MUST be `amberkey-bundle`. Unknown top-level fields MUST be ignored
(forward compatibility). `created_at` is RFC 3339 UTC.

## Compatibility rules

1. New schema versions may add files and manifest fields; they MUST NOT change
   the meaning of existing ones.
2. Readers MUST ignore unknown files and fields.
3. A reader built for schema v1 MUST successfully extract and render the
   packet of any later-version bundle (degraded rendering is acceptable;
   refusal is not).
