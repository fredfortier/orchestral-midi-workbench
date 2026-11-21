# Kontakt loader architecture

## Hosted loading

The [controller](../../ARCHITECTURE.md#integrated-instruments) owns Bitwig
assignment, routing, and the editor lifecycle. The loader submits the exact NKI
to the editor admitted by that transaction. It leaves player creation, patch
customization, and editor closure to their respective owners.

Run in the hosted Wine prefix with the matching display and synchronization
mode. The provisioned Windows environment, loader source and `X:` drive
mapping are distinct inputs. Preserve the running server and its display, DPI,
and encoding configuration during recovery.

Resolve current transaction-owned window handles after submission because the
child render window can be recreated. Retain submission and dialog evidence.
Reconcile an unknown outcome before another submission. Confirmation covers
the loader's observed dialog, render, and navigator state. Sample availability,
saved persistence, and audible behavior each require their own evidence.

## Process contract and verification

The Rust caller embeds the shipped source and assets in
[`kontakt_process.rs`](../../src/kontakt_process.rs). With `--json` before the
subcommand, stdout carries one `kontakt-ctl.result/v1` object and logs go to
stderr. Failures return a nonzero exit status. [`cli.py`](kontakt_ctl/cli.py) and
the command result types own the concrete fields.

Use ephemeral Windows checks and disposable load probes. Durable unit tests
are prohibited. Compilation verifies source only. The
[runtime guide](README.md#run-the-vendored-source-with-the-provisioned-windows-environment)
owns the runnable setup and [library preprocessing](SPEC.batch-resave.md) owns
standalone process scope and path-based receipt reuse.
