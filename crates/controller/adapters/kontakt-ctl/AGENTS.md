# Repo-owned Kontakt loader

These instructions govern this package. Application boundaries are defined in
[controller architecture](../../ARCHITECTURE.md#operations-and-boundaries).

- Ordinary setup loads the specified NKI into the editor opened by integrated
  `midi-bridge controller instrument assign`. Do not create a player, manipulate
  Bitwig, customize patches or close the editor from this package.
- The runtime is Windows Python under Wine. Use the existing provisioned Windows
  `uv.exe run` environment; never create/sync a Linux virtualenv here. Runtime,
  source working directory and `X:` mapping are distinct; use [README.md](README.md).
- Match the hosted Wine prefix and synchronization mode. Do not kill a shared
  wineserver or change display/DPI/encoding to hide a loader failure.
- Resolve current transaction-owned HWNDs after loading; a child render window
  can be recreated. Retain submission/dialog evidence and do not replay unknown loads.
- Library preprocessing is a separate standalone operation. Its scope, completion
  and path-based reuse contract are in [SPEC.batch-resave.md](SPEC.batch-resave.md).
- JSON automation uses `--json` before the subcommand: one result object on stdout,
  logs on stderr, nonzero exit on failure. CLI/result definitions live in
  [cli.py](kontakt_ctl/cli.py) and the command result types, not a copied field list.
- Verify incrementally with ephemeral Windows invocations and disposable loading probes.
  Durable unit tests are prohibited by the project owner. Compilation is not load evidence.
