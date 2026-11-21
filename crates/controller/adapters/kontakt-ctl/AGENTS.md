# Kontakt loader

- Follow the [loader contract](ARCHITECTURE.md) for hosted loads and the [preprocessing contract](SPEC.batch-resave.md) for standalone batch resave.
- Use the provisioned Windows Python and `uv.exe run --no-sync` under Wine. Keep virtualenv setup on the Windows side. Follow the [runtime setup](README.md#run-the-vendored-source-with-the-provisioned-windows-environment) for the checkout, hosted prefix, display, and synchronization mode. Keep the shared Wine server running.
- From this directory, with the documented wrapper configured, run `kontakt_run python -m compileall -q kontakt_ctl`. Use ephemeral Windows checks and disposable instruments for live verification. Preserve evidence when a submitted load has an unknown outcome.
- When changing the loader's shipped files, check the [controller's embedded file list](../../src/kontakt_process.rs). Check the Rust caller as well when changing the JSON result.
