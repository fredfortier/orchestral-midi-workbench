# [Sample library — edition/version]

## Identity, coverage and sources

INSTRUMENTS_ROOT_DIR: [root established by project instructions, configuration or user].
Library root: [library directory beneath INSTRUMENTS_ROOT_DIR].
Patch paths: [NKI paths relative to INSTRUMENTS_ROOT_DIR; sole patch identity].
Player: [documented Kontakt version/edition requirements].
Inventory: [date, file count, covered subset or complete coverage, changes].
Official sources: [local manual path, title/revision, source URL, applicable pages].
Supplemental reports: [forum URL/date/author and precise evidentiary limits].

## Batch resave

```yaml
batch_resave:
  status: unverified
  evidence: null
  completed_at: null
  scope: null
  files_expected: null
  files_rewritten: null
  process_exit_code: null
```

Populate from completed path-based processing evidence using the instrument-intake
procedure. Review covered library/relative patch paths and reuse completed work.
The marker summarizes processing evidence; its presence alone does not establish
completion.

## Patch catalogue

| Patch key | Relative NKI path | Instrument / ensemble | Articulation or technique | Dynamics / velocity behavior | Range / transposition | Required setup | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |

Patch keys are local catalogue labels, not replacement identities. Preserve the
exact NKI path relative to INSTRUMENTS_ROOT_DIR and explain its musical
meaning separately. Identify multi-articulation and scripted/phrase patches;
never imply one NKI always equals one articulation. Explicitly mark unknowns.

## Conventional MIDI control map

| Exact relative NKI / articulation scope | Control / message (MW CC1, BC CC2, XP CC11, pitch bend, velocity, other CC) | Status and stock assignment | Musical effect / interactions | MIDI range and units | Stock baseline / reset | Conditions / exceptions | Manual title, revision, page / section; observed evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |

[Required rows per covered scope: CC1, CC2, CC11, pitch bend and note velocity;
add documented pedals, switches and other MIDI controls. Status is
documented-supported, documented-unsupported, or unknown. Manual silence is
unknown. Group exact patches only with evidence of identical behavior. Record
stock versus MIDI-learn assignments, dynamics versus attenuation, independent
vibrato/attack roles and interactions. Distinguish dynamics and expression
baselines. Include load/reset, behavior before the first MIDI message and
neutral/unity value when established. For pitch bend record center and documented
semitone range/sensitivity, or unknown. Values here are MIDI units (CC 0–127,
velocity 1–127 for sounding notes, pitch bend 0–16383/center 8192 where applicable);
native track automation uses its observed physical units, with explicit boundary
conversion. Cite each row. Current UI settings and empirical exceptions remain
separately identified. Downstream workflows consume this table; incomplete rows
return to documentation-only intake. Project channel routing and artistic anchors
belong in PERFORMANCE.md.]

## Additional performance behavior

[Document legato triggering/overlap, releases, sampled layers, range, keyswitch
notes, microphones and text state not already covered by the control map. Cite
exact patch scopes and sources. Distinguish sampled timbre from attenuation;
unpublished numeric thresholds stay unknown. Project anchors and baselines belong
in PERFORMANCE.md. Do not duplicate the MIDI control map here.]

## Syllabuilder patches

[Include only for applicable patches; follow the routed syllable-documentation
procedure. Exact sample labels, patch scopes and evidence are required.]

| Exact relative NKI(s) | Builder family / section | Ordered sample labels | Sustain/staccato availability | Evidence |
| --- | --- | --- | --- | --- |

| Exact patch scope | Word/cell capacity | Advance / wrap / reset | Numeric selectors and HLS assignments | Entry/release and persistence limits | Evidence |
| --- | --- | --- | --- | --- | --- |

## Resources and configuration requirements

[Sample/resource roots, missing-content considerations, player restrictions,
required script modes and persistence limits. Preprocessing evidence is above;
the executable procedure remains in the intake reference.]

## Musical selection guidance and limitations

[Evidence-supported differences among variants; ranges, transitions, dynamics,
sampled patterns, diction limitations. Project choices belong in PERFORMANCE.md.]

## Verification status

[Documented facts; independently observed behavior with environment/evidence;
unreviewed patches; unresolved conflicts and missing manual coverage.]
