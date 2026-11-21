# Native mixing observations and audio export

Accepted scope: the 2026-11-18 user request admits native mixer reads, native
audio export, reusable mixing/export skills and a complete mixing pass on the
intended open project. This deliberately extends ARCHITECTURE.md's former
MIDI-only boundary. It does not admit generic native property export.

## Contract

`controller mix read --output FILE` returns one stopped-document snapshot of
the project hierarchy, master, mixer settings, sends/output routing and ordered
device identities/activation. Use existing qualified Paths. Include document
generation, scope revision, capture time, stopped transport position and host
identity. `beatSeconds` maps zero, the current position and existing cue beats
through the native tempo map; join it to the separate cue read for musical
labels. It does not expose another automation history. Capture time is an
observation envelope excluded from scope comparison. Volume exposes native physical
value, linear amplitude and dB (null at silence); pan exposes its native range
and mode. Distinguish stored values from effective playback/modulated values;
do not imply instantaneous settings describe an automated performance.
Unsupported fields carry explicit limitations. Never export reflected graphs,
plug-in blobs or duplicate complete clip histories.

The requested live mixing pass also needs bounded mixer writes:
`controller mix set --input FILE --output FILE`, with fresh expected snapshot
and absolute track volume/pan changes. Only those two fields are admitted.
Require unchanged complete mixer scope, stopped transport, no recording,
automation writing, render or save. Refuse automated/modulated parameters.
Preflight the complete batch before one native Undo group; return independent
readback and explicit failure, including partial outcomes. Do not replay or
automatically Undo an uncertain write. Preserve notes, expressions, existing
automation, receiver structure and all unrequested mixer settings.

`controller export-audio start --input FILE --output FILE` admits an explicit
range in quarter-note beats, exact track Paths (including groups/returns/master),
fresh document/scope revision, fresh output directory, format/sample rate/bit
depth, dither, pre/post-fader and realtime options. Resolve targets before any
queue/settings changes; never silently export the master instead of a missing
track. Expose only options verified against the current native export engine;
reject unsupported combinations rather than ignoring them. Initial production
format is WAV; other UI formats are reported as unsupported.

Return an operation identity immediately. `export-audio status` observes that
same operation, native progress/completion/error and each exact output path.
No blocking wait on the document thread. Poll status without resubmitting jobs.
Preserve source musical state and use task-owned export settings without altering persistent export preferences. Existing output files are never overwritten. A completed
native job is separate from independently verified WAV structure, signal,
isolation, tails and subjective musical approval.

## Verification

Qualify volume conversion against native display/engine conversion. In a
disposable project prove exact-path selection, invalid/stale refusal before
mutation, mixer readback and Undo/Redo, filtered export isolation, format/range
and option behavior. Run Rust workspace checks and both Maven profiles. Deploy
matching immutable wire builds before any intended-project operations. Retain
before/after mixer, complete musical content, export receipts and audio metrics
with the project. Do not certify listening or acoustic behavior from settings.
