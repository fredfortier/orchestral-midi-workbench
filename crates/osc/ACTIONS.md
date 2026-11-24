# DrivenByMoss action qualification

The target profile is Bitwig Studio 6.1 with DrivenByMoss 26.6.5. Configure the
numbered actions in **Settings → Controllers → Open Sound Control OSC**.
Each binding combines the category and action label shown in Bitwig. Send
`/action/N` with the slot number in the address and zero OSC arguments.
The extension dispatches the action assigned to that slot in the active
project and editor context. [DrivenByMoss dispatcher](https://github.com/git-moss/DrivenByMoss/blob/7348cda4e5ae8aebd59bb99b8d967a1f4b87677e/src/main/java/de/mossgrabers/controller/osc/module/ActionModule.java#L53-L74)

Use the [production profile](../../skills/bitwig-explore/references/session-actions.md#qualified-configured-profile)
for the installed categories, action labels and preconditions for slots 1–5.
Use the direct protocol where it already covers the intent. Slots 6–20 are
available for actions whose observable result justifies another binding.

## Qualification

- Confirm the live assignment and active disposable project. Resolve the exact
  target through the native controller. For a context-sensitive action, prove
  the required editor and selection state before dispatch.
- Send once, then independently verify the stated result. Check document
  mutations with native reads and check UI outcomes through actual viewport
  or focus observation. A successful UDP send establishes local submission.
- Exercise the closest wrong-context case on a disposable project. For
  **Split**, first establish the exact native context that supplies the split
  point. For visibility and focus toggles, check both starting states before
  treating the action as a reusable route.
- After silence or an ambiguous effect, reread state and reconcile the first
  attempt before considering another send. Record the installed version, slot
  binding, target, initial context, action and observed result.

Use the bridge's generic OSC entry point to invoke slot 1.

```sh
midi-bridge osc /action/1
```

## Exercise the boundary

Use the [production session recipe](../../skills/bitwig-explore/references/session-actions.md)
for the qualified context and effect checks. Retain results in the
[Bach exercise](../../research/QUI-TOLLIS.md).

- Establish whether the native range plus slot 5 supplies the focus needed for
  slot 1. The Bach exercise verified shared empty-range placement on two
  instrument tracks and then on ten. Its occupied-range probe replaced the
  middle of an existing clip with an empty clip. Prove each intended track's
  range is empty before dispatch.
- Compare slot 2 to native selection, which can select an off-screen passage,
  and slot 3 to native lane lookup, which can address a hidden lane.
- For slot 4, compare visible occurrences and stored source content across both
  outputs. Test a range beginning outside the clip before admitting an edit at
  one exact cut. Source notes outside either playback window can legitimately
  survive in stored content.
- For slot 5, exercise the already-focused case to catch a hidden timeline.
- Before adding a binding, compare with the direct protocol. Global
  `/time/signature` and `/tempo/raw` qualify for initial timing. The
  [barline transition](../../skills/bitwig-compose/references/compose-and-revise.md#change-meter-and-step-tempo-at-a-barline)
  uses visible project timing points and a native timing readback. An OSC
  replacement must preserve the timing samples on the original side of the
  boundary. Project new/open/first-save and destructive focus-dependent OSC
  routes stay open until tested in their actual context.
- Verify OSC project focus against the exact native project path before writes.
  When the remote and visible project disagree, the named File → Save action
  has a verified disk outcome on the visible project. Keep remote writes
  gated until the target is resolved.
