# Establish readiness before receiver writes

Read this before [assignment](assignment.md). PERFORMANCE.md supplies the exact
bindings; each patch is identified only by its NKI path relative to
`INSTRUMENTS_ROOT_DIR`. Check that path exists, matches the requested vendor/family,
has applicable library INSTRUMENT.md coverage and is covered by successful
initialization records.

Completed processing evidence identifies the library/relative paths, covered
scope and successful operation outcome. Reuse completed coverage. A failed, incomplete or
unknown operation does not establish success; a playable load alone does not
establish required preprocessing.

Missing documentation or processing coverage requires executing instrument-intake
as specified in SKILL.md, then returning to setup. Intake owns initialization and
reconciliation of prior failed attempts. Resolve all requested bindings before
receiver writes. Do not substitute vendors/patches, reinterpret failed processing
as success, repeat an unresolved operation or introduce an alternate loading path.

Return readiness or the precise failure for each requested binding.
