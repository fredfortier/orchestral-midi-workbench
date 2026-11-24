# DrivenByMoss OSC

This crate supplies OSC encoding and decoding for the `midi-bridge` service.
The service owns the UDP feedback socket. It uses the existing bridge process
and loopback HTTP listener. The Bitwig native adapter protocol is independent.

## Configure the peer

In Bitwig, add or reuse DrivenByMoss **Open Sound Control**. Select an existing
virtual MIDI input and set **Port to receive on** to the command port, **Host to
send to** to `127.0.0.1`, and **Port to send to** to the feedback port. Restart
the extension after changing feedback host or port. The OSC listener is opt-in.
Start the usual controller service with the corresponding ports, for example:

```sh
midi-bridge serve --osc-port 8000 --osc-feedback-port 9000
```

Use the same controller and HTTP ports as before. With a nondefault bridge HTTP
port, pass `--port` to the CLI. A second process must not claim the feedback
port. See [action qualification](ACTIONS.md) for context-sensitive bindings.

## Send a message and observe feedback

```sh
midi-bridge osc /refresh --observe /project/name --observe /play
midi-bridge osc /time f:16
midi-bridge osc /refresh --observe /beat/str
midi-bridge osc /action/1
```

Use the addresses and value types in the [DrivenByMoss protocol](https://raw.githubusercontent.com/git-moss/DrivenByMoss-Documentation/refs/heads/master/Generic-Tools-Protocols/Open-Sound-Control-(OSC).md).
Arguments use `i:` (int32), `f:` (finite float32) or `s:` (string). Quote one
argument if it contains spaces. A bare address sends zero arguments.

The JSON receipt reports `submitted` after the local UDP send. Each requested
feedback address maps to the latest argument array received in the window,
which defaults to 400 ms. Use up to 32 exact addresses. `--wait-ms` accepts
1–2000 ms. A missing address means it was unobserved during that window.
Feedback is asynchronous and may describe an unrelated change. It does not
correlate with a command. Use `/refresh` for a fresh peer state report and
verify document or window effects independently. The service accepts one OSC
submission at a time and never resends a command after an uncertain outcome.
