# Historical Kontakt settings research

These images preserve earlier research only. The alternate settings/window adapter
and its durable tests are retired. They are not current procedures or runtime
inputs; stock NKI loading belongs exclusively to adapters/kontakt-ctl.

# Kontakt Header Tune observations

These 70×20 pixel crops retain only the parameter field and adjacent NI artwork.
They were observed on Kontakt7.10.1 observed 1209×887 layout, rendered at1209×887 on the isolated
owned Bitwig6.1 calibration fixture. They contain no instrument samples.

Independent expected values and source screenshots under the BWV248 project’s
`evidence/runtime-20261101/`:

| Fixture suffix | Value | Source |
| --- | --- | --- |
| full-zero | 0.00 | pitch-compensation-trumpet2-03/rack.png |
| full-normal-zero | 0.00 | pitch-compensation-trumpet2-short-01/compensation-initial-012/rack.png |
| full-two | 2.00 | header-integer-probe-01/after-blur.png |
| full-clamped | 36.00 | header-integer-probe-01/before.png |

Crop coordinates are(1010,97)–(1080,117), with exclusive right/bottom endpoints.
The clamped value is a retained failed calibration outcome, never desired state.
Tests also alter these independent images to establish rejection of missing
decimals, unknown glyphs, blinking carets, edge signs and frame-connected text.
The production reader admits exact thresholded surrounding decoration and glyphs;
these fixtures do not establish compatibility with other NI versions or scales.

`kontakt7-output-st1.png` retains the complete NI instrument Header Output label
and value field at(615,101)–(729,115). Independently observed selected channel:
`st.1`, per the NI7 manual Instrument Header p54 Output Channel control. Source:
`original-rack-investigation-01/rack-04/capture/rack.png`. Thresholded field pixels
also agree exactly in rack-08/capture, rack-16/capture and
rack-21-02/capture-01. This is an observed channel-selection reference; it does
not establish the output mixer’s host-bus wiring or audible signal delivery.

Reference tuning/menu fixtures come from rebuild-final-20261101/
reference-tuning-probe: view-menu/rack.png (Master hidden),
master-hide-menu/rack.png (Master visible), master-visible/rack.png
(Tune440.00Hz). The View menu itself displays Zoom(100%). Earlier80% labels
were inferred and are corrected; no UI zoom preference was changed.
Menu crops(264,31)-(429,268), Tune label/value crop(751,45)-(834,85).
Fixtures are thresholded grayscale (>180) for complete exact field comparisons.
These empty-instance calibration images establish layout only; final sources
require separate current readback and persistence/acoustic evidence.

Capsule fixtures crop(570,165)-(1100,365), preserving exact sidebar, labels,
selected-row markers, enable and CC fields. `capsule-xfade`, `-dynamic`, and
`-expression` come from rebuild-final-20261101/capsule-controller-probe/
normal-before-000, controller-visible, and expression-visible/rack.png. Independent
observations: XFADE; selected enabled Dynamic Xfade CC1; selected enabled
Instrument Vol.2 CC11. `capsule-velocity` comes from runtime-20261101/
pitch-compensation-trumpet2-short-01/compensation-initial-012/rack.png. Its exact
velocity label mask also matches original-rack-investigation-01/rack-21-02/
capture-01/rack.png (harpsichord). These are configuration/layout references,
not acoustic or persistence evidence. Tests perturb actual pixels, swap CC fields,
and simulate retained selection, delayed paint, late modal and changed labels.


Vibrato fixtures use the same Capsule crop. `vibrato-romantic` comes from
rebuild-final-20261101/strings-vibrato-probe/normal-before-000/rack.png;
`vibrato-strong` and `vibrato-without` come from transition-camera-02/frame-065.png
and frame-070.png. Independent observed selectors: Romantic, Strong with14voices,
and Without. `vibrato-controller` comes from vibrato-controller/rack.png: selected
Vibrato Slider, enabled switch and CC3. The sources show the exact three-label
list, not a filename/default inference. Read-only navigation and one guarded style
click are tested against these images, including unknown CC pixels and stale
selection/identity. These fixtures do not establish acoustic or saved-state proof.


Nucleus fixtures come from rebuild-final-20261101/nucleus-settings-probe:
`kontakt7-nucleus-basic` and `-advanced` crop(554,145)-(1350,550) from
basic-restored/rack.png and advanced/rack.png. Observed: selected Hit, disabled
Dynamics; Advanced shows Velocity Curve off and Linear. The separate instrument
title crop(622,81)-(1005,98) restricts the reader to Timpani Hit. The NI Header
Tune crop(1177,97)-(1247,117), `kontakt7-header-full-nucleus-zero`, independently
shows0.00 with the existing exact glyphs. Original render1376×887, Tune shift167px.
The initial Basic capture lacked painted mix buttons; its failed complete-body
comparison is preserved in the runtime evidence, and does not support a general
unchanged-settings claim. Only Basic/Advanced navigation and tab restoration
are calibrated. Manual1.4 §3.2.7/8 supports the short-articulation velocity
interpretation; installed source directory1.3.0 behavior, acoustic response and
persistence require separate evidence.


MIDI-input field fixtures come from early-P4-midi-input-03/input-{before-001,
after-000}/rack.png, crop(620,115)-(725,138): Omni and A/1. Their threshold160
masks agree exactly with early-P1-midi-input-01/input-{before-000,after-000}.
Complete port labels crop(655,131)-(808,258) agree between P1 and P4 and exclude
only the submenu-arrow column. Complete1–16 channel labels crop(842,156)-(881,547)
also agree exactly; `midi-channel-menu` retains their full surrounding submenu
from early-P4-midi-input-02/menu-channels-000. Tesseract misread even clean2/13
and the Header A; these independent exact complete-field references replace
fuzzy OCR for those observations. Original failed canaries remain in evidence.
Tests delete each of16 labels and change Header glyphs; neither a matching full
menu hash nor a desired OCR interpretation can admit unknown field pixels.

