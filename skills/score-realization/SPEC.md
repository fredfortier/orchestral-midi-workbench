# Realization design rationale and evidence

Authoring-only rationale. [SKILL.md](SKILL.md) and its directly routed references
own musical interpretation, conversion, native placement and verification.

The design separates source meaning, executable rendition and observed host state
because valid files do not establish successful placement. Exact receiver bindings
also have a different owner from musical interpretation; keeping those handoffs
explicit avoids a loader silently reorchestrating the piece.

Earlier missing native-pipe observations left some surrounding UI actions
unqualified. Those are historical integration evidence, not an instruction for
runtime agents to infer a current outage. The native-import and keyboard
references own their current operating conditions.
