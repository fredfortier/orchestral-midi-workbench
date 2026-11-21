# Choose the smallest musical phrase dictionary

For a request to realize existing lyrics with sampled syllables, default to the
closest evidenced sampled syllables and explicitly document phonetic losses.
Consider fixed-vowel legato for suitable complete melismas as described below;
do not ask the user to choose between this normal approximation and a wordless
performance. An explicit diction or approximation policy takes precedence.
Ask only when a remaining choice materially changes the intended text or musical
gesture, or no defensible approximation exists. State routine editorial choices
and continue independent work while any consequential question is pending.

Settle the pronunciation tradition and acceptable losses before optimization.
Use these singing priorities to choose approximations, keeping source text beside
every sampled token:

* Preserve the sustained vowel nucleus and its intended register/tone first;
  compare vowel height, front/back placement and rounding using documented or
  heard sounds, never sample-label spelling alone. Among comparable vowels,
  prefer the closest consonant onset/coda and attack, preserving lexical stress.
* Keep one source syllable through its ties/melisma. Use HLS for continuation;
  do not replay consonants on every pitch or insert a sample's extra vowel into
  a monophthong. For a diphthong retain its principal vowel before its offglide.
* Preserve authored note timing and stress. Favor a sample whose consonant and
  vowel onset fit the gesture; this does not authorize moving notes or guessing
  an attack offset. Sustain and detached articulations must fit the sung gesture.
* If phonetics are unverified, label the mapping provisional. List omitted or
  changed vowels, consonants and stress; do not claim intelligible source text.
  Never worsen an established mapping to reduce receivers.

