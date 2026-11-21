---
name: instrument-intake
description: Initialize an installed Kontakt library with verified one-time batch resave and an evidence-backed INSTRUMENT.md from manuals and actual patch inventory. Revalidate receipts to avoid unnecessary resaves. Does not select instruments for a piece or assign receivers in Bitwig.
---

# Instrument intake

Follow [the shared workflow](../WORKFLOW.md). Produce INSTRUMENT.md at the
requested sample-library root, usable by realization and setup. A library is the
documentation unit; a patch is an entry, not automatically its own document.

For a documentation-only request, review the requested manuals/patches, preserve
processing eligibility and receipts, and return without invoking batch resave.

Resolve library -> [review processing coverage](references/batch-resave.md) ->
completed: reuse; missing: back up and initialize once;
failed/partial/unknown attempt: reconcile original operation before further writes.
Then document capabilities. Read that reference before any preprocessing command
or readiness claim; it owns supported execution, tool gaps and the INSTRUMENT.md
processing marker.

When called by kontakt-setup, preserve the caller's exact PERFORMANCE.md bindings
and return to that caller after initialization. Do not choose patches, channels
or tracks. Reuse matching documentation and retained sources; research only gaps
affecting the requested patches. Capability review may cover a subset, but the
existing resave operation processes the complete top-level library folder. State
that processing scope before mutation; do not imply a subset resave is available.

Patch identity is solely the NKI relative path beneath `INSTRUMENTS_ROOT_DIR`. Resaving
does not change a patch's identity when its relative path is unchanged.

1. Resolve the actual root under configured INSTRUMENTS_ROOT_DIR and read its local
   instructions and existing documentation. Inventory actual .nki files with
   relative paths; distinguish edition/version, player requirements and resource
   roots. Search the directory for manuals, articulation guides and preset docs
   before searching remotely. Source NKI writes are confined to the verified
   batch-resave procedure; do not edit sample content or customize patches.
2. Find the applicable official vendor manual: search local files, official web
   documentation/downloads, and relevant music forums for links, version clues
   and reported exceptions. Read the manual's actual relevant pages, not just
   snippets. On an already documented matching edition, reuse retained sources;
   research only missing/stale material. Forum claims remain attributed reports,
   not normative patch behavior. Prefer official sources when claims conflict.
3. Use [the template](assets/INSTRUMENT.md). Make the catalogue cover the actual
   library inventory, grouping identical documented behavior only when each exact
   NKI remains identifiable. If initialization is scoped to a subset, declare
   coverage and unreviewed files explicitly; do not label them usable implicitly.
   Translate abbreviations into conventional musical terms using evidence.
   Unknown behavior stays unknown; filenames alone do not prove true legato,
   range, dynamics layers or switching behavior.
   For covered Syllabuilder patches, follow [document syllables](../kontakt-syllable-builder/references/document-syllables.md),
   merge each exact patch's vocabulary and sequencing evidence into INSTRUMENT.md,
   then resume here. This documentation route does not load or customize patches.
4. Complete the template's **Conventional MIDI control map** before a covered
   patch is handed to another skill. Review the manual's controller chart, dynamics,
   expression, vibrato and pitch sections, then resolve exact-patch exceptions.
   Include explicit rows for CC1 (MW), CC2 (BC), CC11 (XP), pitch bend and velocity,
   plus every documented pedal, switch and other required MIDI control. Classify
   each as documented-supported, documented-unsupported, or unknown; an absent
   manual entry is unknown. Record exact patch scope, stock assignment, musical
   effect, range/units, default/reset, conditions and manual revision/page/section.
   Distinguish assignable/learned controls from stock defaults. Pitch bend needs
   center, range and bend sensitivity when documented; it is not a CC. Separate
   vendor facts from observed settings and project policy. Do not discover device
   macros/knobs unless explicitly requested for controls beyond conventional MIDI.
   Return table coverage and unresolved controls to callers. Missing/stale mappings
   trigger documentation-only intake, never a resave or speculative live mapping.
   Record material usage details: ensemble/solo identity, articulation, playing
   range and transposition, dynamic crossfade versus attack velocity, expression,
   keyswitches, legato triggering/overlap, release behavior, pedals, scripted
   patterns/text, microphones and required defaults where applicable. Cite manual
   revision/page/section per shared rule or exception. Distinguish musical effect
   from its control encoding and document units/ranges.
   Before handing a patch to selection or realization, identify its default
   dynamics mode per articulation: note velocity, CC1 crossfade, CC11 expression,
   or another documented control. Record CC1 and CC11 effects separately, other
   velocity roles, named sampled layers and numeric controller/velocity bounds
   for named dynamics. Cite the exact manual edition/section; group patches only
   where the evidence supports identical behavior. Portato, for example, must
   not inherit staccato's mode merely because its name includes “short”.
   Preserve stock defaults. When numeric layer thresholds are unpublished,
   record that gap and any separately evidenced calibration; do not present
   approximate project anchors as factory layer boundaries or import another
   patch/engine's control map. The caller
   applies the shared workflow's project policy in PERFORMANCE.md; INSTRUMENT.md
   records library facts.
   Keep controller-function evidence separate from numeric calibration: an
   explicitly documented CC1 crossfade remains documented-supported when layer
   thresholds/defaults are unpublished. Record those numeric gaps in their own
   columns, rather than downgrading the whole control to unknown. A genuinely
   unestablished exact-patch assignment remains unknown; do not infer CC11-only
   behavior from silence about CC1.
5. Record required processing eligibility separately from musical capability:
   library/relative patch paths, completed processing records and covered scope.
   Reuse successful path-based coverage.
   ERROR, missing coverage or incomplete outcomes require the applicable branch
   in [the procedure](references/batch-resave.md). Respect the caller's family
   constraints and preserve its musical bindings.
6. Merge into existing INSTRUMENT.md without erasing empirical findings. Library
   AGENTS.md should route to this description; piece-specific taste statements
   must not become universal capability rules. Preserve unrelated instructions.
   Record conflicting old guidance and its actual evidence rather than blindly
   promoting it. Verify paths and inventory coverage; report unknowns affecting
   selection/setup. Return the library root, covered relative patch paths,
   INSTRUMENT.md, processing evidence and actual outcome to the caller.

On failure, retain the original result, logs, backup location, observed partial
progress and owned-process/dialog state exposed by the supported tool. Mark
initialization unverified and return a failed dependency. Reconcile the original
process before further source writes. If reconciliation is unavailable, keep
the affected processing pending and retain the original evidence.

Initialization means documented capability, not an audition, loaded receiver or
configured patch. If no applicable manual can be obtained, record searched scope,
retain an explicitly incomplete document and report which choices remain blocked.
Do not turn this into full-library acoustic qualification or a project orchestration
task. On repeat use, check inventory/edition changes and refresh affected facts only.
