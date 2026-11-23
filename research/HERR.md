# Herr, unser Herrscher case study

## Musical result and evidence

The target is the existing `bwv-245-herr.bwproject`. Intake verified the
159,982-byte MXL against the original download. Its SHA-256 is
`77cd44eaa593b2df2c08d601768d73cc694beca91c84ff51742d6e4fe826d4c6`.
The project holds the original, `SOURCE.md`, the musical decisions in
`PERFORMANCE.md`, and the run evidence in `evidence/herr-20261122T224500Z/`.
The case study omits the project, local sample files and reference audio.

The original has 95 written measures and sixteen staves. Playback analysis
found identical pitches, starts and ends on the aggregate continuo and
bassoon staves. The rendition uses fifteen tracks. The supplied MuseScore
export gives Fine at the end of written bar 58, after the bass has begun
*Zeig uns durch* and the accompaniment has entered the contrasting section.
The working score moves the playback Fine to the preceding G-minor cadence.
The first pass still plays all of the middle section. The MIDI2 rendition has
16,046 notes, with Fine's final sounding release at beat 610 and a duration
of 611 quarter beats. Preserve the original for comparison.

The user chose legato by default and a Corboz-like performance after the
initial sketch. The score's slurs, repeated attacks and written staccatos
guide the patch assignments and gate decisions. Quarter note = 56 uses the
1978 Corboz recording's 10:56 duration as a broad reference. The project
retains separate sampled-intensity and expression contours. The text remains
in the source while the stock vocal receivers use fixed Ah and Oh. See
[Dirksen on the oboes and historical forces](https://www.pieterdirksen.nl/Essays/Johannespassion.htm),
[Hazelgrove, pp. 44–51, on form, harmonic rhythm and texture](https://libres.uncg.edu/ir/uncg/f/Hazelgrove_uncg_0154D_11018.pdf),
and [the 1978 Erato recording](https://www.qobuz.com/us-en/album/bach-johannes-passion-bwv-245-michel-corboz/u11kv7r1alxyb).

## Workflow corrections

| Finding | Correction and reason |
| --- | --- |
| The compose route allowed receiver and arrangement setup to get ahead of a changing rendition. The first trial created empty tracks and loaded one wind patch. Subsequent musical direction changed the patch, tempo and final cutoff. The user removed the trial group. | Settle form, forces, articulation, dynamics and exact stock controls first. Complete `INSTRUMENT.md` coverage, write the musical use in `PERFORMANCE.md`, compile and authenticate MIDI2 plus companion, then create the destination and import. Setup follows native musical readback. An optional companion group lets the saved project root be the destination. |
| Rules drafted from decoded notes missed the exact pre-adaptation descriptor for the keyboard staff. An inspection helper exposed the mismatch but was unsuitable as the production route. Voice-by-note selectors also expanded the first plan to 3,800 rules. | `convert --inspect-rendition` exposes the predicates and written voices directly. `sourceVoice` reduced this score to 53 rules with the same musical destinations. The final run uses the documented CLI throughout. |
| The typed converter explicitly refused the source's preview volume/pan and routed prose. Removing them from a working score lost useful detail from the compiled companion's source. | An explicit `archiveScorePreview` policy keeps those fields in the preserved score and substitutes the planned performance. Other unassociated channel events still refuse. |
| The rule-level dynamics path could write CC on multiple note routes, and a native gate pass would have split the musical authority between Bitwig and the bundle. | Compile the reviewed note endings and independent CC1/CC11 curves. The companion distinguishes musical note/receiver bindings from shared track CC routes, and the MIDI events themselves use channel 1 for those curves. Native placement transfers exact decoded values. |
| Selecting a group header left OSC track creation at project level. Entering the group gave the new child the right ancestry. A ninth track could lie outside the eight-name OSC bank, and the focus action could leave the visible Arranger editor reported as `none`. | Verify ancestry after one creation, page and refresh names before indexed rename, and use native focus and range readback before the empty-clip action. A direct Arranger click established focus when the configured toggle had a different effect. |
| A slot rename on wire 74 changed the name but attempted selection by the pre-rename path. The result stayed unknown and blocked other document operations. Slot ID and devices survived the rename, and the next assignment reported no dispatch. | Resolve the unchanged enclosing path and select the already resolved native slot by object. The code change has production/harness build coverage. Restarting the installed Notes Document Adapter in Bitwig restored native selection and writes. The successful import used fresh track and slot identities. |
| One Berlin Woodwinds folder had historical failures and no current completion receipt. The exact legato variants needed their own stock MIDI evidence. | Instrument intake backed up and initialized all 51 NKIs. The tool confirmed exit 0 and path coverage. Exact legato ranges and transition families were added to its `INSTRUMENT.md`, with CC11 documented and uncertain slot-specific controls explicit. |
| The Strezov Harpsichord successfully opened its larger panel, and the loader's same-size pixel comparison failed after submission. The service blocked subsequent writes while the outcome was unknown. | Retain the original receipt. Inspect the exact Kontakt editor and verify the named Harpsichord rack and its loaded sample memory, then read its native routing. Save the project and restart only the controller service before resuming with untouched receivers. The loader still needs a resize-tolerant confirmation that can distinguish a real patch change from a window resize. |
| The Celli file picker missed the 30-second wait. A read-only Win32 observation later matched the modal and editor owner handles and found an empty filename. The stranded editor also obscured the next receiver's render. | Dismiss the unsubmitted modal and close the empty editors. Ship a 75-second owned-dialog window without relaxing process or owner identity. The voice-2 continuation then verified the exact filename, rack, editor closure and native routing. |
| The first pass preserved source velocity on the fixed-vowel choir and counted native MIDI and audible stems as delivery evidence. The user confirmed that note velocity has no audible effect on these receivers. | Distinguish stored note data, patch-specific control response and authored phrase dynamics. The four vocal parts still need continuous dynamics and expression. Establish the actual factory control response before encoding and verifying the vocal curves. |

## Boundaries and acceptance

`convert` owns performed score and receiver intent. The controller owns exact
native reads, musical writes, cue markers and the integrated assignment
transaction. OSC owns supported session actions and their feedback. Its send
receipt confirms submission. Bitwig's native readback establishes the document
effect. The Kontakt loader submits the prescribed stock NKI inside the
controller-owned editor. Use the production skills for each handoff.

Close with independent evidence for MIDI2 and companion, native clip/curve
content, native markers, receiver/sample response, saved persistence and
representative playback. Keep unverified steps and audition limits visible.
The revised checklist has been reviewed against this case. A fresh-run
comparison is still open.

The converter inspection, compile, MIDI validation and companion-authenticated
decode passed on the final 15-track bundle. An independent comparison matched
all performed source-note identities and endings and all 2,656 encoded CC events
against the 15 planned curves. The final companion targets the project root.
Following the adapter restart, the 15 root-track imports matched the compiled
notes and automation under independent native readback. The 13 cue markers also
matched a separate native read by name and arranger beat. The existing `FX 1`
track survived the placement.

The saved project has 21 routed Kontakt receivers. The integrated loader
verified 20 exact NKI assignments. The Harpsichord was independently
reconciled by its editor and native routing, then produced its own audio
stem. The case-specific loader build was deployed with the longer owned
dialog wait. The final save and native note, curve, and cue checks passed.

All 15 part stems sounded at the first choral entry. A second render
confirmed an 8–10 dB lift of the string tracks, a 4–6 dB lift of the upper
winds, and a 7 dB reduction of the bassoon. The opening, first choral
entry, principal crest, and Fine master excerpts all stayed below
−12 dBFS peak. Fine had a measured decay. The playable project was left
stopped at bar 1. The project evidence records the scope of the renders
and the remaining listening questions.

## INSTRUMENT.md control audit

The companion names 21 receivers and 16 distinct NKIs across five libraries.
Every used relative path exists and appears explicitly in its library document.
Four completed path-based receipts cover the used paths. Berlin Strings has
historical folder-wide success and a later failed rerun, both retained in its
library document. That historical evidence has no per-path receipt. This audit
reused existing processing evidence and made documentation edits only.

| Library | Used NKIs | Exact-scope control evidence |
| --- | ---: | --- |
| Berlin Strings | 8 | Capsule Main Knob, printed p.29, sets legato to XFade and staccato to velocity. The controller table and Common MIDI CCs document CC1 as the XFade controller and CC11 as patch volume. The vendor's Legato guide distinguishes velocity-based attack and transition selection. |
| Berlin Woodwinds Soloists | 3 | The exact Soloists I/II entries specify one recorded mf legato layer. Capsule documents stock CC11 patch volume. The exact factory CC1 mode and filter response and the legato velocity law remain unknown. |
| Árva | 2 | The installed manual, PDF p.4, identifies the true-legato family. CC1 and CC11 remain unknown on both exact Boys Ah patches. The user's no-velocity-effect report is separate from the factory map. |
| Storm Choir Ultimate | 2 | The installed manual's Legato Patches and Patch List distinguish Men Ah, Men Oh and the separate Oh–Ah morph. PDF p.7 describes Velocity Dynamic Influencer. That feature and the syllable CC1 crossfade do not qualify the two fixed vowels. CC1 and CC11 remain unknown on both. |
| Strezov Harpsichord | 1 | The installed manual documents register switches and C1 round-robin reset. The required CC and velocity entries record factory response as unknown. |

Intake bears the burden of establishing the stock assignment and audible
function for each exact NKI and articulation, with a source locator or controlled
observation. CC numbers alone establish MIDI message identity. Numeric boundary
gaps can coexist with a known function. Older library AGENTS.md heuristics on
Árva dynamics, woodwind velocity, and harpsichord velocity are guidance and do
not establish stock response for these exact NKIs.

Realization bears the burden of using applicable, supported standard controls
for the intended musical result. It must explain the phrase-level purpose in
PERFORMANCE.md, encode independent dynamics and expression where both are
supported, and verify channel, initialized values, routing and native curves.
A required effect without established controller behavior is an unresolved
delivery dependency. Audible stems and stored note velocities do not satisfy
that dependency on the four choir tracks. This audit establishes documentation
coverage for the 16 project patches. It does not qualify the four choir patches
for continuous-dynamics delivery or claim full-library capability coverage.
