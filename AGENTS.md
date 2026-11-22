# Orchestral MIDI Workbench

## Development workflow

- For repository development, start with the [collaboration guide](docs/REPO_COLLAB_GUIDE.md) and [architecture map](docs/ARCHITECTURE.md). Read the governing package contract before changing behavior.
- Inspect the worktree and preserve unrelated changes. Keep changes small and update affected docs in the same task.
- For complex work, follow the [execution plan guide](docs/PLANS_GUIDE.md). Before finishing a code change, check the [quality targets](docs/QUALITY_SCORE.md). Use the [ADR guide](docs/ADR_GUIDE.md) for consequential decisions.
- Keep prompts, policies, and architectural rules versioned. Route package development through [the Cargo workspace](crates/AGENTS.md), [Bitwig adapter](crates/controller/adapters/bitwig-ctl/AGENTS.md), or [Kontakt adapter](crates/controller/adapters/kontakt-ctl/AGENTS.md) instructions as applicable.
- For contribution, security, and release work, use [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), and the [CI](docs/CICD.md) and [release](docs/releases/README.md) guides.

## Bitwig project layout

- Follow the [production project layout](skills/WORKFLOW.md#bitwig-project-layout) for saved projects, source scores, and evidence.

## Skill workflow

- Keep music-production skills, their references, and their authoring notes within the production documentation. They must never route to repository development documentation or source code. Report missing capabilities within the production task scope.
- Start music-production work at [task routing](skills/WORKFLOW.md#route-by-outcome). Follow the owning skill and the workflow's [execution boundaries](skills/WORKFLOW.md#mandatory-execution-boundary) for any live action.
- Use the workflow's [artifact authority](skills/WORKFLOW.md#meaning-and-authority) and [mutation ownership](skills/WORKFLOW.md#artifact-and-mutation-ownership) to establish scope. Verify the requested result using the owning workflow's [readiness and reconciliation rules](skills/WORKFLOW.md#readiness-and-change-propagation).
- Put skill execution instructions in `SKILL.md` and directly routed references. Reserve skill `SPEC.md` and maintenance sidecars for authoring and review.
