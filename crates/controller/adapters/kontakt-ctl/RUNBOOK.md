# Kontakt development runbook

## Provisioned Windows runtime

Start in this package directory with the hosted Wine prefix, display and
synchronization flags. Set `KONTAKT_RUNTIME` to the parent of an existing Windows
`.venv` with Python and `uv.exe`. Set `WINEPREFIX` and `DISPLAY` to the values for
the intended runtime. Keep the loader source, environment and `X:` library mapping
distinct.

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

- `/d` selects this checkout's image assets. `/wait /b` preserves exit status
  and JSON. Inherit the running Wine server's `WINEESYNC`/`WINEFSYNC` mode.
- The controller invokes `load-nki --use-running-window --headless`. Use the
  [preprocessing contract](ARCHITECTURE.md#standalone-library-preprocessing) for
  library maintenance and retain unknown outcomes for reconciliation.
