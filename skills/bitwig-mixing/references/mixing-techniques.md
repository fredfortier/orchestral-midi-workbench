# Mixing techniques and quantitative method

## Musical objective

Mixing balances level, frequency content, dynamics and spatial placement over
time. Establish the intended foreground and supporting roles before processing;
evaluate sounds in the ensemble, rather than optimizing every soloed track.[^mix]
For contrapuntal/orchestral material, treat the subject, answering entries,
inner-voice motion, harmonic bass and phrase destinations as candidate priorities.
This is the workflow's musical interpretation policy: establish which of those
roles actually applies from the score, current performance plan and listening.

Shared register and harmonics can create a desirable ensemble blend. Masking is
problematic when competing sound prevents the intended part from being followed;
it is not synonymous with any spectral overlap. Choose the foreground to protect
before reducing competing energy.[^mask]

Preserve meaningful doubling. Orchestration, register and articulation influence
clarity, so an apparent mix problem may originate in source selection or musical
realization.[^orchestra] Diagnose that cause and obtain the relevant authority
before changing it. Do not use mixing as an implicit reorchestration step.

## Balance and fader decisions

Start with a rough balance and locate exceptional peaks before detailed
processing. Keep a consistent monitoring level and compare with suitable
references; inspect the whole progression rather than a single loud passage.[^balance]

Use this diagnostic decision table. The proposed sequence is local editorial
policy built from balance and separation principles, not a universal recipe.[^separation]

| Audible symptom | Question / first experiment |
| --- | --- |
| Important part weak throughout | Is its source healthy? Test its level against reduced accompaniment |
| It disappears only with a competing part | Test masking in that passage; identify simultaneous competing bands |
| Attack absent, sustain already prominent | Inspect sampled attack/articulation, onset and envelope before raising all of it |
| Clear but too distant | Inspect existing room contribution and direct/reverberant relationship |
| A few notes protrude | Try local automation or envelope control instead of raising/compressing the whole part |
| Successive boosts consume headroom | Re-establish relative priorities and lower supporting material |

This workflow recommends small audition increments, typically 0.5–1 dB, as an
explicit editorial starting point. They are not minimum audible differences,
calibration anchors, required instrument gaps or industry acceptance thresholds.
Stop when the intended relation works; do not force equal faders or equal meters.

