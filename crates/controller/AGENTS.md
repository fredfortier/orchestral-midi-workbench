# Controller

- Read the [controller contract](../../docs/architecture/controller.md) before changing commands, host operations, or data formats.
- Design around Bitwig's native project, track, clip, selection, and automation model. Use the same qualified address, authority, input conventions, and result shape wherever the native operation is the same. Make the supported command discoverable in CLI help.
- Keep one supported command, input form, and wire contract per operation. Command aliases and compatibility shims are prohibited. Callers, tests, examples, and production skills use the governing forms.
- Extend the owning controller operation when a capability is missing. Purpose-built operations own host reads and mutations within their scope.
