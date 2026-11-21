# Select a documented receiver

Resolve INSTRUMENTS_ROOT_DIR from project instructions, configuration or the user.
This setting is not a converter option.

```text
musical requirement -> targeted local library discovery
                            |
                  relevant INSTRUMENT.md usable?
                    yes /                  \ no
                     v                      v
               compare variants       instrument-intake
                     ^                      |
                     +----------------------+
                     |
             exact stock NKI + native routing -> PERFORMANCE.md -> rendition inputs
```

Search the local root with rg/find by library/instrument family. Read applicable
local instructions. Avoid dumping the entire NKI collection into context. Compare
instrument/ensemble, technique, register, attack and release, legato transitions,
dynamic law and independent control needs. Filename matches are discovery leads,
not evidence of musical suitability. Read actual documented exceptions.

Invoke [instrument-intake](../../instrument-intake/SKILL.md) for the relevant
library/subset when documentation is missing, stale or insufficient. Return to
selection afterward. Do not initiate every library or replace that skill with
another manual-research procedure here. Missing manual/capability evidence remains
an explicit selection uncertainty; do not invent support.

Record the exact NKI relative path beneath INSTRUMENTS_ROOT_DIR as the sole patch
identity, applicable documentation,
required player version, documented stock behavior, native routing and musical
reason per binding in PERFORMANCE.md. Library patch keys may be catalogue labels;
they do not replace the relative path. Use conventional labels
such as `Legato`, `Pizzicato`, `Sustains — soft attack`, qualified within the musical
track. Dynamics/control modes are specified precisely in the document, not hidden
in a cryptic channel name. Independent parts reuse patch definitions but retain
independent receiver instances. Multi-articulation patches need a documented,
supported switching recipe; one NKI is not automatically one technique.

Do not treat expressive sample-library conventions as MIDI-CI support. Do not
choose sampled runs, patterns or word builders as substitutes for written notes
without an explicit musical decision and executable supported control mechanism.
Inability to load the chosen patch is a setup limitation, not permission to choose
another instrument silently.

The setup boundary does not customize receivers. Select documented stock variants
whose normal behavior fits the intended notes. A control law or manual capability
does not authorize MIDI CC sender devices. Follow the shared workflow's
note-channel slot selection, channel-1 receiver remap and shared track MIDI controls.
Performance Controls, other helper devices, macros, private settings, edited NKIs
and alternate loading workflows remain forbidden. Record an unsupported
requirement rather than silently introducing these mechanisms.
