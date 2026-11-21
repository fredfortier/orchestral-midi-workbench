# Bitwig Java adapter

- Read the [native host contract](../../ARCHITECTURE.md#native-host-execution) before changing SDK or private document calls. Use [wire and deployment rules](../../ARCHITECTURE.md#wire-deployment-and-verification) when changing the adapter-service boundary.

## Build checks

- From this directory, use JDK 25 or newer and run `./mvnw verify` for production changes. Also run `./mvnw -P harness verify` for harness or native-boundary changes. Keep each profile in its own output tree. Use the admitted Bitwig runtime JAR and override the [POM](pom.xml) default with `-Dbitwig.runtime.jar=/absolute/path/to/bitwig.jar` when needed.
- For Rust boundary changes, run `cargo check --locked -p notes-transformer --all-targets` and `cargo clippy --locked -p notes-transformer --all-targets -- -D warnings` from `crates/`. Apply the [workspace checks](../../../AGENTS.md#verification) as relevant.

## Update and smoke-test on Linux

1. When deployed extension bytes change, increment the [Java wire](src/main/java/com/midiworkbench/notes/Wire.java), [Rust wire](../../src/protocol.rs), and [extension version](src/main/java/com/midiworkbench/notes/NotesAdapterDefinition.java) together.
2. Use a disposable project in the admitted Bitwig 6.1 host and keep the qualification extensions and ports independent of the user's setup. Reconcile pending operations before replacing either the listener or the controller.
3. From `crates/`, run `cargo build --locked -p notes-transformer`. If a matching qualification listener is absent, stop any old qualification listener and start `./target/debug/midi-bridge serve` in a separate terminal. Leave Bitwig open.
4. From `crates/`, run the [installer](../../scripts/install-extension.sh). It requires the JDK, `jq`, and `flock`, and publishes the new wire-versioned extension before retiring prior builds. Identical bytes can be reinstalled. Different bytes at the same wire path cause the installer to stop.

   ```sh
   bash controller/scripts/install-extension.sh
   ```

   Set `BITWIG_RUNTIME_JAR` to override the POM's runtime JAR. Set `MIDI_WORKBENCH_EXTENSIONS` to the qualification profile's extension directory if it differs from `$HOME/Bitwig Studio/Extensions`. Retired builds are under `${XDG_STATE_HOME:-$HOME/.local/state}/midi-workbench/deployments/retired/`.
5. Bitwig 6.1 can load the published extension while open. On first setup, add **Notes Document Adapter** in Settings → Controllers. Run the [smoke](../../scripts/smoke-bitwig.sh) and compare the project UUID and cue count with the fixture.

   ```sh
   bash controller/scripts/smoke-bitwig.sh
   ```

   A pass requires a native cue read through the matching CLI, listener and adapter. If reload is in progress, use `./target/debug/midi-bridge adapter bitwig connection list` to confirm the intended live process and wire, then rerun smoke. If loading fails, inspect the listener and Bitwig logs before restarting the qualification host.

   The listener defaults to native port 8788 and API port 8789. For custom ports use `serve --port NATIVE_PORT --api-port API_PORT`, launch Bitwig with `MIDI_WORKBENCH_ADAPTER_PORT` set to the native port, and pass the API port to the smoke script.

Bitwig documents live installation by dropping a `.bwextension` onto the app. The bundled SDK also documents controller-only restart in `resources/doc/control-surface/api/com/bitwig/extension/controller/api/ControllerHost.html#restart()`. Use the smoke to verify that the new classes actually loaded. [Bitwig extension installation](https://www.bitwig.com/support/technical_support/how-do-i-add-a-controller-extension-or-script-17/)

## Development harness

- Add `--harness` to the installer and enable **Notes Editor Harness** only on the disposable profile. To refresh its development code from `crates/`, run `bash controller/scripts/install-extension.sh --harness --code-only`, then `./target/debug/midi-bridge adapter bitwig probe reload`. Check the probe's `error` field and reload result.
- The default scratch directory is `/tmp/notes-editor-harness`. To override it, set `NOTES_HARNESS_DIRECTORY` for the installer and CLI and the JVM property `notes.harness.directory` for Bitwig to the same directory.
- Use focused ephemeral probes. Keep fault injection and history probes in the harness. Report the checks run, fixture and build identities, independent readback, and open limits under the [native verification contract](../../ARCHITECTURE.md#wire-deployment-and-verification).
