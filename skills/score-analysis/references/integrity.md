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

For a complete export audit, use the exact editorial assembly input with the
reconstructed score. Count native sounding occurrences per part and pitch,
applying their clip windows, loops, mute state and declared reset commands.
In the score, expand chord pitches and join contiguous sounding ties within
each voice. Compare the ordered onset and release pairs for each part and
pitch within the documented quantization and notation tolerance. When a
part has multiple voices, check the stored pitch-level tie starts and stops
if whole-part `stripTies()` leaves continuations behind. Keep simultaneous
same-pitch attacks in distinct voices. Use explicit aliases for a comparison
with a source edition and resolve duplicate names before matching parts.

Compare pitches and attacks first, then endpoints within the declared export
quantization/notation tolerance. Distinguish wrong conversion, current DAW edits,
source-edition differences and deliberate performance extensions. Check placement
ends when a stored tail/fermata exceeds the exported extent. Report the discrepancy;
this read-only skill does not extend clips or delete tiny notes.

Source: [31: stripping and making ties](https://music21.org/music21docs/usersGuide/usersGuide_31_clefs.html),
[14: meter context](https://music21.org/music21docs/usersGuide/usersGuide_14_timeSignatures.html).
