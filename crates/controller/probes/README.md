# Retained isolated document research

`NotesDocumentProbe.java` preserves the source behind the material
[selection, CRUD, timeline and staging observations](../validation/editor-selection/README.md).
It is research reference code, excluded from both production and harness builds.
Its named experiments and constants are disposable fixture inputs, not supported
CLI operations. The JS console loader and grid controller that launched these
experiments are retired; original invocations remain alongside their evidence.

`build.sh` checks that the retained Java research source compiles in isolation:

```sh
NOTES_PROBE_JAVAC=/path/to/jdk/bin/javac bash crates/controller/probes/build.sh
python3 crates/controller/probes/verify-evidence.py
```

Current runnable native probes use the separately packaged Java harness and
explicit document arming in [RUNBOOK.md](../RUNBOOK.md). Before reusing an old
research operation, probe its owning native API in that harness and verify the
current build, thread, selection and Undo semantics. Old research success does
not admit production behavior or authorize replaying fixture constants.