For sampled instruments, distinguish amplitude trim from performance controls
that may change timbre, attack or transitions. Verify the exact patch's manual
and observed control response before changing velocity or CC dynamics. The
[expression-control contract](../../WORKFLOW.md#expression-control-specification)
owns those project decisions and preservation duties.

## Separation, space and processing

Use level, arrangement/timing awareness, spectral balance and depth together.
Panning alone does not solve all masking, and a wider stereo image can lose its
apparent separation in mono. EQ can prioritize one source's useful energy over
another; dynamic processing can act only while the conflict occurs.[^separation]

Reverberation supplies space and depth but adds a continuing signal after the
direct attack. Inspect existing sample ambience before adding another room, and
judge decay against subsequent musical events.[^space] Workflow policy: preserve
an established orchestral perspective unless changing it has a stated purpose;
do not impose a fixed seating chart, high-pass frequency or reverb time.

Compression is for a demonstrated envelope/dynamic problem. Compare processed
and bypassed sound at matched loudness, retaining intended attacks and phrase
contrast.[^dynamics] No compressor, limiter, EQ cut or stereo widening is a
mandatory step merely because this is a mixing request.

## Data and measurement

Native settings and rendered sound answer different questions. Bitwig's mixer
exposes track controls; its Spectrum visualizes frequency content and Tool
provides amplitude/pan/width controls and meters.[^bitwig] The native observation
contract reports stopped stored values, not a performance-wide effective value.
Use complete existing automation reads for time-varying intent and audio for
actual output. Never infer audible support from MIDI or parameter values alone.

Retain synchronized, unnormalized renders, exact range/sample rate, source taps,
returns/tails, native balance/processing identity and a phrase-priority map.
This is the workflow's evidence requirement. Separate observed facts, computed
diagnostics, source-backed mechanics and subjective judgments in the report.

Programme loudness and true-peak measurement have standardized algorithms;
use a meter implementing the declared ITU revision when claiming those
measurements.[^meter] Peak/RMS, loudness, spectral balance, transient shape and
stereo/mono behavior are complementary diagnostics rather than interchangeable
scores.[^meter-guide] Specify channel aggregation and time windows. Analyze
representative simultaneous passages; whole-piece averages can conceal local
competition (an inference from masking's time/frequency dependence).[^mask]

For a suspected collision, compare the intended part with its actual competing
accompaniment over the same time interval and frequency band. A band-level
difference is a diagnostic defined here as target band level minus accompaniment
band level, using identical filters/windows/channel treatment. It is not a
psychoacoustic masking threshold. A spectrum overlay identifies shared energy;
audition decides whether the intended distinction is obscured.[^mask]

### Quantitative operating method

For an audio-backed mix, retain waveform or envelope views for timing, average
spectra for shared frequency regions, and a spectrogram when coexistence changes
within the passage. A spectrogram shows frequency content over time; an averaged
spectrum alone loses that temporal distinction.[^spectrogram]

Record sample rate, analysis window, overlap, scaling and channel treatment.
Welch analysis estimates power spectral density by averaging windowed segment
periodograms. With density scaling, integrate the same frequency bins for each
source to compare band power; do not compare plots with independently normalized
vertical scales.[^psd] This workflow's convention for stereo comparisons is mean
left/right power, with full-scale amplitude one; band level is
`10 log10(sum(PSD[k] * bin_width))`. State other conventions explicitly.

Measure target/comparison band-level differences before and after intervention,
and inspect the waveforms/envelopes for attack and timing changes. These are
local diagnostic definitions, not an auditory masking model. A level change can
improve a useful band's relative prominence while leaving spectral shape overlap
unchanged. Do not claim that overlap was eliminated or that audibility was proved
from those numbers. Keep listening approval separate.[^mask]

Use native beat-to-seconds context for musical windows. Do not divide total file
duration by the exported beat count: a tail can cross a later tempo change.
The [native observation/export contract](../../../controller/MIXING.md) owns that
context and the independent output-duration check.

Do not invent an industry-wide allowed overlap percentage, universal critical
frequency or mandatory target-to-masker dB gap. These sources describe contextual
masking and intervention, not such a pass/fail rule.[^mask] Any numerical alert
chosen by an implementation is a labelled heuristic requiring listening
validation, not conformance evidence.

Delivery targets apply only when the destination is established. For example,
EBU R128's programme target is −23 LUFS and its maximum production true peak is
−1 dBTP; these are broadcast requirements, not individual-track balance
targets.[^delivery] Do not force a music mix to a platform loudness number in the
absence of an applicable delivery requirement.

[^mix]: The four elements and their interaction: iZotope, *Mixing Guide:
    Principles, Tips and Techniques*, 2014 edition, chapter 3,
    [official guide](https://downloads.izotope.com/guides/iZotope-Mixing-Guide-Principles-Tips-Techniques.pdf).
[^mask]: Useful blending, contextual obstruction and foreground priority:
    iZotope, [Unmasking Your Mix](https://www.izotope.com/community/blog/unmasking-your-mix-with-neutron),
    sections 1–2. Product-specific automatic processing is not required here.
[^orchestra]: Arrangement and timbre as balance/separation mechanisms:
    Sound On Sound, [Creating Orchestral Music With Sampled Instruments](https://www.soundonsound.com/techniques/creating-orchestral-music-sampled-instruments).
    This older practice article does not establish current sampler capabilities.
[^balance]: Rough balance, reference listening and peaks:
    Sound On Sound, [Mixing Essentials](https://www.soundonsound.com/techniques/mixing-essentials).
[^separation]: Context, prioritization, EQ, dynamics and limits of panning:
    Sound On Sound, [How To Achieve Better Separation In Your Mixes](https://www.soundonsound.com/techniques/how-achieve-better-separation-your-mixes).
[^space]: Spatial placement and time-based effects: iZotope guide, chapter 3
    and chapter 7, Time-Based Effects, [official guide](https://downloads.izotope.com/guides/iZotope-Mixing-Guide-Principles-Tips-Techniques.pdf).
[^dynamics]: Dynamics processing and level-matched evaluation: iZotope guide,
    chapter 5, Dynamics,
    [official guide](https://downloads.izotope.com/guides/iZotope-Mixing-Guide-Principles-Tips-Techniques.pdf).
[^bitwig]: Native controls and analysis descriptions: Bitwig
    [Mix View](https://www.bitwig.com/userguide/latest/the_mix_view/),
    [Spectrum](https://www.bitwig.com/userguide/latest/device_descriptions/#spectrum)
    and [Tool](https://www.bitwig.com/userguide/latest/utility/#tool).
[^meter]: Algorithmic scope: ITU-R BS.1770-5, November 2023,
    [Algorithms to measure audio programme loudness and true-peak audio level](https://www.itu.int/rec/R-REC-BS.1770-5-202311-I).
[^meter-guide]: Meter types and interpretation: AES, [Loudness Basics](https://aes.org/resources/audio-topics/loudness-project/loudness-basics/), and iZotope, [What Is Metering in Mixing and Mastering?](https://www.izotope.com/community/blog/what-is-metering-in-mixing-and-mastering).
[^delivery]: EBU R128, version 5, 2023,
    [Loudness normalisation and permitted maximum level](https://tech.ebu.ch/files/live/sites/tech/files/shared/r/r128v5_0.pdf).
[^psd]: Method and density units: SciPy signal documentation,
    [Welch power spectral density](https://docs.scipy.org/doc/scipy/reference/generated/scipy.signal.welch.html).
    The comparison convention and band formula above are local diagnostic policy.
[^spectrogram]: Consecutive Fourier transforms show changing frequency content:
    SciPy signal documentation, [Spectrogram](https://docs.scipy.org/doc/scipy/reference/generated/scipy.signal.spectrogram.html).
    That function is marked legacy; new implementations may use its documented
    ShortTimeFFT replacement without changing the stated measurement convention.
