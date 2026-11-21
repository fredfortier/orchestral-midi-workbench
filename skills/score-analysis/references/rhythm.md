# Pulse, rhythmic activity and metric placement

Use for static/busy passages, note-value variety, attack density or possible
syncopation. Input is the loaded score and an explicit passage. Return per-part
activity and a few measure/beat examples, not a single complexity score.

```python
for i, part in enumerate(parts):
    data = [r for r in rows(part, merge_ties=True)
            if start <= r["on"] < end and r["tie"] not in ("stop", "continue")]
    attack_moments = sorted({r["on"] for r in data})
    durations = Counter(round(r["end"]-r["on"], 8) for r in data)
    strengths = Counter(r["strength"] for r in data)
    print(i, part.partName, "pitch attacks", len(data), "attack moments", len(attack_moments),
          "moments per quarter beat", len(attack_moments)/(end-start),
          "common durations (quarter units)", durations.most_common(5),
          "metric strengths (pitch attacks)", dict(strengths))
```

A chord contributes several pitch attacks but one attack moment. The duration
histogram counts pitched attacks, so doubled chord members weight it repeatedly;
state that or isolate the line. Ties are merged for durations, and residual
partially tied chord continuations are excluded. If those partial ties affect a
conclusion, inspect that voice rather than claim exact duration statistics.

For “why does this feel syncopated?”, inspect actual notes in the requested bar:

```python
part_index = 0  # set the intended part
measure_number = parts[part_index].getElementsByClass(m21.stream.Measure)[0].number
measure = parts[part_index].measure(measure_number)
for n in measure.recurse().notesAndRests:
    print(n.measureNumber, n.beatStr, n.beatStrength, n.quarterLength, n)
```

`beatStrength` is music21's metric hierarchy, not performed accent. A weak-beat
attack alone is not syncopation: check duration across a stronger beat, ties,
rests, articulation and neighboring parts. Compound-meter beats are not necessarily
quarter notes. No inferred meter should replace the imported meter silently.
For changing activity, run the same bounded comparison on a few named sections;
do not enumerate every subdivision or fit a rhythm model by default.

Sources: [14: beats and beatStrength](https://music21.org/music21docs/usersGuide/usersGuide_14_timeSignatures.html),
[31: ties](https://music21.org/music21docs/usersGuide/usersGuide_31_clefs.html).
