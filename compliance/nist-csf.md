# NIST CSF 2.0 profile (self-assessed)

Last reviewed: 2026-07-11 · Self-assessed; honest about maturity. AmberKey is a
small pre-launch product; this profile says what exists today and names what
doesn't, rather than aspirational language. Operational gaps are tracked in
`ops/SOC2-PHASE0.md`.

## Govern (GV)

Roles are simple (single operator today) and access is enumerated: an access
inventory across the forge, hosts, registrar, and SaaS (Stripe/Twilio/Resend)
is part of the Phase-0 hardening list. Policy exists as working documents
(threat model, disclosure process, this compliance set) rather than a formal
ISMS — stated as such.

## Identify (ID)

- **Asset inventory:** small and explicit — the monorepo, the AWS production
  account (defined entirely by `infra/aws/` Terraform), a self-hosted dev
  host + forge + CI runner, and the SaaS processors listed publicly as
  subprocessors in the privacy policy (AWS, Stripe, Twilio, Resend).
- **Data inventory:** published in the privacy policy, and structurally short —
  ciphertext bundles, contact routing, liveness/billing state. The defining
  control is what the server *cannot* hold: vault contents, keys, or shares.
- **Risk assessment:** the public threat model enumerates residual risks
  honestly (including metadata exposure and duress limits).

## Protect (PR)

- **Identity & access:** passkey-first authentication (see
  [nist-800-63b.md](nist-800-63b.md)), step-up re-auth on sensitive changes,
  single-use rate-limited links for shareholders, env-allowlisted admin.
- **Data security:** end-to-end encryption client-side (age v1); the server is
  blind by architecture, not policy. TLS everywhere in transit.
- **Platform security:** versioned, signed, pinned releases; warnings-as-errors
  and dependency scanning in CI (see [ssdf.md](ssdf.md)).
- **Data at rest:** all production storage is KMS-encrypted (S3 data bucket,
  snapshots, secrets); runtime secrets live in SSM SecureString, injected at
  task start, never in files or repositories.

## Detect (DE)

Production: **GuardDuty** (managed threat detection on the account),
**CloudTrail** (validated audit trail of every control-plane action),
**CloudWatch** alarms (service down — where silence itself alarms — API
errors, DB-replication failures), and **WAF telemetry** at the edge. Formerly
the weakest function; no longer, though we still make no SOC claim — alerts
page one operator. Dev retains its Loki/Grafana stack.

## Respond (RS)

Public coordinated-disclosure channel (`SECURITY.md`, security@); a written
incident-response plan with breach-notification steps is a Phase-0 item. The
audit log gives owners per-account forensics.

## Recover (RC)

Recovery is the product's core competency, in both senses. For the service:
continuous database replication to versioned, cross-region-replicated S3
(**RPO ~1 second**), rebuild-from-nothing via infrastructure-as-code with
restore-on-boot (**RTO ~10 minutes** — the drill is literally the deployment),
plus weekly copies encrypted to the operator's offline key that no cloud
credential compromise can read or destroy. For customers: the entire offline
recovery path (mirrored tool + printed shares) works **even if AmberKey never
recovers**. Restore drill evidence: `ops/DR-RUNBOOK.md`.
