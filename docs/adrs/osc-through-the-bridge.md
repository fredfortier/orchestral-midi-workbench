# Run OSC alongside the native adapter service

Status: Accepted

## Context

Composition work alternates between scoped document operations and Bitwig
session actions. The research harness required an extra listener and logfile.
The OSC peer publishes asynchronous state and accepts generic addresses, while
native operations address exact objects and verify stored effects.

## Decision

Add an optional loopback UDP listener to the existing bridge process and use
one generic CLI to send addresses with typed arguments. Observe requested
feedback in a bounded time window through the same endpoint. Keep OSC outside
the native adapter command vocabulary and versioned wire.

Running a standalone listener retains a second operational lifecycle. Adding
a controller command per OSC action duplicates the extension's protocol and
misrepresents asynchronous feedback as native document authority.

## Consequences

The bridge can run without OSC. With OSC enabled, the configured feedback
port belongs to the bridge for its lifetime. There is no change to the Java
adapter or its wire version. Each action still needs an appropriate target
and independent effect check. UI interaction owns configuration and any
focus or modal behavior that lacks a qualified OSC path.

[Controller architecture](../architecture/controller.md) ·
[OSC setup](../../crates/osc/README.md)
