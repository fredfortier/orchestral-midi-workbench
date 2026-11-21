# Sources and authoring decisions

Qualifying behavior: answer a bounded musical-analysis request from a current
Bitwig export or an explicitly supplied historical score, choosing only relevant
analysis, preserving provenance and separating measurements from interpretation.
The skill neither changes music nor certifies playback or artistic quality.

Shape: reference-backed expert with an inline intent router, shared loading leaf
and six distinct question-specific leaves. No new runtime scripts, workers,
notebook services or persistent Python environment. The existing comprehensive
audit example is optional only when full source/capture verification is requested.
Registration follows repository skill-directory + user `.agents/skills` symlink
convention; metadata is discovery text, not a provider-specific execution API.

## Primary source evidence

Inspected music21's official User Guide notebook sources at master revision
`40e95ae567fe8b8e35317447dc74d5cdbb2fb83d` (2026-11-19):
https://github.com/cuthbertLab/music21/tree/40e95ae567fe8b8e35317447dc74d5cdbb2fb83d/documentation/source/usersGuide

These are documentation/API evidence, not an upstream agent skill or workflow
being copied. Runtime prose and bounded examples are original adaptations of the
mechanisms below; narrative corpus examples and notebook display machinery were
not imported. Pinned runtime dependency is music21 10.5.2 (latest PyPI verified
in this session's preceding analysis work); upgrading is not a routine runtime step.

| Notebook | Inspected mechanism → local decision |
| --- | --- |
| `usersGuide_06_stream2.ipynb`, flattening | Keep part identity and score-relative offsets; retain the flat stream while accessing object offsets. |
| `usersGuide_31_clefs.ipynb`, ties and partial chord ties | Distinguish pitched segments, attacks and durations; residual partial ties limit claims. No automatic re-notation. |
| `usersGuide_14_timeSignatures.ipynb`, beats/beatStrength | Report the metrical model without equating weak attacks with syncopation or quarter units with compound beats. |
| `usersGuide_18_intervals.ipynb`, Interval/direction/isStep/consonance | Explicit monophonic selection; locate leaps and do not infer playability from counts. |
| `usersGuide_22_graphing.ipynb`, piano rolls and distribution comparison | Optional single register plot; control instrumentation/doubling when comparing pitch distributions. |
| `usersGuide_09_chordify.ipynb`, moving-note slices and passing seventh chords | Limit first-pass Roman output; chordified snapshots do not identify harmonic rhythm by themselves. |
| `usersGuide_15_key.ipynb`, correlation, alternatives and key context | Global/local key context is explicit; coefficients are not probabilities and changing window winners do not establish modulations. |
| `usersGuide_23_romanNumerals.ipynb`, flexible chord/key labeling | Preserve raw pitches and contextual alternatives; no functionality/composition-quality score. |
| `usersGuide_61_trees.ipynb`, verticalities and timeToNextEvent | Weight density/consonance by elapsed duration rather than slice count or toChord's default duration; trace named sustained voices. |

Additional API inspected: music21 10.5.2 `voiceLeading.py`:
https://github.com/cuthbertLab/music21/blob/v10.5.2/music21/voiceLeading.py
`VoiceLeadingQuartet.parallelFifth/parallelOctave` supply candidates only, gated by
monophonic part identity and uninterrupted sounding pairs. Tree `.part` access and
all actual code examples are execution-checked, not inferred solely from prose.

## Local evidence and coverage

The preceding Vivaldi export demonstrated real traps: 4,193 attacks versus 4,234
pitched segments, explicit reset commands, a native tiny note, placement clipping,
and Roman-label sensitivity to spelling. These motivate generic integrity checks;
no piece names, pitch expectations or mappings appear in runtime instructions.
Freshness belongs to `bitwig-export-midi2`; current-score analysis invokes that
owner, supplied historical analysis does not. Controller/convert architecture and
`examples/music21_analysis.py` own full comparison and representation details.

Discovery walkthroughs: should trigger for “analyze current orchestration”,
“why this suspension?”, “Roman analysis of this MXL”, “awkward melodic leaps”,
and “check whether export lost notes”. Should not trigger as the action owner for
“rebalance audio”, “load Kontakt”, “edit these notes”, or “research historical
performance cues”. Combined requests return to their named owner at each boundary.
A valid XML with stale project scope is a defeating case: freshness must pass
before current-piece conclusions. A polyphonic keyboard is a defeating case for
melodic/parallel metrics: fail that branch rather than invent a voice. Supplied
historical MXL remains valid without a DAW connection.

Validation covers frontmatter, links, registration target, actual snippet execution,
and boundary probes described at completion. Author walkthroughs are not a
comparative agent evaluation and do not prove discovery/model behavior improved.
No general harmonic-ground-truth or instrument-playability classifier is claimed.

Executed validation (2026-11-19): all Python setup/analysis/plot blocks passed
against the current case MXL in ephemeral `uv run --with music21==10.5.2`
processes. Synthetic probes confirmed two tied segments count as one attack and
eight quarter beats, rejected chordal and overlapped melodic inputs, and measured
one four-beat sustained part as four part-density beats. The optional plot was
written to `/tmp` only. `quick_validate.py`, direct-route/local-link checks and
registration-target readback passed. No durable tests or new runtime script were
added; no live DAW recapture was needed to validate authoring snippets.
