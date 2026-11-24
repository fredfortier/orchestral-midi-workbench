# Program and verify each phrase

Work serially on the exact open instance returned by setup. Verify the native
selected slot and binding; slot ensure owns selection. Confirm its patch and UI
against the settled binding; retain baseline content/options and recovery evidence.
Confirm exactly one instrument in the Instrument Navigator and matching rack
header. Additional Kontakt instruments are not Bitwig layers: reconcile them
through setup before programming. The integrated loader verifies an empty or sole
instrument rack, removes the sole instrument before an explicit replacement,
and verifies one resulting Navigator instrument. Never assume File > Load replaces
rack content or load manually into empty rack space.
Do not use a remembered screen position after a layout/focus change. On a
qualified regular dashboard, enter one observed row at a time and reread its
labels before the next row. Use integer window-relative click coordinates;
retain patch-specific row/column geometry only in case evidence.

1. Compare the dashboard with the phrase definition. Retain matching content.
   Otherwise use New only on the intended receiver whose baseline can be restored.
   For each cell in reading order: click that destination, click its sampled label,
   then inspect the resulting label and position. Explicitly select every next
   destination: choosing a sample can replace the current cell without advancing.
   Keep initial phrases unconnected and note-driven. On the observed 3.5 dashboard,
   outline circles are unconnected; filled/crossed connectors enable Connect/Morph.
2. Delete a mistaken cell with the toolbar Delete tool, confirm its delete-position
   prompt, then click the target and inspect its neighbors. Do not depend on
   modifier-click support. Select a cell before editing Sustain/Staccato or its
   controls; revisit it and a neighbor to establish whether the edit is local.
   Changing articulation can change attack/release defaults. Restore the planned
   articulation and values. Check capacity by entering and counting at least the
   longest required sequence on this receiver, then restore its intended content.
3. Independently read every cell, empty tail, connector and relevant option against
   the definition. Disable automatic pause reset for explicit entry control; inspect
   Agile Legato, HLS assignment, release and overlap settings. Qualify numeric MIDI
   selectors with a held, bounded low-note probe: observe the named selector activate
   and its word marker move to word one, then verify a short pulse ending before
   the first sung note. Displayed octave labels are insufficient. Record actual
   numeric note, word assignment, lead, duration and slot-selection note channel. Restore any
   experimental content; never replay uncertain input or assume host Undo restores
   script state. Prefer direct cell entry; preset insertion may discard tail cells.
4. Establish entry/reset/HLS ordering with a bounded authorized musical test.
   Establish the qualified neutral HLS state before each occurrence, then reset
   before every phrase occurrence on its slot-selection note channel. New syllables
   advance; continuation pitches retain the intended syllable. Distinguish pedal
   HLS suppressing attack from held-key repetition retaining attack. Test a phrase containing an extra continuation pitch: the cursor must retain
   that syllable and resume on the next authored syllable, finishing on the
   occurrence's last consumed cell. For a shortened prefix, leave the unused
   tail untriggered and test the next reset from that noninitial cursor. Verify which
   mechanism implements each occurrence; do not invent controller lead times.
5. Hand off musical-note assignments, reset notes, note-offs and HLS schedule.
   Use setup's standard Filter/Map chain and shared channel-1 track controls.
   Test an extra-pitch HLS passage to verify delivery and advancement. If it fails,
   return the fresh routing/control observations to the owning setup or edit
   operation; do not copy channel curves or add broadcasters/senders. Read MIDI
   behavior from INSTRUMENT.md's control map; missing patch facts return to intake.
   Release old state before reuse; do not depend on cursor wrap.
   Include the receiver's dynamics requirements in this handoff; verify initialized
   phrase dynamics separately from reset/HLS success. Use the shared
   [singing guidance](../../bitwig-edit/references/dynamics-and-expression.md#singing-patches)
   without assuming that unchanged automation suits a newly loaded patch.
6. Through the existing native edit owner, change complete clips once and reread
   independently. Preserve musical pitches/timing/velocity, expression, geometry,
   unrelated automation and routing. Document generated reset/control events
   separately from musical notes and reconcile future bundle reimport.
7. Close the configured Kontakt editor through its owning Bitwig plug-in window
   control; verify that exact editor is absent before moving to another receiver.
   Preserve the loaded instrument and its configuration. If Save is in scope,
   verify the project file write through the qualified native workflow. Keep the
   active project open for readback. Audition/export representative
   entries, a melisma, repetition and rest/restart before claiming synchronized
   singing; check every occurrence structurally. Mid-phrase seeking/loops remain
   unsupported until a verified chase strategy restores their required history.

Return actual coverage, pronunciation losses, reset/readback/save/audio evidence
and unresolved limits. Do not equate valid MIDI, a matching dashboard or an export
file's existence with correct diction or audible synchronization.
