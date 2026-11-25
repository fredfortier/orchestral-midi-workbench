# Kontakt loader

Loads the requested NKI into the editor owned by the controller transaction.
See the [runtime instructions](AGENTS.md#provisioned-windows-runtime) and [architecture](../../../../docs/architecture/kontakt-loader.md).

The bundled `assets/targets/` crops come from the Kontakt 7 interface. The
loader uses the Instrument Navigator and Remove Instrument controls to verify
the rack during replacement. The File menu, Batch Resave menu, and folder
button targets support the menu and standalone preprocessing paths.
