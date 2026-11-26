# Performance — [work / movement / rendition]

## Identity and provenance

Project: [actual native path and identity].
Source: [stored original, source location, edition].
Current rendition: [revision and MIDI/companion links, or not yet realized].

## Realization

[Musical intent; authored instructions versus editorial choices; orchestration,
articulations, timing, dynamics, tuning and vocal-text fidelity as applicable.
Initially unfilled. Realization owns this section and the bindings below.]

[For a new or revised articulation plan, link the gesture map or record it here.
Include score and performed locations, evidence and inference, playing intent,
the receiver binding and its verification status for the requested scope.]

### Receiver bindings

| Binding key | Score part / voice and musical purpose | Track / layer chain | Note channel (1–16 display) | Instrument / articulation / dynamics behavior | Library patch key / exact NKI | Stock behavior / native routing | Evidence / approximation |
| --- | --- | --- | --- | --- | --- | --- | --- |

[Link library INSTRUMENT.md, exact executable instrument/binding identities and
NKI path relative to INSTRUMENTS_ROOT_DIR, which is the sole patch identity.
Setup assigns/loads stock NKIs without customizing their defaults. CC-controlled
bindings use native track-level MIDI automation. Record layer note-channel filters,
control-message passage, output remaps and Kontakt input separately. Explain
independent receiver instances and passage scope where relevant. Do not infer
dynamic meaning from CC numbers or filename tokens.]

### Expression control specification

| Binding / exact NKI and articulation | Documented dynamics and expression controls, stock baselines and velocity role | Named sampled dynamics and evidenced bounds | Project entry values and role of each control | Track MIDI lane / control channel / receiver routing | INSTRUMENT.md evidence and execution status |
| --- | --- | --- | --- | --- | --- |

[Realization fills every binding from instrument-intake's INSTRUMENT.md before
programming controls. Follow the shared workflow's expression contract: retain
the patch default, set note velocity for velocity dynamics, automate CC1 for
crossfade, and plan an independent CC11 phrase/level envelope where supported.
Follow the shared dynamics-and-expression reference: CC11 is neither fixed at
maximum by default nor a copy/transform of CC1. Specify separate musical roles,
passage anchors, initial values and ranges; a flat span needs a musical reason.
For a verified CC11-only patch, use that control without inventing CC1 support.
Consume the library's Conventional MIDI control map. Keep stock baselines
distinct from chosen entry values. Missing facts return to documentation-only
intake. Use track MIDI lanes, shared across related
slots where routing and control semantics permit. Do not create sender devices or
wrap automation in clips. Device macro/knob discovery is explicit opt-in only for
controls beyond conventional MIDI. Identify independent channel/control needs.
Give numeric units and distinguish vendor bounds, measured calibration and
editorial targets. Use a labelled project baseline if a supported control's
factory number is unpublished. A required effect with unknown controller
behavior remains pending.
Documentation, actual host configuration and audible response are separate facts.]

### Musical curve anchors and verification

| Track / articulation and required effect | Score passage and performed occurrence | Composer marking, score evidence or cited musical reading | Phrase role and justified direction | Control and entry, change, hold or release targets | Decoded MIDI and native readback / pending reason |
| --- | --- | --- | --- | --- | --- |

[Cover each required effect and each musically meaningful change. Link the
musical reasons to the interpretation notes below where applicable. Explain
holds in musically level spans. Group equivalent occurrences where the same
rationale holds. Check the required effects against both the decoded bundle and
native readback. Record affected scope and consequence for any pending effect.]

### Approximations and open musical questions

[Required, accepted, provisional, or unsupported; affected scope and consequence.
Do not infer user approval.]

## Interpretation and cue map

[Global arc, principal/subordinate climaxes, preparation and release. Cues owns
this section and its decisions/evidence; initially unfilled.]

| Cue key / exact Bitwig marker name | Musical function | Performance direction |
| --- | --- | --- |

### Interpretive decisions and sources

[Authored evidence, musicological citations with pages/sections, chosen reading.]

### Cue placement evidence

[Current note scope and mapping; proposed/verified/partial/stale; actual marker
readback links. Keys persist when positions change; retired keys are not reused.]

## Configuration verification

[Setup owns this section. Per binding: intended stock patch and native routing versus observed
rack, sample-resolution status, persistence checks, evidence and remaining gaps.]

## Listening and revision notes

[Requested audition findings, native edits/divergence, corrections, and explicit
user musical response. Structural verification is not acoustic validation.]
