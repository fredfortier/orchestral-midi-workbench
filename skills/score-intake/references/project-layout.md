# Saved Bitwig project layout

## Bitwig project layout

- The project root is the directory containing the saved `.bwproject`. Keep `PERFORMANCE.md` there.
- Keep the original score and its provenance record in the project's established source location. For a new project without an agreed source location, use the project root.
- Use project-root `references/` for external listening references and `evidence/` for new evidence. Honor the project's existing evidence location.
- Preserve existing project files and paths. Create folders as needed and leave `recordings/`, `master-recordings/`, and other DAW-managed directories to Bitwig. Follow the [checkpoint procedure](#experimental-project-backups) for manual backups.

## Experimental project backups

Before an authorized experiment on an existing project, save the current project
through [session actions](../../bitwig-explore/references/session-actions.md#direct-actions)
and confirm completion. Resolve its actual saved `.bwproject` path with
`controller info`; do not copy an older on-disk state while edits remain unsaved.

Keep manual checkpoints in `backups/` beside the main project file:

```text
<project>/
  <name>.bwproject
  backups/
    <name>.20261118T214600Z.before-syllable-builder.bwproject
  auto-backups/
```

Use the actual project stem, current UTC timestamp and a short experiment label.
Copy only the saved `.bwproject` into this folder, as a single-file checkpoint
analogous to Bitwig's auto-backups. Do not copy the project directory, assets,
samples, exports or companion files. Leave Bitwig's `auto-backups/` untouched.
Choose a fresh filename, never overwrite an existing checkpoint, and verify the
copy is byte-identical to the saved main file before starting the experiment.
If saving or copying cannot be confirmed, stop dependent mutations.

Continue working in the open main `<name>.bwproject` at its original path; it is
the experimental fork. Do not use Save As to move work into the backup or create
a second project workspace. Record the checkpoint's relative path in the existing
project evidence. Further checkpoints follow the same convention.

This checkpoint restores project-file state only; external files remain shared.
Keep required external resources in place and preserve them under their owning
workflow. If rollback is requested, preserve the experimental main file as another
checkpoint. Restore the selected backup only when the main project is closed
and verify that the restored file matches the checkpoint byte for byte.
