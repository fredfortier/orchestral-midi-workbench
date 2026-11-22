# Proposed DrivenByMoss action bindings

The target profile is Bitwig Studio 6.1 with DrivenByMoss 26.6.5. Configure the
numbered actions in **Settings → Controllers → Open Sound Control OSC**.
Each binding combines the category and action label shown in Bitwig. Send
`/action/N` with the slot number in the address and zero OSC arguments.
The extension dispatches the action assigned to that slot in the active
project and editor context. [DrivenByMoss dispatcher](https://github.com/git-moss/DrivenByMoss/blob/7348cda4e5ae8aebd59bb99b8d967a1f4b87677e/src/main/java/de/mossgrabers/controller/osc/module/ActionModule.java#L53-L74)

| Slot | Category | Bitwig action | Intended use and acceptance check |
| --- | --- | --- | --- |
| 1 | General | **Create Empty Clip** | With an exact empty time range on one instrument track, create one empty arranger Note Clip at the requested start and duration. Verify track, geometry, empty notes, unchanged neighboring clips and curves, and stopped transport. |
| 2 | Arranger | **Zoom Arranger to Selection** | Frame the selected arranger object or time range in the viewport. Check actual viewport bounds and verify native selection and stored content are unchanged. |
| 3 | Arranger | **Toggle Automation Lanes for Selected Tracks** | Reveal or hide automation lanes on the intended track headers. Observe visibility before toggling, verify the intended lane is revealed, and check neighboring tracks and stored curves. |
| 4 | General | **Split** | Split one known arranger clip at a qualified edit point. Verify the split point, resulting clip and source bounds, and stored notes and expression across the split. Check that unrelated curves and neighboring content stay intact. |
| 5 | Project | **Focus/toggle Arranger Timeline Panel** | Establish Arranger Timeline Panel focus when an action requires it. Test from both a different panel and an already-focused Arranger. Verify actual focus independently of panel visibility, and document any toggle side effect. |

Reserve slots 6–20 for later qualification. Use direct OSC endpoints for
existing session actions such as seek, Save, Undo/Redo, and note-editor
visibility. Use native entity reads for arranger inventory, native selection
for qualified paths and ranges, and native replacement for stored notes,
expression, and curves. [Document navigation evidence](../../research/WORKSHEET.md#7-arranger-document-navigation)

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

Run this from `crates/` to invoke slot 1.

```sh
cargo run --locked -p osc -- send 127.0.0.1:8000 /action/1
```

Slot 1 was observed as **Create Empty Clip** in the settings before this
specification was written. Its effect is unverified. Slots 2–5 above are
proposed assignments.
