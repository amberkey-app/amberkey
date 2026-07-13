# AmberKey Offline Recovery Procedure

This procedure works with **zero network access** and **no AmberKey
infrastructure**. It is what the printed instruction sheet points to. If the
company no longer exists, this still works.

## What you need

1. **Share cards** — enough to meet the quorum printed rules (the tool tells
   you your progress as you enter cards).
2. **The continuity bundle** — a single `.age` file, from the holder USB copy,
   an AmberKey download, or the owner's backups.
3. **The recovery tool**, either:
   - `recover.html` — one self-contained file; open in any modern browser
     directly from disk (`file://`). No internet needed or used.
   - `amberkey-recover` — a command-line binary (Linux/macOS/Windows).

## Getting the tool (mirror list, in priority order)

1. `https://recover.amberkey.app` (hosted copy — save the page)
2. GitHub releases: `github.com/amberkey-app/amberkey`
3. Codeberg mirror: `codeberg.org/amberkey/amberkey`
4. GitLab mirror: `gitlab.com/amberkey/amberkey-recover`
4. Software Heritage archive:
   `archive.softwareheritage.org/browse/origin/directory/?origin_url=https://codeberg.org/amberkey/amberkey`
   (or search "amberkey")
5. Internet Archive: `archive.org/details/amberkey`

## Verifying the tool

Every card prints the tool's SHA-256 hash and the minisign public key
fingerprint.

```
sha256sum recover.html          # must match the printed hash for that release
minisign -Vm recover.html -P <printed public key>
```

If the hashes differ, the card may predate a newer release: any release's
signature must verify against the printed minisign key. When in doubt use the
exact release whose hash is printed.

A note on rebuilding from source: builds are deterministic within a pinned
environment, and the published hash is produced and signed from the release
environment. Bit-exact independent reproduction across arbitrary machines is
a known gap we are closing (a dependency's proc macro compiles
nondeterministically); the signature, not a rebuild, is the trust root today.

## Procedure (browser)

1. Disconnect from the internet (optional but recommended; the tool makes no
   requests either way).
2. Open `recover.html` from disk.
3. Type each card's word phrase, one card at a time. The tool shows which
   groups are complete and what is still missing.
4. Load the bundle file when prompted.
5. The packet renders in place. **Start with the executor checklist.**
   Save/print what you need.

## Procedure (command line)

```
amberkey-recover --bundle continuity-bundle.age --out packet/
# then open packet/packet/executor-checklist.md
```

Enter each card's words on one line when prompted; finish with an empty line.
`--check-shares` validates cards and shows quorum progress without decrypting.

## If something fails

- *"invalid mnemonic checksum"* — a word is mistyped or misread; the checksum
  pinpoints nothing for safety, re-type the whole line.
- *"insufficient shares"* — you need more cards; the `--check-shares` output
  (or the browser progress panel) says which groups are incomplete.
- *"share digest verification failed"* — a card is from a different (older,
  revoked) share set. Check the case number printed on each card: all cards
  must show the same case number.
- The bundle will not decrypt with a complete quorum — the bundle and the
  cards are from different vault generations. Look for a newer bundle copy or
  older cards.
- You decrypted the bundle with third-party age tools and have a file you
  can't open — it is a standard tar archive: `tar xf <file>` (built into
  macOS, Linux, and Windows 10+), or any archive utility.

Under no circumstances does partial progress reveal anything: below the
quorum, the cards carry zero usable information about the vault key.
