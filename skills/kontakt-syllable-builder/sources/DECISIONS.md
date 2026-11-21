# Owner decisions and evidence scope

Owner: repository user. Accepted scope: the skill-authoring conversation of
2026-11-18 and its explicit instruction to author and apply the workflow.

| Decision | User direction retained |
| --- | --- |
| Entry/delivery | Begin with an open Kontakt window and eligible patch; favor computer-use procedure, not kontakt-ctl development. |
| Portability | Use the current project as a case study; do not leak its specifications into reusable skills. |
| Intake | Route Syllabuilder instrument intake to this skill's reference for extracting/documenting each patch's syllables. |
| Lyrics | Establish the performance.companion.json convention and provide jq discovery/extraction; use XML-aware lyric interpretation. |
| Musical decomposition | Identify motifs, reusable atomic phrases and musically sound fixed-vowel use before realization. |
| Receiver model | Configure one phrase per named receiver slot, reused across compatible occurrences. One or more admitted note channels share its cursor and controls. |
| Optimization | Document business rules and a specific algorithm minimizing necessary distinct phrases. The user accepted the subsequent minimum-label/CP-SAT proposal with “ok”. |
| Repetition and entry | Repeated fragments may occur inside longer configurations to simplify performance. The implemented baseline starts at word one; do not assume random syllable-index access. Independently audit actual standalone suffix entries. |
| Selectable entries | Later approval: record word-keyswitch entry reuse as a possible optimization in SPEC.md. Qualify actual word boundaries, selectors, fragment termination and persistence before consolidation; the baseline solver does not implement it. |
| Naming and integration | Name slots from actual lyric syllables. Extend the existing integrated slot/assignment/kontakt-ctl load with an option to retain its active editor; do not add a parallel loading workflow. |
| Procedural qualification | Carefully probe computer-use manipulation and document a repeatable procedure. Consider a cross-Strezov kontakt-ctl interface only if the procedure is insufficient; prefer procedural realization. |
| Rack contents | Keep exactly one instrument in Kontakt Navigator; layering belongs to Bitwig. Establish the actual replacement action, including preference-dependent double-click, rather than append with global Load. |
| Editor disposition | Close the Kontakt window once its configuration is complete; keep the loaded receiver and saved configuration. |
| Slot selection | Explicitly select the instrument slot after creating it; qualified path targeting alone is not visible Bitwig slot selection. |
| Standard operations | User accepted container ensure; required-name slot ensure that reapplies All/Same, stock Filter/Map and complete note admission plus shared channel1, then selects the slot in controller code; independent instrument assignment preserving other devices. Instantiate stock devices and configure in code, with no shipped/custom presets. Articulated notes use channels2–16; nonarticulated single-channel parts may use1 for everything. Calling ensure reapplies defaults; omit it to preserve UI tweaks. |
| Completion | Fully author, implement the first case study, actively identify friction and tighten the skill. Ground design in SPEC.md; keep runtime prose operational. |

These records retain accepted local design decisions. SKILL.md and its routed
references are the runtime contract; SPEC.md records rationale and the unimplemented
entry-point proposal, not operating obligations.
External mechanics require their own evidence. Case-study outputs remain in the
project's existing evidence area; no live result is implied by this decision record.
