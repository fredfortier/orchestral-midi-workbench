# Load once; share an ephemeral context

Open for every analysis. Input is the acquired MXL/XML and its known provenance;
output is `score`, indexed `parts`, `start/end` in quarter-note beats, and `rows`.
Return to the intent router. No permanent script is needed for ordinary analysis.

From `crates/`, concatenate the setup and selected reference blocks into one stdin
run. Alternatively start `python -i` through the same uv environment and execute
blocks in that process. Separate uv invocations do not share Python variables.

```sh
uv run --no-project --with 'music21==10.5.2' python - <<'PY'
# Paste setup below, then only the selected analysis blocks.
PY
```

The version is the tested dependency, not an instruction to upgrade on every
invocation. If the user asks for latest, verify PyPI and report the resolved
version; check changed APIs before substituting it. Do not install a project venv,
launch notation software, or call `.show()` for a numerical question.

```python
import copy
import math
from collections import Counter
from pathlib import Path
import music21 as m21

score_path = Path("/absolute/path/to/current/score.mxl")
score = m21.converter.parse(score_path, forceSource=True)
parts = list(score.parts)
if not parts:
    raise ValueError("Expected one score with parts; resolve an Opus/part explicitly")
start, end = 0.0, float(score.highestTime)  # half-open analysis interval
if end <= start:
    raise ValueError("No positive score extent")

def rows(part, merge_ties=False):
    source = part.stripTies() if merge_ties else part
    flat = source.flatten()
    for obj in flat.notes:
        atoms = obj.notes if isinstance(obj, m21.chord.Chord) else [obj]
        for atom in atoms:
            strength = float(obj.beatStrength)
            yield dict(on=float(obj.offset), end=float(obj.offset + obj.quarterLength),
                       pitch=atom.pitch, ps=float(atom.pitch.ps),
                       tie=atom.tie.type if atom.tie else None,
                       measure=obj.measureNumber, beat=obj.beatStr,
                       strength=strength if math.isfinite(strength) else None)

def chosen_score(indices):
    selected = m21.stream.Score()
    for i in indices:
        selected.insert(0, copy.deepcopy(parts[i]))
    return selected

print("music21", m21.__version__, "file", score_path, "quarter beats", end)
for i, part in enumerate(parts):
    events = list(rows(part))
    print(i, part.partName, "measures", len(part.getElementsByClass(m21.stream.Measure)),
          "pitched segments", len(events),
          "pitch attacks", sum(r["tie"] not in ("stop", "continue") for r in events))
```

Choose indices from this inventory, not assumptions about staff order or unique
names. For an excerpt set `start/end` from actual measure offsets; do not assume
constant meter or that the first measure is numbered 1. Analysis snippets preserve
global offsets. Scope key/texture analysis with explicit part indices when a
reduction, doubled bass or editorial keyboard would bias it.

Offsets are quarter-note units, not seconds or necessarily metrical beats. Use
raw segments for duration weighting; they partition tied durations. Attacks exclude
tie continuations. `stripTies()` helps melodic adjacency but partially tied chords
may retain ties: inspect residuals instead of treating every chord as one attack.
Keep the imported score unmodified; transformations belong to disposable copies.

Source mechanisms: User Guide [6: hierarchy/flattening](https://music21.org/music21docs/usersGuide/usersGuide_06_stream2.html)
and [31: ties](https://music21.org/music21docs/usersGuide/usersGuide_31_clefs.html).
