# Bitwig composition workflow stories

Use this map when a musical result crosses skill boundaries. Each owning skill
supplies its execution procedure and verification. Pass the current project,
intent and evidence to the next owner. Resume at the first missing result.

| Workflow story | Handoff |
| --- | --- |
| Supplied score to a playable project | Intake the score and initialize intended libraries. Realize the reviewed performance into MIDI2, verify native import, assign the exact receivers, then place cues or mix when requested. [Score and performance](score-and-performance.md) |
| Prepare a sung phrase in context | Carry lyric coverage and the receiver from realization to setup and the builder. Return the verified phrase and control map to the native editing owner. [Score and performance](score-and-performance.md) |
| Audition and apply a repair | Resolve the native passage, checkpoint the main project, audition, apply to the agreed scope and independently verify. [Compose and revise](compose-and-revise.md) |
| Analyze the current performance | Capture the qualified native scope, interpret and export it, then analyze the reconstructed score. [Readback](readback.md) |

## Route by outcome

```text
REQUEST
 +-- compose across workflows --------> bitwig-compose -------> STOP after requested stages
 +-- establish score workspace ------> score-intake ----------> STOP
 +-- document installed library -----> instrument-intake -----> STOP
 +-- configure vocal syllable phrases -> kontakt-syllable-builder -> STOP
 |    +-- missing patch facts --------> instrument-intake --> return
 |    +-- stock receiver assignment --> kontakt-setup -----> return
 |    `-- native note/control edits --> bitwig-edit -------> return
 +-- realize/revise score -----------> score-realization
 |    +-- missing workspace ---------> score-intake -------> return
 |    +-- missing relevant docs -----> instrument-intake --> return
 |    `-- specify -> convert -> native import -> verify ------> STOP
 +-- research/place cues ------------> performance-cues ------> STOP
 |    `-- requires intended project open + relevant MIDI loaded
 +-- assign/load instruments --------> kontakt-setup ---------> STOP
 |    +-- contextual scope -> bitwig-explore -> resolved owners -> return
 |    +-- missing/stale initialization -> instrument-intake -> return
 |    `-- requires settled receiver specification; setup owns layer/slot materialization
 +-- mix / diagnose balance ---------> bitwig-mixing
 |    `-- audio evidence ------------> bitwig-export-audio --> return
 +-- export native audio -----------> bitwig-export-audio ----> STOP
 +-- export MIDI2 / reconstruct score -> bitwig-export-midi2 --> STOP
 +-- analyze score / composition ----> score-analysis --------> STOP
 |    `-- current DAW score needed --> bitwig-export-midi2 --> return
 +-- inspect/navigate ---------------> bitwig-explore --------> STOP
 `-- edit notes/expression/curves ----> bitwig-edit -> verify --> STOP
```

Use the saved musical project's own instructions to resolve its project paths.
Use the owning production skill and current CLI help to establish available operations.
Do not invent command names or install another control surface to fill a gap.

## Meaning and authority

```text
original score + user direction + documented library capabilities
                              |
                       PERFORMANCE.md
                              |
                  explicit supported conversion inputs
                              |
                           convert
                              |
                       MIDI2 + companion
                              |
               native placement and clip replacement
                              |
                     independent native readback
