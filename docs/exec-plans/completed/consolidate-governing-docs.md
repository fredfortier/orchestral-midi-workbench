# Consolidate governing documentation

## Goal

Make package architecture the review surface for musical and native data
contracts. Keep READMEs short and retain only decision-relevant development
history.

## Scope

- Consolidate controller, converter and Kontakt contracts into their owning
  architecture files. Preserve wire grammar, musical policy and failure rules.
- Remove the design and product-spec scaffold. Keep executable skill procedures
  with their skills and qualification procedures with their runbooks.
- Preserve catalog consistency checks and qualify any migration of their inputs.

## Milestones

- [x] Inventory the architecture, legacy work records and documentation consumers.
- [x] Consolidate the contracts and recover decision history.
- [x] Retarget links and documentation verification, then check the result.

## Validation

Compare old and new contracts by responsibility. Check local links, removed
path references, catalog policy checks and the affected offline tests.

## Outcome

The root and package READMEs are at most seven lines. Package architecture now
owns the merged contracts. Three ADRs retain consequential decisions
and pin the retired evidence locations.

All 232 offline workspace tests reported success with the pinned inputs.
The research checker, link audit and catalog checks passed. Remaining archive
provisioning and native deployment work is in the
[debt tracker](../tech-debt-tracker.md).
