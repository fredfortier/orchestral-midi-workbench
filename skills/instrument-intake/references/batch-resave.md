# Initialize a library once and reuse completed coverage

Read this before processing or claiming readiness. Intake owns standalone library
initialization through the repo-owned `controller/adapters/kontakt-ctl`; assignment
uses that loader only through the integrated midi-bridge operation.

```text
Resolve library + relative patch paths + existing processing records
  -> completed successful coverage: retain marker; skip resave
  -> missing coverage, no unresolved attempt: back up -> initialize once
       -> completed success + clean owned-process exit: record coverage; return
       -> failure/partial/unknown: retain original evidence; reconcile; no replay
```

## Identity and reuse

Use `INSTRUMENTS_ROOT_DIR`. Patches are identified only
by their relative NKI paths beneath it. Processing records establish completed
work for named library/patch paths and scope. Reuse completed coverage. Review
coverage after a known library replacement or addition of uncovered paths.

A documented successful folder-wide operation can establish coverage for patches
in that folder. A log's existence alone, an ERROR, an interrupted run, a playable
load or sample-resolution repair does not establish successful initialization.
Preserve prior failures alongside later outcomes; do not promote partial progress.

A later failed rerun does not by itself erase prior completed initialization.
Reconcile that attempt's outcome and retain both records. Reuse established
path-based coverage unless evidence identifies a resulting defect affecting the
requested patches. Do not run initialization again merely to obtain newer evidence.

## Execution prerequisites

Read local library instructions and existing INSTRUMENT.md. Resolve the existing Wine drive mapping for INSTRUMENTS_ROOT_DIR. The existing command accepts one top-level library
folder, not an NKI or nested subset. Announce that scope. Before any source write,
back up all `.nki`, `.nkm` and `.nkb` files in that folder outside the library and
verify the backup covers those relative paths. Preserve samples unchanged.

Read the [package README](../../../controller/adapters/kontakt-ctl/README.md) before
Wine execution. Establish runtime, prefix, display and synchronization settings
from the running environment before `winepath` or launching commands. Do not
try combinations, kill the shared wineserver or change the DAW's environment.
Infrastructure failure is not evidence of missing initialization.

`batch-resave --check` verifies completed processing coverage by relative
instrument paths. A check exits 0
for covered paths without launching Kontakt. Only a missing/uncovered processing
record permits the initialization branch; infrastructure errors do not.

Use the existing Windows runtime through Wine `uv.exe run
--no-sync kontakt-ctl --json batch-resave LIBRARY_FOLDER`, with the repo-owned
source and documented working-directory/environment setup. No Linux runtime or
alternative helper is permitted. Retain the literal invocation, stdout/stderr,
wall-clock/monotonic timing, backup location and actual exit result. The operation
must reuse completed path-based coverage and close its own standalone Kontakt
cleanly before reporting success. Never infer completion from a file count alone.

## Failure and return

On error, partial progress or unknown outcome, retain the original operation and
mark initialization unverified. Capture owned-process/dialog state only through
supported tool affordances. Do not submit again, kill an unknown process, restore
files while processing may still be live, or invent manual recovery. Diagnose and
repair the owning operation within scope. An unsupported reconciliation or safe
restoration step is an explicit tool capability gap, not a completed recovery.

After completed success, merge path-based processing status into INSTRUMENT.md:
record the library path, covered relative paths or folder scope, operation evidence,
completion time and clean process result. Reuse an existing successful record when
it already covers the scope. Return that marker, evidence and exact relative NKI
paths to the caller. Documentation coverage and processing coverage remain separate;
never imply that unreviewed patches have documented musical capabilities.
