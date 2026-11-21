# Kontakt loader

- Follow the [loader contract](ARCHITECTURE.md) for hosted loads and the [preprocessing contract](ARCHITECTURE.md#standalone-library-preprocessing) for standalone batch resave.
- When changing the loader's shipped files, check the [controller's embedded file list](../../src/kontakt_process.rs). Check the Rust caller as well when changing the JSON result.

## Provisioned Windows runtime

Use the existing Windows Python and `uv.exe` in the hosted Wine prefix. From
this directory, set `KONTAKT_RUNTIME` to the parent of the provisioned Windows
`.venv`. Set `WINEPREFIX` and `DISPLAY` to the hosted runtime and inherit its
`WINEESYNC`/`WINEFSYNC` mode. Keep the shared Wine server running and keep the
loader source, virtualenv and `X:` library mapping distinct.

```bash
KONTAKT_SOURCE_WIN=$(winepath -w "$PWD")
KONTAKT_VENV_WIN=$(winepath -w "$KONTAKT_RUNTIME/.venv")

kontakt_run() {
  env PYTHONPATH="$KONTAKT_SOURCE_WIN" UV_PROJECT_ENVIRONMENT="$KONTAKT_VENV_WIN" \
    wine start /wait /b /d "$KONTAKT_SOURCE_WIN" /unix \
    "$KONTAKT_RUNTIME/.venv/Scripts/uv.exe" run --no-sync "$@"
}
kontakt_run python -m compileall -q kontakt_ctl
kontakt_run kontakt-ctl --help
```

`/d` selects this checkout's assets. `/wait /b` preserves exit status and JSON.
Use ephemeral Windows checks and disposable instruments for live verification.
The controller invokes `load-nki --use-running-window --headless`. Retain
submission evidence and reconcile unknown outcomes before retrying.
