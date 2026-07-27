# Contributing

Thanks for helping people's families. A few hard rules keep this codebase
trustworthy:

## Reporting and proposing changes

- **Bugs and questions:** open an issue on the public repo
  (github.com/amberkey-app/amberkey).
- **Security vulnerabilities:** email security@amberkey.app — see
  [SECURITY.md](SECURITY.md) for coordinated disclosure and the safe harbor.
  Never open a public issue for a vulnerability.
- **Pull requests** against the public repo are welcome and reviewed. It is an
  assembled mirror of a private monorepo's open subset, so accepted changes are
  applied internally (with credit) and flow back out at the next release — your
  PR may be closed as "merged internally" rather than merged directly.
- **Tests are required:** new functionality comes with tests, bug fixes come
  with a regression test. Contributions are accepted under Apache-2.0.

## Crypto rules

1. **All crypto lives in `crates/core`.** No cryptographic code in TypeScript,
   the server, or anywhere else. The web app and recovery tool call the WASM
   build of core; the CLI links it natively.
2. **Standard formats only.** SLIP-39 and age v1. No custom share encodings,
   no proprietary extensions inside standard artifacts.
3. **The Trezor vector suite is the acceptance gate.** Any change under
   `crates/core/src/slip39/` must keep `tests/trezor_vectors.rs` green, full
   suite, no cases skipped.
4. **Bundle compatibility is forever.** Readers must open every published
   schema version. New fields/files only; never change meaning.

## Process rules

- Every PR touching the liveness or release-ceremony logic must update
  `spec/threat-model.md` (even if only to state "no change to the model,
  because …"). Reviewers enforce this by hand for now.
- `cargo clippy --workspace --all-targets --features kit -- -D warnings` must
  pass; so must `cargo audit` and `npm audit` (high+).
- The recovery tool must stay: one file, zero network requests, reproducible.
  The Playwright suite asserts the first two; CI's double-build the third.
- Compile with `-j2` in scripts and docs (small-VM friendly).

## Licensing boundary

Open source (Apache-2.0): `crates/core`, `crates/recover-cli`,
`web/recovery-tool`, `spec/` — published via `scripts/publish-recovery.sh`.
Everything else is proprietary (`LICENSE.md`). Nothing under the open
components may ever depend on a proprietary component; the reverse is fine.

## Dev setup

See README. Rust stable, Node 20, `wasm32-unknown-unknown` target, wasm-pack.

## Tone in user-facing copy

Survivor-facing text (recovery tool, instruction sheets, shareholder emails,
`site/` docs) is read by grieving, often non-technical people. Short
sentences. No jargon. No urgency theater. Reassure before instructing.
