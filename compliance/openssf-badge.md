# OpenSSF Best Practices badge — answer sheet (passing level)

Last reviewed: 2026-07-11 · Status: **PASSING (100%)** —
<https://www.bestpractices.dev/projects/13576>, registered 2026-07-11. The
badge is in the public README and on the trust page. The answers below are the
submitted record; keep them in sync with the live entry.

**Scope:** the badge covers the **open-source recovery components** published
in `github.com/amberkey-app/amberkey` (crypto core, recovery CLI, offline
recovery tool, format specs, compliance docs) — the badge program certifies
FLOSS projects, and that repo is the FLOSS project. It is not a claim about
the hosted AmberKey service; the service's posture is covered by the other
documents in this directory.

## Basics

| Criterion | Answer |
|---|---|
| Project homepage (HTTPS) | Met — https://amberkey.app |
| Description of what/why | Met — README + amberkey.app |
| Contribution process documented | Met — CONTRIBUTING.md (public-mirror PR flow described) |
| Contribution requirements (tests, style) | Met — CONTRIBUTING.md (tests required, clippy -D warnings, rustfmt) |
| FLOSS license | Met — Apache-2.0 (`LICENSE`), stated in README + Cargo.toml |
| License location | Met — repo root |
| Basic documentation | Met — `spec/` (bundle format, recovery procedure, share card, threat model) |
| Reference documentation | Met — the specs are the reference; CLI has `--help` |
| Sites HTTPS | Met — site, repo, mirrors all HTTPS |
| Discussion mechanism | Met — GitHub issues (enabled 2026-07-11) |
| English supported | Met |
| Maintained | Met — active releases |

## Change control

| Criterion | Answer |
|---|---|
| Public VCS | Met — GitHub + Codeberg mirror + Software Heritage |
| Change history / interim versions | Met with caveat — the public repo is assembled per release from a private monorepo; per-release history is public, day-to-day history is not. Stated in CONTRIBUTING.md. |
| Unique version numbering | Met — semver tags (v0.x.y) |
| Release notes | Met — CHANGELOG.md (Keep a Changelog format) + per-release notes on tagged releases |

## Reporting

| Criterion | Answer |
|---|---|
| Bug reporting process | Met — GitHub issues; CONTRIBUTING.md |
| Bug response | Met — small project; issues acknowledged (track record accrues) |
| Vulnerability report process | Met — SECURITY.md, security@amberkey.app |
| Private vulnerability reporting | Met — email channel |
| Response time ≤ 14 days | Met — 72-hour acknowledgment committed in SECURITY.md |

## Quality

| Criterion | Answer |
|---|---|
| Working build system | Met — cargo; `node build.mjs` for the recovery tool |
| Automated test suite | Met — cargo tests (incl. the complete Trezor SLIP-39 vector suite), Playwright offline-recovery tests |
| New-functionality tests policy | Met — CONTRIBUTING.md |
| Tests run in CI | Met — every push |
| Warning flags | Met — `clippy -D warnings` (warnings are build failures) |

## Security

| Criterion | Answer |
|---|---|
| Secure development knowledge | Met — threat model is a spec document; design is blind-server |
| Good crypto practices (published algorithms) | Met — SLIP-39, age v1 (X25519/ChaCha20-Poly1305), SHA-256, Ed25519; see `compliance/crypto.md` |
| No home-rolled crypto | Met — the sole in-house cryptographic code (SLIP-39 arithmetic) implements a public spec and is gated on its reference vectors |
| Keylengths | Met — 256-bit curves/keys throughout |
| No broken algorithms (MD5/SHA-1 for security) | Met — none |
| Perfect forward secrecy where applicable | N/A — data at rest, not a transport protocol; TLS handles transport |
| Password storage | N/A — no passwords exist |
| Secure random | Met — OS RNG via getrandom |
| Vulnerabilities patched ≤ 60 days | Met — commitment; `cargo audit`/`npm audit` gate CI |
| No leaked credentials in repo | Met — secrets live in gitignored env files only |

## Analysis

| Criterion | Answer |
|---|---|
| Static analysis | Met — clippy (all targets, deny warnings) every push |
| Static analysis for vulnerabilities | Met — cargo audit + npm audit every push |
| Dynamic analysis | Met (suggested criterion) — Playwright end-to-end suites incl. a fully offline recovery run |
| Memory-safety tools | N/A justification — Rust's compile-time guarantees; no unsafe crypto code |

## Registration steps (owner)

1. Sign in at bestpractices.dev with the GitHub account that owns
   `amberkey-app/amberkey`; add the project.
2. Paste the answers above (the form is the same order).
3. On "passing," add the badge line the site gives you to the public README
   (via `scripts/publish-recovery.sh`'s README block) and update the trust page.
