# Archive key

Status: draft · 2026-07-23

The **archive key** is a second age X25519 identity, separate from the vault
master key, used to encrypt arbitrary export files (service data that has no
provider legacy tool: ChatGPT, Claude, Notion, Google Takeout, and the like) so
the recovery circle can decrypt them after the owner is gone. Survivor-facing
copy may call it the **legacy recipient key**; both names mean the same thing.

## Provenance

- Generated **client-side** as `age::x25519::Identity::generate()`
  (`crates/core` `generate_identity`), the same primitive as the vault key.
- The **private half** is stored as a Layer 2 secret item of
  `kind = "archive_identity"` inside the encrypted bundle (one JSON file under
  `secrets/`, see `bundle-format.md`). It is protected exactly like every other
  vault secret: recoverable only through the SLIP-39 share ceremony plus the
  bundle.
- The **public recipient** (`age1...`) is derived on demand
  (`recipient_from_secret`) and surfaced in the console, printed on the kit, and
  may be placed in the executor packet. It is public by construction: it only
  lets someone *encrypt* to the owner, so sharing or printing it costs nothing.

## Invariants

1. **Never rotated on vault re-key.** A vault re-key regenerates only the vault
   identity and re-encrypts the plan verbatim to it; anything in the plan
   (including the archive key) rides along unchanged. This is what makes every
   file ever encrypted to the archive recipient decryptable forever, with no
   need to retain superseded vault identities.
2. **Append-only.** If an archive key is ever replaced (e.g. suspected
   compromise), a new one is appended; the old ones are retained so their
   ciphertext still opens. The **newest** is the recipient surfaced for new
   encryptions; recovery holds **all** of them and tries each.
3. **One per plan by default.** Provisioned automatically the first time an
   owner with a vault unlocks, so existing bundles upgrade transparently.

## Sidecar file convention

- An archive export is a **bare age v1 file over raw bytes** (no tar wrapper),
  produced by `encrypt_file(recipient, bytes)` — so a stock `age -r <recipient>`
  produces an interchangeable file.
- Convention: name it `<original-name>.age` (e.g. `chatgpt-export.zip.age`).
- AmberKey **does not store** these files. The owner keeps them anywhere durable;
  the executor packet records *where* (see the knowledge-estate inventory).

## Recovery

Two steps, both offline:

1. Reconstruct the vault via the normal share ceremony (cards → SLIP-39 →
   vault identity → decrypt bundle). The decrypted bundle contains the archive
   key(s).
2. Decrypt the archive export with the recovered archive key:
   `decrypt_file(secret, bytes)`. The recovery tool exposes this as an
   "Open an archive export" panel; it tries each recovered archive key.

## Escape hatch (no AmberKey required)

Because an archive export is a plain age file, any `age`/`rage` binary decrypts
it given the recovered archive secret key:

```sh
age -d -i archive-key.txt chatgpt-export.zip.age > chatgpt-export.zip
```

where `archive-key.txt` holds the `AGE-SECRET-KEY-1...` recovered from the
bundle. This is the same survivability guarantee as the bundle itself.

## Threat notes

An archive ciphertext is decryptable by (the `.age` file) + (a card quorum) +
(the bundle) — i.e. the **same blast radius as the vault**. So archive exports
are as sensitive as the family backup: keep them away from card holders, and
never hand a card holder both an export and enough cards. See
`spec/threat-model.md`.
