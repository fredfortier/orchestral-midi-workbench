# Tessitura, orchestration and textural build

Use for range, exposed register, density, doubling or who carries a passage.
After loading, select relevant part indices and `start/end`. Return a part table
and the locations of a few textural changes; add one plot only when useful.

```python
indices = list(range(len(parts)))  # narrow to the requested family when appropriate

def union_duration(intervals):
    total, cursor = 0.0, start
    for a, b in sorted(intervals):
        if b > cursor:
            total += b - max(a, cursor)
            cursor = b
    return total

def weighted_pitch(data, fraction):
    ordered = sorted((r["ps"], min(end, r["end"]) - max(start, r["on"])) for r in data)
    target = sum(w for _, w in ordered) * fraction
    running = 0.0
    for pitch, weight in ordered:
        running += weight
        if running >= target:
            return pitch
    return None

for i in indices:
    data = [r for r in rows(parts[i]) if r["on"] < end and r["end"] > start]
    if not data:
        print(i, parts[i].partName, "silent in interval")
        continue
    intervals = [(max(start, r["on"]), min(end, r["end"])) for r in data]
    print(i, parts[i].partName, "range", (min(r["ps"] for r in data), max(r["ps"] for r in data)),
          "duration-weighted central 80%", (weighted_pitch(data, .1), weighted_pitch(data, .9)),
          "active fraction", union_duration(intervals) / (end - start))

texture_score = chosen_score(indices)
tree = texture_score.asTimespans(flatten=True, classList=(m21.note.Note, m21.chord.Chord))
density_beats = Counter()
for v in tree.iterateVerticalities():
    a = max(start, float(v.offset))
    b = min(end, float(v.offset + v.timeToNextEvent)) if v.timeToNextEvent is not None else end
    sounding = {id(ts.part) for ts in v.startTimespans + v.overlapTimespans if ts.part is not None}
    if sounding and b > a:
        density_beats[len(sounding)] += b - a
print("concurrent sounding parts -> quarter beats", dict(sorted(density_beats.items())))
```

Central 80% is an operational tessitura summary, not a player's comfort range.
Weight raw tied segments by duration; union intervals for occupancy so chords do
not make a part “more than 100% active.” Concurrent parts differ from concurrent
pitches. Silence outside the first/last event is not included in the density
histogram above; report that denominator when converting it to percentages.

For a requested visual, reuse these rows rather than another extraction:

```python
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
fig, axes = plt.subplots(len(indices), 1, sharex=True, squeeze=False,
                         figsize=(11, max(3, 1.5 * len(indices))))
for ax, i in zip(axes[:, 0], indices):
    for r in rows(parts[i]):
        a, b = max(start, r["on"]), min(end, r["end"])
        if b > a:
            ax.broken_barh([(a, b-a)], (r["ps"]-.35, .7))
    ax.set_ylabel(parts[i].partName or str(i))
axes[-1, 0].set_xlabel("Quarter-note beats from score start")
fig.tight_layout()
fig.savefig("register-texture.png", dpi=160)
plt.close(fig)
```

Review overlap at a named passage for doubling, unison/octave reinforcement or
separation. Density and register are not acoustic masking/loudness measurements.
A tutti need not be the musical climax. Do not silently delete editorial continuo
or doubled bass: state the part selection and compare with/without when relevant.

Mechanisms: [22: piano rolls and pitch distributions](https://music21.org/music21docs/usersGuide/usersGuide_22_graphing.html),
[61: timespans and verticalities](https://music21.org/music21docs/usersGuide/usersGuide_61_trees.html).
