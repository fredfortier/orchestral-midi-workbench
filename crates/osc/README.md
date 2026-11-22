# DrivenByMoss OSC harness

This standalone CLI sends OSC messages and inspects UDP feedback with [rosc](https://docs.rs/rosc/0.11.4/rosc/).

In Bitwig, add the DrivenByMoss **Open Sound Control** controller and select
a MIDI input. For a local smoke test, set **Port to receive on** to `8000`,
**Host to send to** to `127.0.0.1`, and **Port to send to** to `9000`.
These are example ports. Restart the extension after changing the feedback
host or port. [DrivenByMoss OSC protocol](https://raw.githubusercontent.com/git-moss/DrivenByMoss-Documentation/refs/heads/master/Generic-Tools-Protocols/Open-Sound-Control-(OSC).md)

From `crates/`, run the listener first and send the refresh in a second terminal.

```sh
cargo run --locked -p osc -- listen 127.0.0.1:9000
cargo run --locked -p osc -- send 127.0.0.1:8000 /refresh
```

`/refresh` asks DrivenByMoss to resend its state. Look for feedback such as
`/project/name` and `/play`. The listener prints the sender and decoded OSC
message or bundle. Feedback goes to the controller's configured destination.
A successful send confirms local UDP submission. Feedback establishes that
the controller responded.

For a local check without a DAW, send `/project/name s:Loopback` to the
listener port `9000`. Send arguments use `i:` for a 32-bit integer, `f:`
for a 32-bit float, and `s:` for a string. Quote an argument containing spaces.

## MIDI Dependencies

Use ALSA’s `snd-virmidi` driver. I confirmed it’s available on this host and isn’t currently loaded. [Linux kernel documentation](https://www.kernel.org/doc/html/latest/sound/alsa-configuration.html#module-snd-virmidi)

1. Create one virtual MIDI port and verify that it appears.

   ```bash
   sudo modprobe snd-virmidi midi_devs=1
   aconnect -i
   ```

2. In **Bitwig → Settings → Controllers → Open Sound Control**, select the new **Virtual Raw MIDI** input and enable the existing controller. Choose the new port by its displayed name. The previously saved `Virtual Raw MIDI/3` is absent. DrivenByMoss uses this input to inject MIDI from OSC commands, so there’s no `aconnect` routing step for the OSC harness. [DrivenByMoss setup](https://raw.githubusercontent.com/git-moss/DrivenByMoss-Documentation/refs/heads/master/Generic-Tools-Protocols/Open-Sound-Control-(OSC).md)

3. Verify that Bitwig opened its configured OSC command port.

   ```bash
   ss -lunp 'sport = :8000'
   ```

Once `8000` is bound, listen on `9000` and send `/refresh` to `8000`.
