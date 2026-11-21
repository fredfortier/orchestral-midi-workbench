# Melodic contour and interval economy

Use for an awkward leap, line shape, repeated notes or a registral peak. Input is
one identified monophonic part/voice plus the loaded context. Return its interval
summary and a few located examples. Do not infer a melody by taking the top note
of every orchestral chord.

```python
line_index = 0  # choose from the inventory, not an assumed soprano position
line = parts[line_index].stripTies()
line_flat = line.flatten()  # retain the site that supplies score-relative offsets
notes = sorted(line_flat.notes, key=lambda n: float(n.offset))
if any(not isinstance(n, m21.note.Note) for n in notes):
    raise ValueError("Select an explicit monophonic line; this part contains chords")
if any(float(a.offset + a.quarterLength) > float(b.offset) + 1e-8
       for a, b in zip(notes, notes[1:])):
    raise ValueError("Overlapping notes: resolve intended voice/legato before melodic adjacency")
notes = [n for n in notes if start <= float(n.offset) < end]
motions = []
for a, b in zip(notes, notes[1:]):
    iv = m21.interval.Interval(a, b)
    motions.append(dict(measure=b.measureNumber, beat=b.beatStr,
                        offset=float(b.offset), interval=iv.directedName,
                        semitones=float(iv.semitones), step=iv.isStep,
                        gap=float(b.offset - a.offset - a.quarterLength)))
print("line", parts[line_index].partName, "interval pairs", len(motions))
print("repeats", sum(r["semitones"] == 0 for r in motions),
      "diatonic steps", sum(r["step"] for r in motions))
print("largest leaps", sorted(motions, key=lambda r: abs(r["semitones"]), reverse=True)[:5])
if notes:
    peak = max(float(n.pitch.ps) for n in notes)
    print("peaks", [(n.measureNumber, n.beatStr, n.pitch.nameWithOctave)
                    for n in notes if float(n.pitch.ps) == peak][:5])
```

Interval-pair denominators are not note counts. A repeated pitch and a diatonic
step are separate categories; chromatic spelling can affect diatonic classification.
The gap field identifies adjacency across silence: do not describe that as a
continuous phrase automatically. Overlap can be intentional performance legato,
not a counterpoint error. If the part cannot supply an unambiguous voice, return
its range/contour limits and the unresolved selection rather than fabricate one.

Review direction changes, leap recovery and repeated peaks at musical phrase
boundaries. “Singable/playable” additionally requires voice/instrument, tempo,
articulation and breathing/bowing context; a leap count cannot certify it.

Source: [18: intervals, direction, step and compound/simple names](https://music21.org/music21docs/usersGuide/usersGuide_18_intervals.html).
