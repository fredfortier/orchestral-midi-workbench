# Dissonance, suspensions and independent voices

Use for a specific tension/resolution, crossing or parallel-motion question.
Input is the loaded score, passage and identified voices. Return before/during/
after pitches with locations and candidate explanations. Whole-score consonance
rates are optional descriptive measures, never rules for how music should sound.

```python
counterpoint_indices = list(range(len(parts)))  # narrow to the relevant voices
cp_score = chosen_score(counterpoint_indices)
cp_tree = cp_score.asTimespans(flatten=True, classList=(m21.note.Note, m21.chord.Chord))
classification_beats = Counter()
for v in cp_tree.iterateVerticalities():
    a = max(start, float(v.offset))
    b = min(end, float(v.offset + v.timeToNextEvent)) if v.timeToNextEvent is not None else end
    if b > a and v.pitchSet:
        classification_beats["consonant" if v.toChord().isConsonant() else "other"] += b-a
print("quarter beats by conventional chord consonance", dict(classification_beats))
probe = start  # set the offset at the questioned sonority
v = cp_tree.getVerticalityAt(probe)
print("at", probe, "pitches", sorted(p.nameWithOctave for p in v.pitchSet))
for ts in v.startTimespans + v.overlapTimespans:
    print(ts.part.partName, ts.offset, ts.endTime, ts.element)
```

Use `timeToNextEvent`, not `toChord().quarterLength` (which is a default duration).
Silence is excluded; explain that denominator. Consonance classification depends
on spelling and a conventional tonal model. A dissonant seventh/suspension can be
exactly the intended effect. Preparation and stepwise resolution require tracing
the *same voice* on both sides, not just neighboring chord roots.

For requested fifth/octave candidates between two monophonic parts:

```python
upper_index, lower_index = 0, 1  # replace with intended upper/lower voices
u_data, l_data = list(rows(parts[upper_index])), list(rows(parts[lower_index]))

def one_at(data, t):
    active = [r for r in data if r["on"] <= t < r["end"]]
    if len(active) > 1:
        raise ValueError("Polyphonic/overlapping part: select an explicit voice")
    return active[0] if active else None

points = sorted({r[k] for r in u_data+l_data for k in ("on", "end") if start <= r[k] < end})
previous = None
flags = []
for t in points:
    u, l = one_at(u_data, t), one_at(l_data, t)
    if u is None or l is None:
        previous = None  # do not bridge a silence
        continue
    if u["ps"] < l["ps"]:
        flags.append((t, "crossing", u["pitch"].nameWithOctave, l["pitch"].nameWithOctave))
    if previous:
        _, old_u, old_l = previous
        quartet = m21.voiceLeading.VoiceLeadingQuartet(
            m21.note.Note(old_u["pitch"]), m21.note.Note(u["pitch"]),
            m21.note.Note(old_l["pitch"]), m21.note.Note(l["pitch"]))
        if quartet.parallelFifth() or quartet.parallelOctave():
            flags.append((t, "parallel candidate", u["measure"], u["beat"]))
    previous = (t, u, l)
print("candidate count", len(flags), "first examples", flags[:10])
```

The example indices must be replaced with the intended voices from the inventory.
A keyboard or legato-overlapped part may fail the monophony check. Deliberate orchestral doubling is not a counterpoint
mistake. Compare onsets, ties and rest boundaries; do not issue a “parallel errors”
total without identifying the two independent voices and the intended style.
If voice identity is unresolved, keep the verticality observation and stop the
parallel branch. No live note changes follow from a flag.

Mechanisms: [61: sustained/start/stop timespans and duration weighting](https://music21.org/music21docs/usersGuide/usersGuide_61_trees.html),
[18: interval identity and consonance](https://music21.org/music21docs/usersGuide/usersGuide_18_intervals.html).
`VoiceLeadingQuartet` is an additional API, not a ready-made orchestration rule.
