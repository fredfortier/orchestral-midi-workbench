# Native Bitwig sources

- [Bitwig's public Java extensions](https://github.com/bitwig/bitwig-extensions)
  describe SDK lifecycle. Build-specific document behavior needs local qualification.
- [Openwig's document dispatch](https://github.com/Axyom/openwig/blob/d9ffed4cd3ea7ad7f2f94dee5992545e8e54db2d/src/openwig/controller/openwig_bridge.control.js#L657-L675)
  informed deferred command completion. The Java harness owns this repository's probes.
- Original research, failed hypotheses and observation addresses are retained at
  `e0e0e1d:skills/.development/bitwig-internal/SOURCES.md`. Paths starting
  `crates/controller/validation/` refer to the retired repository's validation
  tree. The [controller decision record](../../../docs/adrs/controller-native-authority.md)
  records its revision and the later qualification provenance.

Use the [controller architecture](../../../docs/architecture/controller.md)
and [qualification runbook](../../../crates/controller/RUNBOOK.md) for current authority.
