# Tonal plan, Roman numerals and cadential direction

Use for key, modulation, applied dominants, cadences or harmonic rhythm. Input is
the loaded score, explicit harmonic part selection and a named passage. Return
key alternatives plus a short checked progression with bass/pitches and locations.
Do not print a Roman label for every passing event and call it a harmonic analysis.

```python
harmonic_indices = list(range(len(parts)))  # explicitly exclude reductions/doublings if justified
harmonic = chosen_score(harmonic_indices)
key_material = harmonic  # for a local key question use harmonic.measures(first, last)
estimate = key_material.analyze("key")
print("estimated key", estimate, "correlation", estimate.correlationCoefficient)
print("alternatives", [(str(k), k.correlationCoefficient)
                       for k in estimate.alternateInterpretations[:3]])
context = estimate  # replace with m21.key.Key("b") only for an established B-minor reading
reduction = harmonic.chordify()
chords = [c for c in reduction.recurse().getElementsByClass(m21.chord.Chord)
          if start <= float(c.getOffsetInHierarchy(reduction)) < end]
structural_samples = [c for c in chords if c.beatStrength >= .5]
for c in structural_samples[:12]:
    label = m21.roman.romanNumeralFromChord(c, context)
    applied = m21.roman.romanNumeralFromChord(c, context, preferSecondaryDominants=True)
    print(c.measureNumber, c.beatStr, "bass", c.bass().nameWithOctave,
          "pitches", [p.nameWithOctave for p in c.pitches],
          "Roman", label.figure, "secondary candidate", applied.figure)
```

The .5 metric threshold and twelve-row cap keep a first pass small; they are not
universal harmonic-boundary rules. For a named offset, inspect the chord *sounding*
there, including a chord that started earlier:

```python
probe = start  # set to the requested global quarter-note offset
active = next((c for c in reduction.recurse().getElementsByClass(m21.chord.Chord)
               if float(c.getOffsetInHierarchy(reduction)) <= probe <
               float(c.getOffsetInHierarchy(reduction) + c.quarterLength)), None)
print("at", probe, active)
```

Choose global versus local key context explicitly. A correlation is not a
probability, and a new window winner alone does not establish modulation. Confirm
cadential bass motion, leading tones and resolution across a few measures.
Review claimed V7/x against the actual arrival; preference for secondary dominants
is a hypothesis switch, not evidence. For harmonic rhythm, consolidate passing
sonorities and held suspensions in this short progression before counting changes.

Missing source spelling can produce misleading roots/inversions. If a label looks
implausible, compare a disposable `c.simplifyEnharmonics(keyContext=context)` with
the original pitches/label; report a changed interpretation, not recovered spelling.
Do not force every dissonant verticality into a triad. Do not use `functionalityScore`
as a composition-quality score. Analyze with/without editorial accompaniment when
its realization or doubling materially weights the key/chord result.

Source: [9: chordify and passing sonorities](https://music21.org/music21docs/usersGuide/usersGuide_09_chordify.html),
[15: key estimates/context](https://music21.org/music21docs/usersGuide/usersGuide_15_key.html),
[23: Roman numerals](https://music21.org/music21docs/usersGuide/usersGuide_23_romanNumerals.html).
