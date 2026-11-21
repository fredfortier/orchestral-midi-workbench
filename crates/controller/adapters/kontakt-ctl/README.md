# kontakt-ctl

Repo-owned Windows-under-Wine loader. Ordinary setup invokes it only through
integrated `midi-bridge controller instrument assign`; it loads the exact NKI into
the adapter-owned editor. It does not assign players, edit Bitwig or customize
receivers. [Architecture](ARCHITECTURE.md#hosted-loading) owns
that boundary.

## Run the vendored source with the provisioned Windows environment

Enter this package from the repository root as shown below. Keep the provisioned
Windows environment separate from the loader source. The wrapper selects this
checkout while reusing that environment under Wine.

Tested environment: Kontakt 7.10.1, x64 Windows Python 3.11, `/home/user/wine`,
`DISPLAY=:0`. `X:` maps to `/mnt/data/kontakt`. The existing Windows runtime is
`/home/user/src/oatsvine/music-composition-skills/tools/kontakt-ctl/.venv`.
Substitute another already provisioned Windows environment when appropriate.

```bash
cd crates/controller/adapters/kontakt-ctl  # from the repository root
export WINEPREFIX=/home/user/wine DISPLAY=:0
KONTAKT_RUNTIME=/home/user/src/oatsvine/music-composition-skills/tools/kontakt-ctl
KONTAKT_SOURCE_WIN=$(winepath -w "$PWD")
KONTAKT_VENV_WIN=$(winepath -w "$KONTAKT_RUNTIME/.venv")

kontakt_run() {
  env PYTHONPATH="$KONTAKT_SOURCE_WIN" UV_PROJECT_ENVIRONMENT="$KONTAKT_VENV_WIN" \
    wine start /wait /b /d "$KONTAKT_SOURCE_WIN" /unix \
    "$KONTAKT_RUNTIME/.venv/Scripts/uv.exe" run --no-sync "$@"
}
kontakt_ctl() { kontakt_run kontakt-ctl "$@"; }

kontakt_ctl --help
kontakt_ctl --json batch-resave 'Library Folder' --check
```

Match `WINEESYNC`/`WINEFSYNC` to the **running** Wine server. The tested standalone
maintenance server had neither variable enabled. When either is incorrectly
forced on, Wine can fail before Python starts. Do not kill a shared wineserver or
change the running DAW's synchronization mode. `/wait /b` preserves exit status
and JSON; `/d` is essential to resolving this checkout's image assets.

`batch-resave` accepts exactly one literal directory name directly beneath `X:`.
Absolute paths, nested paths, traversal, wildcards and filesystem redirects are
rejected. Use one library per invocation. No instrument files means an error.

## Loading results and failures

`load-nki --use-running-window --headless` is the controller's subprocess entry,
not a separate manual setup stage. Preserve the original exact path and transaction.
`load_confirmed` establishes the loader's dialog/render/navigator checks, not sample
completeness, saved persistence or audible behavior. Native editor closure belongs
to the controller after confirmed loading. Unknown loads require reconciliation;
a failed call does not prove no input was submitted.

Do not use legacy rack-doctor, private-setting or standalone-player operations to
recover ordinary assignment. Repair the owning integrated operation.

## Library maintenance and verification

[The preprocessing contract](SPEC.batch-resave.md) owns completion and path-based
reuse; initialization is separate from loading. Back up instrument files before
an authorized resave. `resave-all` is maintenance, not a setup prerequisite.
Missing images require checking source working directory; infrastructure failures
do not invalidate completed processing or authorize a repeat resave.

JSON uses `--json` before the command: one `kontakt-ctl.result/v1` object on stdout,
logs on stderr, nonzero exit on failure. Actual fields live in
[cli.py](kontakt_ctl/cli.py) and the command result types.

Durable unit tests are prohibited. Use the same Windows runtime/source environment
for compilation and ephemeral checks:

```sh
kontakt_run python -m compileall -q kontakt_ctl
```

Live mutation uses disposable instrument files with available samples. Retain
unknown process/dialog outcomes; never kill unrelated players or the shared Wine
server. An offline pass does not prove a native transaction completed.