`kontakt7-capsule-dynamic-off.png` is the 550×215 Capsule panel at screen
(570,165), from the owned BWV248 P10 Choir Women Sustains dynamics readback in
`evidence/rebuild-final-20261101/all-settings-03/P10.c00-dynamics-mode/`
(`dynamics-xfade-000.png`). It retains the selected Dynamic Xfade CC1 row and
exact disabled switch. The strict reader still rejects disabled assignments.
The enable operation admits only that OFF artwork, unchanged CC1 and window
identity, one gesture, positive enabled readback and restored Performance view.

`kontakt7-logo.png` is crop (12,11)-(150,29) from the first verified batch03
Kontakt header. Its threshold160 mask is identical across all22 verified slots
and the P13 reference-read failure, where Tesseract read the same logo as
`KONITAKT`. The complete23-image calibration is retained in the owned project's
`evidence/rebuild-final-20261101/kontakt-logo-calibration-23-observations-01.json`.
This exact artwork supplies header recognition only. It cannot establish an
instrument filename, empty rack, source identity, process or native ownership.

The ARVA/FREYJA MIDI-input crops and FREYJA Tune crop were observed on
2026-11-03 in the owned Vivaldi RV589 II project, NI7.10.1 at1209×887.
Sources are `evidence/20261103-continuo/assignment-recovery-007/slot-003/rack.png`
(ARVA Boys Sopranos Ah Legato, independently read Omni) and `slot-004/rack.png`
(FREYJA Altos Ah Legato, Omni and Tune0.00). MIDI crops retain(620,115)-(725,138).
The complete label font occupies local(0,3)-(105,17); the rest is authenticated
frame artwork. Tests compose the separately observed A1 font with these frame
vectors; that composition is not a live channel-setting claim.

FREYJA/WOTAN performance/advanced crops retain(550,140)-(1187,450), from
`freyja-options-probe-003-{before,after}.png` and
`wotan-options-probe-002-{before,after}.png` in the same evidence directory.
These were bound view-only probes on the exact independently loaded FREYJA Altos
Ah Legato and WOTAN Tenors Ah Legato racks. Both Advanced Options panels show
CC#1 Dynamics and CC#11 Expression. The renderer, family artwork, gear, complete
caption and destination labels authorize view navigation separately from the
numeric assignment fields. No parameter or NKI was submitted by these probes.
Tests change assignments, caption, identity, family and modal state and check
whether restoration is independently safe. These references do not admit the
older ARVA legato layout, another player version or scale, nor establish acoustic
behavior or persistence. They contain UI artwork and no instrument samples.

`kontakt7-midi-port-labels-arva.png` retains(655,131)-(808,258) from the first
ARVA MIDI-input attempt's `menu-ports-009/rack.png`. Its five complete label
masks match the earlier standard fixture exactly through y256. Only the final
row at y257 differs: it lies below the popup and shows underlying ARVA artwork.
The reader therefore checks(655,131)-(808,257). The regression retains that
exterior row and separately removes each of the five labels to require refusal.

The later `midi-input-arva-a1` and `midi-input-freyja-a1` crops independently
record actual committed `[A] 1` headers, at the same105×23 coordinates. Sources:
`settings-remaining-002/p4.c00-midi-input/rack.png` and
`settings-remaining-003/p5.c00-midi-input/rack.png`. Both native-fenced setters
passed their actual post-commit readbacks. These observed vectors corroborate
the earlier font/frame composition test; they are not synthesized images.

`kontakt7-wotan-restored.png` comes from the independent
`wotan-options-restore-after.png` view-only probe. It records the collapsed main
view with its hovered gear, a distinct exact mask from the initial unhovered
gear. The unchanged main artwork remains a separate navigation guard. Tests use
this actual terminal frame in every WOTAN restoration flow and also start a
complete view cycle from it; altered gear artwork still refuses.

The `wotan-basses-{performance,advanced,restored}` crops retain the same region
from `wotan-basses-options-{probe-before,probe-after,restore-after}.png`. These
2026-11-03 bound view-only observations show the Basses skin's distinct exact
RGB family/main artwork. Its complete gear, caption, destination labels and
CC1/CC11 fields match the independently observed Tenors masks. Tests cover both
views and reject altered assignments in this skin as well.

`kontakt7-freyja-advanced-unhovered.png` retains the same(550,140)-(1187,450)
region from `cold-freyja-options-independent.png`. A read-only independent
2026-11-03 cold-session capture corroborated the previously unadmitted open
gear state observed in the failed cold dynamics read. Complete family artwork,
caption, destination labels and CC1/CC11 numeric masks are unchanged; only the
exact open gear mask is added. A separately guarded view-only restoration
returned the original Performance artwork. Tests read this state without input,
exercise a full view cycle, reject altered assignments after safe restoration,
and retain refusal for unknown gear artwork. No control value was submitted.

`kontakt7-wotan-advanced-unhovered.png` retains the same region from the
independent pre-restoration capture `cold-wotan-options-independent.png`. It
corroborates the cold Tenors open-icon state while the family artwork, complete
caption/destination labels and numeric fields remain unchanged. A single guarded
view-only toggle restored the original Performance artwork; no setting was
submitted. The two cold-state regression tests exercise both observed families.
The Basses profile shares the existing measured WOTAN navigation/field masks;
its distinct exact family/main artwork guards are retained.
