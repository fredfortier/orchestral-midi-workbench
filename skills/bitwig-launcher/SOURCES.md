# Managed-host evidence and scope

The launcher is a Linux integration workflow, authored from the existing
[host CLI](../../../controller/src/cli_host.rs),
[native session contract](../bitwig-internal/references/service-contract.md) and
[managed launcher](../../../controller/scripts/managed-bitwig.py). It does not own
musical interpretation or private API admission.

The November 2026 investigation established these reusable distinctions:

- Bitwig controller TCP uses its published callback apparatus; the
  [retained transport evidence](../../../controller/validation/native-transport/README.md#actual-handler-and-thread-evidence)
  records controller-thread callbacks and document-thread dispatch. Connection
  readiness alone does not establish an active editor.
- The managed profile must have bounded browser roots. An observed desktop-profile
  clone exhausted the local 524288 inotify-watch limit; after setting isolated
  project/library roots and removing inherited sound/music locations, one restart
  used 700 watches. This is a bounded observation, not a universal watch budget.
  The [troubleshooting procedure](references/troubleshooting.md) requires measuring
  actual watches rather than interpreting this error as disk exhaustion.
- Saved native PipeWire links can override launch environment preferences. The
  [event-driven router](../../../controller/scripts/managed-bitwig.py) identifies the
  managed engine through its display/client properties and changes only its output
  links. A stopped one-second monitor recording verified stereo 48 kHz capture;
  it did not establish audible musical signal or repair reported crackling.
- X11 discovery must inspect Bitwig's instance string (`--classname`); the observed
  window had only one WM_CLASS string, so `--class` found no host and an earlier
  shutdown mistakenly returned success. The current close helper waits for the
  identified application process to exit after a WM_DELETE_WINDOW request.

An earlier audio field report localized a GTK chooser stall to unavailable GVFS
services and an unresponsive user manager. Its stack matched the reported
[systemd issue 44200](https://github.com/systemd/systemd/issues/44200); that match
was supporting evidence, not proof of a permanent upstream repair. A subsequent
reboot restored the chooser and the temporary desktop launcher wrapper was
removed. Do not prescribe the removed wrapper or a systemd upgrade as a verified
fix. Audible crackling remains unresolved by those observations. PipeWire
[clock configuration documentation](https://docs.pipewire.org/page_man_pipewire_1.html)
does not establish that changing sample rate cures an unidentified artifact.

Source logs, WAVs, full snapshots and screenshots stay with disposable experiments
or project evidence; they are not runtime skill assets. Production recognition
fixtures and contract regression tests have a different purpose and remain with
those implementations. No fresh-context model comparison was performed for this
skill: structural validation and exercised host behavior establish narrower facts.
