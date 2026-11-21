# Unnamed musical tooling

## Development workflow

- Start with the [collaboration guide](docs/REPO_COLLAB_GUIDE.md), [architecture map](docs/ARCHITECTURE.md), and [core beliefs](docs/design-docs/core-beliefs.md). Read the governing package contract before changing behavior.
- Inspect the worktree and preserve unrelated changes. Keep changes small and update affected docs in the same task.
- For complex work, follow the [execution plan guide](docs/PLANS_GUIDE.md). Before finishing a code change, check the [quality targets](docs/QUALITY_SCORE.md) and record the change using the [history guide](docs/HISTORY_GUIDE.md).
- Keep prompts, policies, and architectural rules versioned. Route package development through [the Cargo workspace](crates/AGENTS.md), [Bitwig adapter](crates/controller/adapters/bitwig-ctl/AGENTS.md), or [Kontakt adapter](crates/controller/adapters/kontakt-ctl/AGENTS.md) instructions as applicable.
- For contribution, security, and release work, use [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), and the [CI](docs/CICD.md) and [release](docs/releases/README.md) guides.

## Bitwig project layout

- The project root is the directory containing the saved `.bwproject`. Keep `PERFORMANCE.md` there.
- Keep the original score and its provenance record in the project's established source location. For a new project without an agreed source location, use the project root.
- Use project-root `references/` for external listening references and `evidence/` for new evidence. Honor the project's existing evidence location.
- Preserve existing project files and paths. Create folders as needed and leave `recordings/`, `master-recordings/`, and other DAW-managed directories to Bitwig. Follow the [checkpoint procedure](skills/WORKFLOW.md#experimental-project-backups) for manual backups.

## Skill workflow

- Start music-production work at [task routing](skills/WORKFLOW.md#route-by-outcome). Follow the owning skill and the workflow's [execution boundaries](skills/WORKFLOW.md#mandatory-execution-boundary) for any live action.
- Use the workflow's [artifact authority](skills/WORKFLOW.md#meaning-and-authority) and [mutation ownership](skills/WORKFLOW.md#artifact-and-mutation-ownership) to establish scope. Verify the requested result using the owning workflow's [readiness and reconciliation rules](skills/WORKFLOW.md#readiness-and-change-propagation).
- Put skill execution instructions in `SKILL.md` and directly routed references. Reserve skill `SPEC.md` and maintenance sidecars for authoring and review.