These are project interpretation priorities informed by [choral diction practice](https://www.galachoruses.org/resource/diction),
not a universal metric of intelligibility. [Friedrichs et al., ICPhS 2015, §§2–4](https://www.internationalphoneticassociation.org/icphs-proceedings/ICPhS2015/Papers/ICPHS0434.pdf)
found vowel-specific pitch effects in one controlled speaker/listener experiment;
register alone does not justify replacing every vowel with Ah. Exact sampled
phonetics and the resulting phrase still require their own evidence.

Read complete parts before choosing motifs. Mark complete sung gestures by text,
stress, phrasing, breaths/rests and articulation. Mandatory gesture/restart and
receiver-family boundaries cannot be crossed to compress the dictionary. Barlines
alone are not boundaries. Admit internal cuts only at independently attacked new
syllables or qualified receiver handoffs; document each reason. Never cut a tie,
melisma, indivisible elision or required continuous transition. Prefer whole words
and stress groups; justify any one-syllable gesture or necessary split.

Use fixed-vowel legato for intended wordless singing or a complete suitable vowel
segment/melisma when its recorded vowel and tone fit. Disclose omitted consonants.
Use Mm for intended/accepted humming; composite patches only for the intended
documented vowel transition. Slow tempo alone does not select Slow Legato. Prefer
builder HLS where it preserves the syllable adequately. A mid-syllable transfer to
a vowel receiver requires separate audible and state-continuity qualification.

For an entry at arranger beat zero, qualify simultaneous selector/first-note
ordering before relying on it: no preceding selector pulse fits inside that
placement. If the selector arrives after the first attack, admit a one-cell
configuration for the opening syllable and a receiver handoff at the next
independently attacked authored syllable. Qualify that entry, preserve its text
and musical timing, and solve again with the resulting mandatory boundary and
safe cuts. Mark that gesture `"single_cell_receiver": true`; a boundary alone
does not prevent prefix reuse of a longer receiver. Do not shift a sung note or
assume a saved cursor to hide the failure.

Prepare a reviewed planning JSON, separate from executable MIDI/companion:

```json
{"capacity":24,"available_channels":[2,3,4],"gestures":[
  {"id":"g1","atoms":[{"sample":"TOKEN","articulation":"sustain"}],
   "cuts":[0,1],"cut_reasons":{},"settings":{"nki":"library/patch.nki"}}
]}
```

Replace illustrative numbers/labels with evidence. Each atom represents one sung
syllable; include all configuration-affecting cell settings in it. Gesture settings
include the exact NKI and required engine settings. Cuts are boundary indexes;
zero and atom count are required. List each permitted internal cut's musical
reason. Keep fixed-vowel bindings and other occupied channels out of the available
list. Separate gestures/lanes with incompatible settings or simultaneous cursor
histories; do not share a receiver between contrapuntal parts.

Enumerate all contiguous candidates between admitted cuts, within verified
capacity. Canonicalize exact atom sequences plus settings. Pitch, onset, tempo and
external dynamics need not distinguish configurations when controls remain valid.
Connected rhythmic phrases or incompatible cell settings do distinguish them.
Each occurrence starts at word one and consumes an exact prefix of its configured
atoms. A shorter prefix needs no separate receiver: stop its sung notes at the
authored boundary and reset before the next use; do not trigger the unused tail.
Matching requires every consumed cell setting and engine setting to agree.
Common openings followed by divergent cells are different configurations; an
interior suffix cannot be reused by assuming random access. A one-cell entry
constraint remains binding, so resegment longer gestures at admitted cuts rather
than keeping a redundant longer phrase starting with that one-cell phrase.
Use the exact minimum-label path solver:

```sh
uv run "$skill_dir/scripts/phrase_plan.py" solve \
  --input "$evidence/planning.json" --output "$evidence/dictionary.json" --seconds 60
```

The solver minimizes distinct configurations, then occurrences, then programmed
cells. It enumerates every eligible interval and shares configuration activation
across gestures. It reports status, objective values and primary bound. Only
OPTIMAL establishes the minimum over the reviewed candidate set. FEASIBLE is an
incumbent, not a minimum. No solution returns to musical/capacity review; never
relax diction or safe cuts automatically. Resolve concurrent cursor demands into
independent lanes before solving; the script does not infer concurrency.

Review optimization **before programming or accepting** the bank. The solver
rejects selected strict-prefix pairs within compatible settings and reconstructs
the consumed prefixes. Review its result against the source and control schedule:
no uncovered/duplicated syllables, no invented cuts, no unused-tail attacks, and a
verified reset from a noninitial cursor after a shortened use. OPTIMAL plus this
review establishes irreducibility only within the admitted candidates and controls.

Record in PERFORMANCE.md: original-text mapping/losses; gestures and cut reasons;
canonical phrase definitions; source/performed occurrence-to-phrase/channel table;
fixed-vowel choices; reset/HLS schedule; candidate scope and solver status/bounds.
Independently reconstruct each gesture from consumed phrase prefixes: exact
ordered atom equality, every source syllable and native musical note covered once,
and every occurrence starting at configuration index zero. Audit standalone suffix
entries explicitly; an optimizer status alone does not establish correct grouping.
One configured phrase owns one named slot using the standard Filter/Map chain.
The solver emits one performance channel per phrase; extra admitted channels
share that same cursor and do not create independent phrase histories. Reuse its receiver at every
compatible occurrence; every entry begins at the first word. Retain musical
timing, pitches, articulation and stress. Route setup to kontakt-setup; continue
here when its integrated assignment returns the loaded editor with
`--keep-editor-open`. Name each Instrument Layer slot from the **source MIDI/score
lyric syllables** represented by a complete configured sequence, in their original
order and with syllable boundaries visible: e.g. `Do mi nus De us`, never the
mapped sample labels `No Ki Luh Leh Luh`. Preserve distinct source-text aliases
when several texts map to one bank; retain the full bank's source name for shorter
prefix occurrences and list their consumed text in PERFORMANCE.md. Add only a
necessary settings qualifier. Opaque IDs and sampled labels belong in evidence,
not slot names. Reuse and rename the existing matching slot when reconciling a bank.
