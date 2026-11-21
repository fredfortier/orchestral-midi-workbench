# Is the score faithful enough to analyze?

Use after loading for a suspicious export, count, duration or source comparison.
Return a short list of located discrepancies and what remains safe to conclude.
Do not repair intentional performance timing into textbook notation by assumption.

```python
for i, part in enumerate(parts):
    data = list(rows(part))
    unusual = [r for r in data if r["end"] <= r["on"] or
               (0 < r["end"] - r["on"] < 1/128)]
    print(i, "first onset", min((r["on"] for r in data), default=None),
          "last release", max((r["end"] for r in data), default=None),
          "tiny/zero segments", len(unusual))
    for r in unusual[:5]:
        print(" candidate", r["measure"], r["beat"], r["pitch"].nameWithOctave,
              r["on"], r["end"] - r["on"])
print("encoded keys", list(score.recurse().getElementsByClass(m21.key.KeySignature)))
print("encoded meters", [str(t) for t in score.recurse().getElementsByClass(m21.meter.TimeSignature)])
```

The 1/128-quarter threshold is a diagnostic trigger, not a minimum legal note.
Grace notes can have zero duration. Pickups, incomplete bars and multiple voices
also defeat blanket “bar duration must equal meter” assertions. Locate the event,
then compare native capture and notation decisions before attributing a defect.
A missing key signature is missing metadata, not proof of C major.

For a requested complete export audit, reuse the existing example rather than
inventing a comparator. This is the durable-script exception to ephemeral analysis:

```sh
uv run --no-project ../../examples/music21_analysis.py /absolute/path/score.mxl --capture /absolute/path/interpreted.json --output /absolute/path/analysis.json
```

Run that command from this skill directory (`crates/skills/score-analysis`);
from `crates/`, use `examples/music21_analysis.py`. `--capture` must be the exact
editorial assembly input, not an arbitrary later capture. Add `--reference SOURCE`
and repeat `--reference-part 'Exported name=Source name'` for explicit aliases.
Duplicate source names require resolution; a realized continuo is not automatically
identical to a source piano reduction. Read the script's exit status and audit
fields; its default extra harmonic report is not a complete functional analysis.

Compare pitches and attacks first, then endpoints within the declared export
quantization/notation tolerance. Distinguish wrong conversion, current DAW edits,
source-edition differences and deliberate performance extensions. Check placement
ends when a stored tail/fermata exceeds the exported extent. Report the discrepancy;
this read-only skill does not extend clips or delete tiny notes.

Source: [31: stripping and making ties](https://music21.org/music21docs/usersGuide/usersGuide_31_clefs.html),
[14: meter context](https://music21.org/music21docs/usersGuide/usersGuide_14_timeSignatures.html).
