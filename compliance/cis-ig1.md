# CIS Controls v8 — IG1 self-assessment

Last reviewed: 2026-07-11 (updated for the AWS production stack — these rows
describe production as deployed by `infra/aws/`) · Self-assessed; IG1 is the designed scope for small
organizations. Verdicts: ✅ in place · ⚠️ partial (gap named) · ➖ N/A for a
single-operator SaaS of this size. Cross-references: `ops/SOC2-PHASE0.md`,
`ops/DR-RUNBOOK.md`.

| # | Control (IG1 scope) | Verdict | Evidence / gap |
|---|---|---|---|
| 1 | Inventory of enterprise assets | ✅ | Small and enumerated: two app CTs (dev/prod), reverse proxy, forge+CI, observability CT, operator workstation — documented in the deploy/DR runbooks. |
| 2 | Inventory of software assets | ✅ | Everything deployed is a pinned, versioned container image; dependencies are lockfiled (`Cargo.lock`, `package-lock.json`) and audited in CI. |
| 3 | Data protection | ✅ | Data inventory is public (privacy policy); customer secrets are client-side encrypted (blind server); all production storage is **KMS-encrypted at rest** (S3, snapshots); the DB replicates continuously to versioned, cross-region-replicated S3, plus weekly copies encrypted to the operator's offline key. |
| 4 | Secure configuration | ✅ | Hardened cookies, strict CSPs, no default credentials anywhere, admin disabled unless allowlisted, TLS terminated at the proxy for all public hosts. |
| 5 | Account management | ⚠️ | Product accounts: passkey-only, disabled-account enforcement, admin allowlist. Gap: the *operator-side* account/SSH-key inventory (Proxmox, forge, registrar, SaaS) is written but not yet verified item-by-item. |
| 6 | Access control management | ✅ | MFA-by-construction for the product (passkeys); least privilege: read-only deploy tokens, send-only mail key, restricted Stripe key, step-up re-auth on sensitive actions. Operator MFA verification is the same open item as control 5. |
| 7 | Continuous vulnerability management | ✅ | `cargo audit` + `npm audit` gate every push; ECR scans images on push; patch path is a pinned-image release away. Production runs on Fargate — there is no server OS for us to patch, closing the former manual-OS-updates gap. |
| 8 | Audit log management | ✅ | CloudWatch Logs (90-day retention) for the service, CloudTrail (validated log files) for the account, WAF request sampling at the edge; plus the per-account append-only application audit log. Dev keeps its Loki stack. |
| 9 | Email & browser protections | ➖ | No enterprise email/browser fleet; transactional mail is SPF/DKIM-verified via the provider. |
| 10 | Malware defenses | ➖/✅ | No endpoint fleet; servers run pinned containers on minimal images — the IG1 intent (limit executable surface) is met structurally. |
| 11 | Data recovery | ✅ | Nightly encrypted backups (local + off-site FTPS), documented RPO 24h / RTO ~1h, **restore drill passed 2026-07-11** (`ops/DR-RUNBOOK.md`). |
| 12 | Network infrastructure management | ✅ | Single ingress (NPM, TLS), internal-only LAN for CTs, LAN-only observability, cluster firewall default-drop with explicit allows. |
| 13 | Network monitoring & defense | ✅ | WAF at the edge on every surface (managed core rules, known-bad inputs, IP reputation, API rate limiting), GuardDuty threat detection on the account, CloudWatch alarms on service health and DB replication. |
| 14 | Security awareness | ➖ | Single operator today; becomes real (with evidence) at first hire — noted in Phase 0 §12. |
| 15 | Service provider management | ✅ | Subprocessors enumerated publicly (privacy policy); gap (minor): collecting their SOC 2/DPA PDFs into a vendor file is on the Phase-0 list. |
| 16 | Application software security | ✅ | The strongest area: SSDF mapping (`ssdf.md`), warnings-as-errors, reference-vector tests, reproducible signed releases, public VDP with safe harbor + CVE commitment. Change control: protected `main` (PR + green CI + no force-push) and tag-gated, dev-first releases (`ops/WORKFLOW.md`). |
| 17 | Incident response management | ⚠️ | Disclosure channel + 72h ack committed (SECURITY.md); the internal IR plan document (roles, breach-notification timeline) is still to be written — the main remaining Phase-0 gap. |
| 18 | Penetration testing | ⚠️ | Not yet performed; an independent review of the crypto core is the published escalation path (trust page). Planned, not claimed. |

**Summary: 12 ✅ · 3 ⚠️ (operator MFA/account inventory — now including the
AWS account's root MFA and IAM users; IR plan; pen test) · 3 ➖.** The ⚠️
items are individually small and tracked in `ops/SOC2-PHASE0.md`.
