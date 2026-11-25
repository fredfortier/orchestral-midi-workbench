# Bitwig Control Deck and the Workbench controller

Control Deck could take over **some of the session commands we send through DrivenByMoss**, especially transport, basic track setup and exact-ID Bitwig action dispatch. It also offers useful device and plug-in controls. **Workbench still needs its document adapter for stored MIDI and automation**, even assuming Control Deck and its open draft work as designed. Control Deck reads and writes a Launcher step grid, prints clips by recording playback, and proposes recording automation with a moving playhead. Workbench addresses the stored sources, reads their full content, and verifies a scoped edit against the document. Those differences affect what survives a round trip, how a write is verified, and whether it needs playback. [Control Deck tool reference][help-tools] · [Workbench snapshot and replacement contract](../docs/architecture/controller.md#one-complete-musical-snapshot)

Scope as of 2026-11-25. The local submodule and upstream `main` resolve to [Control Deck 1.2.1, `d0996be`][deck-main]. The draft is [PR #25][pr25] at [`1f89aa6`][draft-head]. The Workbench side describes the current checkout, including its local changes. This is source analysis with offline tests. Live Bitwig behavior is assumed for the replacement comparison and remains unqualified here.

## How the two paths reach Bitwig

```text
Control Deck
  Agent -- MCP stdio --> Node server ------+
                                           +-- JSON/UDP 50701 <--> Bitwig API 18 JS
  Browser dashboard --> local status API --+                             |
                                                                      banks/cursors
                                                                      actions/browser

Workbench
  CLI -- HTTP --> midi-bridge -- native framed JSON/TCP --> Java adapter
                       |                                       |
                       |                                  document thread
                       |                                       |
                       |                                  stored sources
                       |
                       +-- optional OSC/UDP --> DrivenByMoss --> session actions
```

[HELP's component map][help-pieces] leads to the [MCP registrations and request logger][deck-mcp], the [UDP client][deck-client], and the [Bitwig receiver][deck-receiver]. The Node client sends one JSON datagram with a per-install token, request ID, method, parameters, timeout and reply port. It matches replies to pending IDs. Inside Bitwig the [dispatcher queue][deck-queue] runs one operation at a time and holds up to 32 waiting requests. [Request][deck-client] and [reply][deck-reply] each have a 60,000-byte ceiling. A correlated reply tells the caller which submitted request got a result. On timeout or loss, the queue has no retained mutation receipt to reconcile a second request with the first. Compare the Workbench [native session and frame contract](../docs/architecture/controller.md#wire-deployment-and-verification), which separates request IDs, mutation IDs, document revisions and session epochs, and retains bounded results for repeated mutation intent.

Control Deck's main safety address is zero-based bank index plus current name. [Track resolution][deck-target] refuses duplicate names except in the rename path. Device and clip operations can select the track to move a cursor. Status, transport, action dispatch and scene-wide launch have their own scope, so HELP's general exact-target and readback language needs to be read per operation. [Tool reference][help-tools] · [Transport handler][deck-transport] · [Action dispatch][deck-actions] · [Scene launch][deck-scene] [Selection cleanup][deck-selection] is best effort, limited to recoverable main-bank selections and top-level device selection on the same track. [Status][deck-actions] gives a project name and visible session state. Workbench [resolves typed paths under the exact project and parent](../docs/architecture/controller.md#paths-and-structural-reads), with native discriminators for duplicate siblings and hidden targets. A successful positional write on a uniquely named track supplies less identity evidence than a qualified native path.

The dashboard uses the [same client to poll status, tracks and the selected device chain][deck-dashboard]. Its HTTP API also supports reindexing the local plug-in library. It exposes neither the general mutation surface nor a document-content read. Device-chain polling can reach the cursor-based list operation, so a dashboard read does not establish that the UI selection stayed untouched. [Status API routes][deck-dashboard-routes] · [Device-list dispatch][deck-dispatch] · [Track selection][deck-target]

## Replacement by operation

| Operation | What Control Deck covers | Implication for Workbench |
| --- | --- | --- |
| Session status, play/stop, seek, scalar tempo | [Transport handler][deck-transport] submits play/stop/seek and returns observed transport after a delay. The tempo path checks the requested value. [Status and action dispatch][deck-actions] also expose engine state and the action registry. | Strong candidates to retire corresponding direct OSC calls after qualification. Keep the existing [project and effect checks](../skills/bitwig-explore/references/session-actions.md#direct-actions). The typed transport tool does not implement meter, engine activation, or per-track Launcher audition and return to Arrangement. |
| Configured OSC actions and feedback | [Action lookup and invocation][deck-actions] use Bitwig's exact action ID, so an agent could find actions without assigning numbered DrivenByMoss slots. The reply `invoked: true` records the dispatch, with no inspection of its effect. | Candidate for the [five qualified bindings](../skills/bitwig-explore/references/session-actions.md#qualified-configured-profile) if their exact IDs, starting editor context and outcomes match. Empty-clip creation, split, zoom, lane visibility and focus still need the same document or viewport verification. Save, Undo/Redo, and other generic actions likewise need their own qualification. Existing clients that need raw addresses, typed arguments or asynchronous feedback still use the [OSC submission path](../crates/controller/src/osc_session.rs#L106-L132). |
| Tracks, mixer and cues | [Track and mixer tools][deck-track-mcp] cover basic creation, rename/delete and normalized volume/pan, mute/solo/arm. [Mixer readback][deck-mixer], [peak meters][deck-meters] and [cue list/upsert][deck-cues] add useful session controls. Main and effect banks have separate limits of 128 and 32. | Good coverage for simple session tasks. For child-track creation, verify the resulting ancestry before replacing the current [OSC workflow](../skills/bitwig-explore/references/session-actions.md#direct-actions). Workbench still supplies qualified hierarchy, native mixing/routing scope and audio export. Use current [mix and export semantics](../docs/architecture/controller.md#mixing-and-audio-export) when project, bus, return, signal path, units or render selection matter. |
| Device, sample and plug-in operations | [Device and sample tools][deck-device-mcp] search Bitwig's Devices or Samples browser, insert stock devices by UUID or CLAP by ID, and insert samples into Sampler. [Nesting and preset-save tools][deck-nest-mcp] and [parameter tools][deck-parameters-mcp] add chain operations, direct parameter IDs and Remote Controls. [Direct parameter write][deck-param-set] checks a normalized value. | Useful additions for building tracks and operating plug-ins. Current parameter values are control state. They do not identify or read stored automation on those parameters. Browser and insertion operations do not supply Workbench's [qualified receiver routing and exact Kontakt patch assignment](../docs/architecture/controller.md#integrated-instruments). |
| Launcher notes and scenes | [Clip tools][deck-clip-mcp] list, read, create or replace a Launcher note clip and launch a scene. Single-clip grid content is verified after creation. | Suitable for generating and inspecting simple grid-aligned patterns. Workbench [Launcher and Arranger content reads](../docs/architecture/controller.md#one-complete-musical-snapshot) still own full stored note data and edits on existing identities. Scene launch also differs from auditioning one qualified track/slot. |
| Arranger MIDI, automation and export | The released [clip dispatcher][deck-dispatch] offers Launcher-to-Arranger recording. The [parameter handlers][deck-param-set] set current values. The draft adds cursor inspection and a recorded automation ramp. | No equivalent to Workbench's [complete native note/automation edit](../docs/architecture/controller.md#replacement-ownership-and-failure) or [document capture and MIDI 2.0 assembly](../docs/architecture/controller.md#self-contained-midi2-export). The optional `osc` crate itself only supplies message encoding and decoding for DrivenByMoss. It is independent of native content access. [OSC contract](../crates/osc/README.md#drivenbymoss-osc) |

Transport, basic track setup and qualified action dispatch are the candidates to shrink the OSC dependency. There is no meaningful percentage based on Control Deck's tool count. A generic Bitwig action can subsume a configured OSC slot while still lacking typed project targeting and effect readback. Conversely, a single Workbench content read may include many tracks' worth of notes and automation. [Action invocation][deck-actions] · [Qualified action preconditions](../crates/osc/ACTIONS.md#qualification) · [Workbench structural and content scopes](../docs/architecture/controller.md#one-complete-musical-snapshot)

## MIDI and automation fidelity

```text
Control Deck notes    track + slot --> select cursor --> step cells --> JSON
Control Deck draft    parameter + beat range --> record gesture --> 2 samples

Workbench read        exact project + paths --> stored notes, expressions, curves
Workbench change      observed + desired --> preflight --> one native Undo step
                                                       --> independent reread
```

**Launcher grid.** [HELP describes quantized reads][help-clips]. The [writer][deck-note-write] requires on-grid starts, writes 0–15 MIDI channels and 7-bit attack velocity through `setStep`, and scrolls in 256-step pages. The [reader][deck-note-read] consults the note-step observer, confirms each candidate with `getStep`, and reports a grid beat, pitch, velocity, duration, channel and mute. The [comparison][deck-note-verify] checks the cell and note values, including missing, extra and muted notes. There are no fields in this surface for stable note IDs, release velocity, operators or expression graphs. The MCP request caps creation at 750 notes and 512 beats, with a default quarter-beat step and a minimum 1/32-beat step. An occupied slot with `replace: true` follows [delete, create, fill, verify][deck-replace]. Verification failure can follow mutation. The handler reports the failure with no restoration of the deleted clip.

**Arranger print.** The [print schema][deck-print-mcp] accepts only slots 0–7 although ordinary clip and scene tools reach slot 1023. The [script's limits and recording logic][deck-limits] cap a print at 1,024 beats. It schedules Launcher events against the running playhead, temporarily disables loop and overdub, disarms tracks and records at 666 BPM. [Completion and cleanup][deck-print] return scheduled/recorded intervals and restored transport observations. They do not enumerate the resulting clips or their notes. This is useful for printing patterns, with a different success condition from verifying exact native Arranger placement, clip/source coordinates and note content.

**Stored content.** The Workbench [editor adapter](../crates/controller/adapters/bitwig-ctl/src/main/java/com/midiworkbench/notes/internal/EditorAdapter61.java#L124) can read explicit paths and apply a plan against frozen content. Its [automation adapter](../crates/controller/adapters/bitwig-ctl/src/main/java/com/midiworkbench/notes/internal/AutomationAdapter61.java#L63) resolves lane/target owners, including automation outside note clips. It preserves physical domains, placement and shared-source aliases, and refuses a complete read on an unresolved stored owner. The [mutation preflight](../crates/controller/adapters/bitwig-ctl/src/main/java/com/midiworkbench/notes/internal/AutomationAdapter61.java#L166) and [independent reread](../crates/controller/adapters/bitwig-ctl/src/main/java/com/midiworkbench/notes/EditorSession.java#L185-L194) support note/expression changes and complete curve edits, including MIDI CC lanes, in one Undo transaction. [Native capture](../crates/controller/adapters/bitwig-ctl/src/main/java/com/midiworkbench/notes/internal/MidiCapture61.java#L17-L42) adds timing and hierarchy for Rust [MIDI 2.0 assembly](../crates/controller/src/export_midi.rs#L99). The draft gesture does not add any of these representations.

## Upstream work in progress

The public [issue][all-records] and [PR][all-pulls] inventories returned no standalone issues in either state. [PR #25][pr25] was the only open PR, still a draft opened 2026-10-30, with seven commits and no reviews, comments, or reported checks on its head. The draft adds six tools, taking the registry from 35 to 41. The other 24 PRs were merged. The relevant released changes include [Launcher note reads (#4)][pr4], [note readback (#5)][pr5], the [reply cap (#8)][pr8], [print cleanup (#9)][pr9], [scene paging (#10)][pr10], and [selection cleanup (#14)][pr14]. [Release #24][pr24] is the released baseline here. The [1.2.0 changelog][deck-release] reports a live Bitwig 6.1.3 test for the preceding release. I found no public Linux-port PR or issue in that inventory.

| Draft change | What the branch actually implements |
| --- | --- |
| Arranger discovery and rename | [Listing][draft-arranger] walks the cursor only if both `selectFirst` and `selectNext` exist. Otherwise it returns the selected clip with `complete: false`. Name and stretch mode are marked unreadable. [Rename][draft-rename] verifies a Launcher name through the slot. Arranger uses track name plus cursor play-start, and can reply successfully with `verified: false`. Neither tool reads Arranger notes or creates a native clip inventory. |
| Automation ramp | The [target resolver][draft-target] accepts volume, pan, a Remote Control or a direct parameter ID. Direct parameters have no touch/release implementation. The [writer][draft-write] stops transport, disables loop and overdubs, changes tempo, enables Arranger automation write in latch mode and drives one linear or power-shaped ramp for up to 256 beats. It schedules updates roughly every 20 ms while playback advances. [Verification][draft-verify] samples the value at the start and at end minus 0.01 beat, with a normalized tolerance of 0.08. It cannot establish interior shape, stored point order, lane ownership, or preservation around the range. Failure restores session controls and attempts to restore a direct parameter value. The code does not roll back recorded points, and cleanup leaves transport stopped. |
| Audio-file placement | The [MCP wrapper][draft-audio-mcp] reads WAV/AIFF/FLAC timing. The [host handler][draft-audio] computes Raw beat length at current BPM and returns `placed: false`. The tool name describes a requested action that this implementation declines. |
| Presets and cues | [Preset search and load][draft-presets] cover Bitwig's preset browser or absolute `.bwpreset` / `.vstpreset` files. They have no route to a plug-in's internal preset browser. The load compares observed device and preset names and attempts Undo on a detected replacement or mismatch. [Cue additions][draft-cues] add rename/delete checks against the cue bank. |
| Print arm and timeout recovery | [Arm restore][draft-arms] reuses saved scroll/index/name and falls back to a name lookup. That helps with scrolling and still does not supply native identity when names collide. The [deadline change][draft-timeout] tightens request cleanup, and the [client waits beyond the host deadline][draft-client] for a structured timeout. Neither adds a stored-content rollback or replay receipt. |

The ramp handler also replaces the request cleanup callback after device resolution. For device targets, that can discard the saved-selection restoration installed during track selection. Include same-track and different-track selection in the live pass. [Draft cleanup registration][draft-write] · [Draft selection restoration][draft-selection]

The [PR's live checklist][pr25] calls for a Bitwig 6.1.3 pass on cursor navigation, arm restoration during bank scrolling, volume and Remote Control ramps, direct-parameter refusal, real preset loads and print timeout recovery. Code inspection adds a useful acceptance distinction for ramps. Compare the actual stored lane before and after if the requirement is editing a curve. Two playhead values alone can agree with the request even when the intermediate or adjacent curve differs. [Draft verification][draft-verify] · [Native curve representation](../docs/architecture/controller.md#one-complete-musical-snapshot)

## Linux port and `midi-bridge`

[Upstream PORTING.md][porting] describes macOS as the supported install, with a portable Node/API 18 core. For a headless Linux bridge, the essential pieces are the stamped controller script in Bitwig's configured script directory, Node with its installed dependencies, and the matching token. The documented Linux default is `~/Bitwig Studio/Controller Scripts`. The [current setup script][deck-setup-paths] hardcodes the Mac path and LaunchAgents directory, and [stamps and writes the controller][deck-stamp]. Platform-specific paths and deployment are the first porting work. Keep upstream's loopback and token requirements. Qualify the installed script's bind, matching-token response and packet sizes on the actual Linux Bitwig host before using it for production. [Client configuration][deck-client] · [Host registration][deck-receiver]

The name *MIDI helper* refers to the [CoreMIDI virtual endpoints][deck-midi-helper], used only by [Bitwig auto-discovery][deck-limits]. [HELP's manual-add path][help-install] leaves the ports unassigned with control over UDP. Upstream's suggested ALSA sequencer helper would restore the convenience of auto-detection. It would neither transport notes nor add storage APIs. The Workbench `midi-bridge` executable is a separate Linux-deployed service with a [native extension installation and smoke procedure](../crates/controller/adapters/bitwig-ctl/AGENTS.md#update-and-smoke-test-on-linux). A Linux Control Deck port can provide a second session-control peer, with no wire compatibility with that Java adapter.

For the full upstream product, [PORTING.md][porting] also calls for systemd user services, a platform installer path and Linux plug-in locations. The [scanner][deck-scanner] also assumes macOS bundle suffixes and `Contents/Info.plist` via `/usr/bin/plutil`. A useful Linux inventory needs Linux discovery and either native metadata or an explicit metadata fallback. Keep the dashboard and optional wrapper outside the minimal command-path qualification. The existing dashboard API, scanner and wrapper are separate from the MCP-to-controller connection. [Component map][help-pieces] · [Build and service setup][deck-setup-services]

Offline on Linux with Node v26.12.0, root `npm run check` and `npm test` passed at `d0996be` (104 tests) and `1f89aa6` (129 tests in an extracted source archive). The [test commands][deck-package] exercise the [Node client against an echo peer][deck-client-tests] over local UDP and the controller in a [fake host with a manual clock][deck-harness]. The [draft ramp fixture][draft-tests] simulates stored parameter behavior. These runs establish syntax and offline behavior. Linux Bitwig, the installer, dashboard build, and real controller API timing were outside the runs.

## Document-driven access with fewer round trips

A practical direction is to expose the existing Workbench operations through a browsable, documented tool surface and borrow Control Deck's concise descriptions, targeted schemas, and observed-value results. [Control Deck MCP schema and logger][deck-mcp] · [Workbench command grammar](../docs/architecture/controller.md#command-grammar)

```text
Document operation                     Session action

info: project + scoped structure        resolve project + current editor
    |                                       |
read: complete selected owners          dispatch exact qualified action
    |                                       |
replace: observed + desired             observe document / viewport / audio
    |
independent reread + operation receipt
```

1. **Structure first.** Reuse the [native info/selection contract](../docs/architecture/controller.md#paths-and-structural-reads). Get the project's identity, selection and smallest relevant subtree without shipping note/curve data or unrelated plug-in parameters. Return native paths a caller can pass directly to the next operation. Discover expensive parameter catalogs only when requested.
2. **Read by musical scope.** Collect all relevant clip and automation owners in one [complete scoped read](../docs/architecture/controller.md#one-complete-musical-snapshot). Keep source and played coordinates distinct. Full arrays within that scope carry exact note and curve detail together, including automation-only tracks. This avoids round trips per note, controller page or curve point. The document frame has a [16 MiB ceiling](../crates/controller/src/frame.rs#L3), while a 512-beat Launcher read at 1/32-beat steps would visit 64 step windows with a scheduled 180 ms wait per window, before cursor setup. That is a schedule estimate derived from the [reader's loop][deck-note-read].
3. **Use the same write and receipt path for every client.** Translate an explicit desired snapshot through the existing [prepare, plan and replace flow](../docs/architecture/controller.md#replacement-ownership-and-failure). One batch can change notes and curves and verify the same scope, with one native history step and recovery by operation identity. If MCP is useful, put its musical tools over that contract. Keep Deck's grid and real-time ramp operations explicit for callers who specifically want those semantics.
4. **Qualify session substitutions by effect.** Start with play/stop/seek and scalar tempo. Then qualify each action ID in the exact contexts already covered by the [DrivenByMoss profile](../skills/bitwig-explore/references/session-actions.md#qualified-configured-profile). Compare same-project targeting, preserved selection and the observed result. Retire only the routes with equivalent outcomes. For document access, measure elapsed time, returned bytes, and calls per scoped read/edit against a dense clip, multiple owners and an automation-only track. [Workbench verification contract](../docs/architecture/controller.md#wire-deployment-and-verification)

[deck-main]: https://github.com/remymazmanian/bitwig-control-deck/commit/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d
[draft-head]: https://github.com/remymazmanian/bitwig-control-deck/commit/1f89aa637f49d012b77c886c4982a421af2c59ba
[all-records]: https://github.com/remymazmanian/bitwig-control-deck/issues?q=is%3Aissue
[porting]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/PORTING.md
[help-tools]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/HELP.md#mcp-tool-reference
[help-pieces]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/HELP.md#the-pieces-in-plain-language
[help-install]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/HELP.md#setting-up-bitwig
[help-clips]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/HELP.md#clips--arrangement
[deck-mcp]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/src/index.mjs#L1-L81
[deck-client]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/src/bitwig-client.mjs#L5-L104
[deck-receiver]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/controller/ControlDeck.control.js#L3042-L3071
[deck-queue]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/controller/ControlDeck.control.js#L595-L654
[deck-reply]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/controller/ControlDeck.control.js#L557-L593
[deck-target]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/controller/ControlDeck.control.js#L319-L359
[deck-selection]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/controller/ControlDeck.control.js#L361-L409
[deck-actions]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/controller/ControlDeck.control.js#L2390-L2448
[deck-dashboard]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/src/dashboard-server.mjs#L33-L76
[deck-dashboard-routes]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/src/dashboard-server.mjs#L79-L129
[deck-dispatch]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/controller/ControlDeck.control.js#L2458-L2530
[deck-transport]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/controller/ControlDeck.control.js#L1240-L1266
[deck-track-mcp]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/src/index.mjs#L143-L174
[deck-mixer]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/controller/ControlDeck.control.js#L1620-L1661
[deck-meters]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/controller/ControlDeck.control.js#L2818-L2840
[deck-cues]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/controller/ControlDeck.control.js#L1268-L1461
[deck-device-mcp]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/src/index.mjs#L254-L323
[deck-nest-mcp]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/src/index.mjs#L176-L200
[deck-parameters-mcp]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/src/index.mjs#L325-L384
[deck-param-set]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/controller/ControlDeck.control.js#L2723-L2788
[deck-clip-mcp]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/src/index.mjs#L202-L252
[deck-note-write]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/controller/ControlDeck.control.js#L1689-L1744
[deck-note-read]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/controller/ControlDeck.control.js#L1746-L1802
[deck-note-verify]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/controller/ControlDeck.control.js#L1804-L1849
[deck-replace]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/controller/ControlDeck.control.js#L1918-L2021
[deck-print-mcp]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/src/index.mjs#L221-L233
[deck-limits]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/controller/ControlDeck.control.js#L1-L33
[deck-print]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/controller/ControlDeck.control.js#L2182-L2346
[deck-setup-paths]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/scripts/setup.mjs#L21-L33
[deck-stamp]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/scripts/setup.mjs#L125-L148
[deck-setup-services]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/scripts/setup.mjs#L150-L239
[deck-midi-helper]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/native/control_deck_midi_ports.c
[deck-scanner]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/src/plugin-index.mjs#L9-L98
[deck-package]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/package.json#L9-L18
[deck-harness]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/test/support/load-controller.mjs#L4-L38
[draft-arranger]: https://github.com/remymazmanian/bitwig-control-deck/blob/1f89aa637f49d012b77c886c4982a421af2c59ba/controller/ControlDeck.control.js#L2546-L2695
[draft-rename]: https://github.com/remymazmanian/bitwig-control-deck/blob/1f89aa637f49d012b77c886c4982a421af2c59ba/controller/ControlDeck.control.js#L2988-L3094
[draft-target]: https://github.com/remymazmanian/bitwig-control-deck/blob/1f89aa637f49d012b77c886c4982a421af2c59ba/controller/ControlDeck.control.js#L2700-L2772
[draft-write]: https://github.com/remymazmanian/bitwig-control-deck/blob/1f89aa637f49d012b77c886c4982a421af2c59ba/controller/ControlDeck.control.js#L2698-L2914
[draft-verify]: https://github.com/remymazmanian/bitwig-control-deck/blob/1f89aa637f49d012b77c886c4982a421af2c59ba/controller/ControlDeck.control.js#L2916-L2972
[draft-audio-mcp]: https://github.com/remymazmanian/bitwig-control-deck/blob/1f89aa637f49d012b77c886c4982a421af2c59ba/src/index.mjs#L352-L378
[draft-audio]: https://github.com/remymazmanian/bitwig-control-deck/blob/1f89aa637f49d012b77c886c4982a421af2c59ba/controller/ControlDeck.control.js#L3100-L3129
[draft-presets]: https://github.com/remymazmanian/bitwig-control-deck/blob/1f89aa637f49d012b77c886c4982a421af2c59ba/controller/ControlDeck.control.js#L3132-L3296
[draft-cues]: https://github.com/remymazmanian/bitwig-control-deck/blob/1f89aa637f49d012b77c886c4982a421af2c59ba/controller/ControlDeck.control.js#L1356-L1418
[draft-arms]: https://github.com/remymazmanian/bitwig-control-deck/blob/1f89aa637f49d012b77c886c4982a421af2c59ba/controller/ControlDeck.control.js#L323-L359
[draft-timeout]: https://github.com/remymazmanian/bitwig-control-deck/blob/1f89aa637f49d012b77c886c4982a421af2c59ba/controller/ControlDeck.control.js#L662-L726
[draft-client]: https://github.com/remymazmanian/bitwig-control-deck/blob/1f89aa637f49d012b77c886c4982a421af2c59ba/src/bitwig-client.mjs#L74-L104
[draft-tests]: https://github.com/remymazmanian/bitwig-control-deck/blob/1f89aa637f49d012b77c886c4982a421af2c59ba/test/controller-automation.test.mjs#L6-L140
[pr4]: https://github.com/remymazmanian/bitwig-control-deck/pull/4
[pr5]: https://github.com/remymazmanian/bitwig-control-deck/pull/5
[pr8]: https://github.com/remymazmanian/bitwig-control-deck/pull/8
[pr9]: https://github.com/remymazmanian/bitwig-control-deck/pull/9
[pr10]: https://github.com/remymazmanian/bitwig-control-deck/pull/10
[pr14]: https://github.com/remymazmanian/bitwig-control-deck/pull/14
[pr24]: https://github.com/remymazmanian/bitwig-control-deck/pull/24
[pr25]: https://github.com/remymazmanian/bitwig-control-deck/pull/25
[all-pulls]: https://github.com/remymazmanian/bitwig-control-deck/pulls?q=is%3Apr
[deck-release]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/CHANGELOG.md#120---2026-10-28
[deck-client-tests]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/test/client.test.mjs#L12-L36
[deck-scene]: https://github.com/remymazmanian/bitwig-control-deck/blob/d0996be52acf1a0e5d35ff6cc40cd7577f0b415d/controller/ControlDeck.control.js#L2380-L2388
[draft-selection]: https://github.com/remymazmanian/bitwig-control-deck/blob/1f89aa637f49d012b77c886c4982a421af2c59ba/controller/ControlDeck.control.js#L424-L447
