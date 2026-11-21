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
are prohibited. Compilation verifies source only.

## Standalone library preprocessing

```text
batch-resave literal X: child -> exact folder readback -> owned process rewrites
     covered receipt: reuse <----- clean exit + full verified inventory
```

- Accept one literal directory directly under the `X:` mapping. Reject traversal,
  redirects, wildcards and empty inventories. Serialize transactions, retain
  one standalone PID and verify the actual selected folder before submission.
  Only the retained standalone process may be closed or terminated by cleanup.
- Completion requires the original `.nki`/`.nkm`/`.nkb` inventory rewritten,
  nonempty exclusively readable outputs, cleared owned dialogs and process exit
  0. Timeout, unresolved samples, partial writes or forced cleanup fail.
  Reconcile uncertain processing before retry. Back up original instrument files
  for authorized processing. Disposable fixtures cannot certify a real library.
- Atomically publish `.kontakt-ctl-batch-resave.json` after clean completion.
  `kontakt-ctl.batch-resave/v3` records target folder, covered relative filenames,
  counts, completion time, exit code and evidence-log path. Reuse uses covered
  paths and a successful result. Compatible v2 coverage remains usable without
  its hashes. `--check` and already covered requests skip launch. Documented
  historical folder processing can establish reuse during intake.
- Verify path and process ownership, folder readback, completeness, cleanup
  and receipt reuse in ephemeral Windows checks. For live qualification,
  independently observe the exact target, complete rewrites and clean exit.
