# Architecture decision records

Use an ADR when a consequential choice needs lasting rationale for review.
Keep the current contract in the owning `ARCHITECTURE.md`. Git records
chronology and routine changes.

- Use a descriptive, undated slug under `docs/adrs/`. Start with
  [the template](adrs/template.md) or run `make new-adr SLUG=decision-slug`.
- Record the decision's status, context, credible alternatives, consequences,
  and evidence limits. Pin evidence to its source revision when it matters.
- Link the affected architecture. If a choice is superseded, link both ADRs
  and preserve the original rationale.
- Keep sensitive values, local machine paths, and secrets out of the record.
