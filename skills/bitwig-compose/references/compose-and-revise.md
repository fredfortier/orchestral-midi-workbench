# Compose and revise

## Write a part against the existing music

Resolve the donor material, the intended owner and the edit range through
[exploration](../../bitwig-explore/SKILL.md). Use
[score analysis](../../score-analysis/SKILL.md) when the function of the material
or its harmonic context is unclear. Use existing cue anchors when applicable.
Make the relation between the candidate and its donor explicit, including what
it does between important arrivals and at the principal arrival. Send a
representative connecting phrase and climactic figure, or the requested full
revision, to [editing](../../bitwig-edit/SKILL.md). Carry written precedent,
editorial invention, stored result and audition status into project notes.

## Audition a repair, then apply it

1. Resolve both surfaces through [exploration](../../bitwig-explore/SKILL.md).
   Record the source notes and arranger occurrence, then establish the intended
   Launcher owner and scene. For an empty destination require an actual native
   `launcher read`. Preserve the source arranger content during prototyping.
2. Use [editing](../../bitwig-edit/SKILL.md) to create or replace the intended
   Launcher clip and independently reread it. Use
   [session actions](../../bitwig-explore/references/session-actions.md) to
   audition and return playback to the Arranger. Check that the original
   Arranger clip is unchanged.
3. Apply only the candidate the user approved or explicitly delegated for
   application. Find the intended class of occurrences from their musical
   function, score markings, and native notes. Match transpositions or
   rhythmic variants where they serve the same function. Preserve adjacent
   figures and any source passages that appear similar but are outside the
   class.
4. Send the exact Arranger clip paths and approved changes to editing. Verify
   the amended occurrences, surrounding content and any shared source aliases.
   For a Launcher transfer, compare the source and destination in the same
   beat coordinates and match numeric values numerically. Compare independent
   readback directly with the approved candidate as well as the write request.
   For saved delivery, Save and verify through session actions.

An approved faster trill can transfer to other explicitly marked repetitions
and transpositions. An otherwise similar written note with only a wavy line
stays outside that class until the score and user direction establish its role.

## Append to existing receivers

Resolve the ending, exact insertion beat, receiving tracks and layers, and the
existing meter/tempo at the join. Identify the append's first meter/tempo,
preservation boundary and listening check. When the pulse carries forward, use
an exactly adjacent empty range on the intended tracks and leave timing
unchanged. Qualify any required local timing change before altering the
existing arrangement. For an experimental append to existing work, use the
[project checkpoint](../../WORKFLOW.md#experimental-project-backups).

Pass the timed material, receiver mapping and exact boundary to
[realization](../../score-realization/SKILL.md) for score-derived material or
[editing](../../bitwig-edit/SKILL.md) for an explicit playable passage. Use the
[session actions](../../bitwig-explore/references/session-actions.md) for any
qualified host action required for the join. Use the procedure below when the
join is an exact barline with a held tempo before an abrupt change. Verify the
previous closing bars, native global state across the join and the first
appended phrase. Save and audition the join when the requested delivery calls
for it.

### Change meter and step tempo at a barline

1. Record the exact join in native quarter-note beats and the corresponding
   Arranger barline. Capture the receiving track's native meter and
   beat-to-seconds samples before changes. Use a native range around the join
   and the qualified Arranger zoom to frame the barline.
2. For a new meter, use **Insert Time Signature Change** in the beat ruler's
   context menu at that barline. In the Point Inspector confirm its position,
   and set and reread the numerator and denominator. Establish the meter
   before creating the appended clip if the clip should inherit that meter.
3. For a tempo step from a constant baseline, use **Add Automation Lane** in
   the transport Tempo control's context menu. Double-click in the Tempo lane
   at the join to add a point. Select only that point, verify its Inspector
   position, then use the Inspector's numeric entry to set and reread its BPM.
   On Bitwig 6.1, triple-click the numeric value for text entry. Verify the
   preceding point has the old BPM and enable its **Hold**. The first inserted
   breakpoint on a flat lane can supply a beat-zero point with the old BPM.
   A join point without Hold on its predecessor can create a ramp through
   preceding music. An existing nonconstant curve requires a separate
   preservation check before changing its last segment.
4. Use `midi-bridge controller midi capture --path track.json --output timing.json`
   on a track spanning both sides. Compare `globals` against the original
   meter and elapsed times up to the boundary, and check the new meter and
   seconds per beat after it. A quarter-note step to 50 BPM takes 1.2 seconds
   per beat. Independently reread the previous and appended clips. For saved
   delivery, verify the timing map from the persisted project as well.

[Bitwig time signature changes](https://www.bitwig.com/userguide/latest/playing_back_the_arranger/#time-signature-changes) · [Bitwig automation](https://www.bitwig.com/userguide/latest/automation/)