```

| Authority | Owns | Does not prove |
| --- | --- | --- |
| User direction | Outcome, scope, accepted departures | Successful execution |
| Original score | Authored notation, source identity | Every editorial decision |
| Official manuals + observed inventory | Documented behavior + available files | Suitability for this passage |
| Library INSTRUMENT.md | Cited capability description; observed exceptions distinguished | Project instrument selection or loaded state |
| Project PERFORMANCE.md | Normative musical choices, receiver bindings, approximations, cues | Actual host state |
| MIDI2 + companion | Executable rendition + exact intended patch/routing bindings | Audible support or current host state |
| Fresh observations / retained receipts | Measured state / historical operation evidence | Intent or subjective approval |

The agent translates musical choices into supported inputs. Neither CLI executes
arbitrary prose. Before conversion/import/setup, compare the affected musical
specification with the actual typed inputs and companion. Record source locations and declared revisions in operation evidence. This is an agent review obligation;
there is no general automatic PERFORMANCE.md compiler/agreement checker today.
Conflicts require reconciliation, not silently choosing the easiest authority.

PERFORMANCE.md is the current musical specification. Preserve relevant older
commentary as context; do not create a competing assignment ledger. Exploration
and bounded native edits do not require adopting the rendition workflow.

## Domain identities

```text
score part / voice               authored musical identity
sample library                   installed collection; one INSTRUMENT.md per root
Kontakt instrument patch (.nki)  sampled/scripted resource, possibly multi-articulation
receiver binding                 project track/channel -> stock patch + native routing
Instrument Layer                 Bitwig device containing layer chains (CLI: slots)
Arranger Cue Marker              native landmark linked by permanent cue key
```

A MIDI channel is an address, not an instrument. A UMP group is not a Bitwig group.
CLI instrument channels are 1..16; companion/native note channels are 0..15.
Kontakt input A/1 is a separate receiver address. Track identity, binding key,
patch identity, channel, and temporary native IDs are distinct. Resolve qualified
owners afresh; duplicate display names are possible. Musical names such as
`Violins I` / `Pizzicato` link to stable binding keys in PERFORMANCE.md; file slugs,
bank positions and UUIDs do not substitute for musical labels.

PERFORMANCE.md owns all orchestration choices. Kontakt patch identity is solely
the `.nki` path relative to `INSTRUMENTS_ROOT_DIR`.
Identify resources by
source location, relative instrument path and declared version; establish edit
authority through qualified native owners, explicit expected values, native
change information and operation IDs. Reuse completed initialization by covered
paths and observed operation outcome.

Per-note expression and track/device automation have separate ownership, units,
and write authority. Source beats, arranger beats, bar numbers and seconds differ.
Stored notes are not expanded loop occurrences. Sampled legato is not a synonym
for ties, slurs, overlap, or an instruction to connect every note.

## Artifact and mutation ownership

| Skill | Owns | Completion evidence |
| --- | --- | --- |
| [bitwig-compose](../SKILL.md) | Requested multi-stage composition outcome | Owner evidence for each requested result |
| [score-intake](../../score-intake/SKILL.md) | Saved workspace, original score, project instructions | Actual saved path + unaltered source + recorded project identity |
| [instrument-intake](../../instrument-intake/SKILL.md) | Library documentation and verified one-time batch preprocessing | Completed path-based processing coverage + INSTRUMENT.md marker + cited capabilities/unknowns |
| [score-realization](../../score-realization/SKILL.md) | Musical realization, patch selection, conversion, native notes/controls/channel layers | Intended and imported content/routing agree; save confirmed |
| [performance-cues](../../performance-cues/SKILL.md) | Scholarship, arc, cue sheet, markers | Current MIDI scope + cited decisions + independent marker readback |
| [kontakt-setup](../../kontakt-setup/SKILL.md) | Existing controller assignment and exact NKI loading | Integrated assignment/load result and native routing readback; no customization |
| [bitwig-mixing](../../bitwig-mixing/SKILL.md) | Musical balance, scoped mixer changes and audio-backed diagnosis | Native readback, musical preservation, verified exports and listening limits |
| [bitwig-export-audio](../../bitwig-export-audio/SKILL.md) | Exact native channel/range/options export | Native final outcome plus independent file/audio checks |
| [bitwig-explore](../../bitwig-explore/SKILL.md) | Discovery, deliberate selection, reads, requested audition | Complete intended scope; retained observation; bounded finding |
| [bitwig-edit](../../bitwig-edit/SKILL.md) | Authorized notes/expression/automation changes | Requested result + independent full preservation readback |

PERFORMANCE.md uses the [shared template](../../score-intake/assets/PERFORMANCE.md).
Intake establishes identity; realization owns realization/bindings; cues owns
interpretation/cue sections; setup owns configuration evidence. Every writer
preserves the other sections and user commentary. Setup may report a required
specification correction but cannot silently change musical intent. Cues do not
automatically enact their directions as notes, automation or tempo changes.

## Readiness and change propagation

```text
workspace     musical spec     native content     receiver setup     cue placement
   [ ]            [ ]              [ ]                 [ ]               [ ]

audible behavior [ ]     user musical approval [ ]     (separate evidence)
```

These are independent facts, not one global READY flag. Require only those needed
by the requested task. Relevant MIDI can exist without this workflow's companion;
exploration, cues and bounded native edits do not require regenerating it.

| Change / condition | Route |
| --- | --- |
| Same intent/resources and matching current state | Verify/reuse; no duplicate project, group, slot or load |
| Generated rendition changes | Realization; reconvert; supported reimport only |
| Authorized native notes/expression revision | Editing; required setup returns to editing; record bundle divergence without automatic reconversion |
| Patch changes control law | Revise bindings + conversion; reconvert; reconcile setup |
| Patch changes, performed messages unchanged | Revise companion/specification; reconcile setup; no automatic reconversion |
| Cue-only interpretation/placement change | Cues; musical changes require realization/editing scope |
| Native user edits diverge from imported bundle | Preserve and reconcile before replacement; no automatic reverse conversion |
| Unknown operation outcome | Original operation status + fresh read; no blind replay or Undo |
| Missing/ambiguous owner, shared-source or loop conflict | Preserve state; resolve scope or report precise unsupported operation |
| Missing required capability | Keep affected result pending; continue independent authorized work |

Changed executable decisions invalidate affected plans. Changed arrangement can
invalidate cue placement. Changed patches can invalidate acoustic evidence.
An updated library document is evidence to review affected choices, not permission
to reorchestrate existing projects. Audition uses exploration; corrections route
to editing, realization or setup. A full listening review is not an automatic task.
