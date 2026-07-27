# AmberKey recovery components

[![OpenSSF Best Practices](https://www.bestpractices.dev/projects/13576/badge)](https://www.bestpractices.dev/projects/13576)
[![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/amberkey-app/amberkey/badge)](https://scorecard.dev/viewer/?uri=github.com/amberkey-app/amberkey)
*(badge scope: this repository — the open recovery components, not the hosted service. Scorecard process checks score low by construction: this is an assembled release mirror, not the day-to-day tree.)*

Everything needed to recover an AmberKey continuity bundle **without
AmberKey**: the crypto core (SLIP-39 + age, gated on the official Trezor test
vectors), the offline recovery tools, and the format specifications.

- `spec/recovery.md` — the recovery procedure (start here)
- `web/recovery-tool` — builds `recover.html`: one self-contained file that
  works from `file://` with no network
- `crates/recover-cli` — `amberkey-recover`, the command-line equivalent
- `spec/bundle-format.md` — bundles are age v1 + tar + SLIP-39: standard
  formats, recoverable with third-party tools alone

Mirrored at github.com/amberkey-app/amberkey, codeberg.org/amberkey/amberkey,
and gitlab.com/amberkey/amberkey-recover. Verify releases against the SHA-256
hash and minisign key printed on any AmberKey share card.

Copyright 2026 Tarnover, LLC. Licensed Apache-2.0. Published from AmberKey's development
repository; issues and security reports: security@amberkey.app.

```sh
cargo test --workspace                    # includes the full Trezor vector suite
cd web/recovery-tool && node build.mjs    # reproducible recover.html
```
