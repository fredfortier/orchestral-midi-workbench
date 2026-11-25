# Orchestral MIDI Workbench

## Development workflow

- For repository development, start with the [collaboration guide](docs/REPO_COLLAB_GUIDE.md) and [architecture map](docs/ARCHITECTURE.md). Read the governing package contract before changing behavior.
- Inspect the worktree and preserve unrelated changes. Keep changes small and update affected docs in the same task.
- For complex work, follow the [execution plan guide](docs/PLANS_GUIDE.md). Before finishing a code change, check the [quality targets](docs/QUALITY_SCORE.md). Use the [ADR guide](docs/ADR_GUIDE.md) for consequential decisions.
- Keep prompts, policies, and architectural rules in the repository. Follow the [Cargo workspace](crates/AGENTS.md), [controller](crates/controller/AGENTS.md), [Bitwig adapter](crates/controller/adapters/bitwig-ctl/AGENTS.md), and [Kontakt adapter](crates/controller/adapters/kontakt-ctl/AGENTS.md) instructions as applicable.
- Use durable-documentation for architecture. State the agreed design in present tense and omit project version labels, implementation history, and plans.
- For contribution, security, and release work, use [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), and the [CI](docs/CICD.md) and [release](docs/releases/README.md) guides.

## Bitwig project layout

- For any Bitwig operation, including adapter development and smoke tests, follow the [single-instance preflight](skills/bitwig-launcher/SKILL.md#single-instance-preflight). Reuse the running application across all displays and profiles. Xpra is prohibited.
- Follow the [production project layout](skills/score-intake/references/project-layout.md#bitwig-project-layout) for saved projects, source scores, and evidence.
- Keep live operations within the user's requested musical task. Verify edits in the current session. Closing, reopening, or switching the active project, and restarting Bitwig or its services, require an explicit user request for that action.

## Music-production routing

- Open the owning skill before acting. Use its documented Workbench entry points, including `convert` for score conversion and `midi-bridge` for native Bitwig operations. Never bypass purpose-built tooling with desktop input, third-party converters, private host calls, or ad hoc scripts. Use computer interaction only for the specific operations routed to it by the owning skill. If an operation fails, diagnose it through that skill and report any missing capability.
- Keep music-production skills, their references, and their authoring notes within the production documentation. They must never route to repository development documentation or source code.
- Put skill execution instructions in `SKILL.md` and directly routed references. Reserve skill `SPEC.md` and maintenance sidecars for authoring and review.

| Requested operation | Start here |
| --- | --- |
| Work spanning multiple musical operations | [Bitwig Compose](skills/bitwig-compose/SKILL.md) |
| Set up the project and preserve the source score | [Score Intake](skills/score-intake/SKILL.md) |
| Convert a score, import a rendition, or realize an orchestration | [Score Realization](skills/score-realization/SKILL.md) |
| Locate, select, inspect, or audition native material | [Bitwig Explore](skills/bitwig-explore/SKILL.md) |
| Edit notes, per-note expression, or automation | [Bitwig Edit](skills/bitwig-edit/SKILL.md) |
| Analyze the score or arrangement, or develop performance cues | [Score Analysis](skills/score-analysis/SKILL.md) or [Performance Cues](skills/performance-cues/SKILL.md) |
| Initialize a library, load an exact Kontakt patch, or configure a sung phrase | [Instrument Intake](skills/instrument-intake/SKILL.md), [Kontakt Setup](skills/kontakt-setup/SKILL.md), or [Kontakt Syllable Builder](skills/kontakt-syllable-builder/SKILL.md) |
| Mix or export the arrangement | [Bitwig Mixing](skills/bitwig-mixing/SKILL.md), [Audio Export](skills/bitwig-export-audio/SKILL.md), or [MIDI 2.0 Export](skills/bitwig-export-midi2/SKILL.md) |
