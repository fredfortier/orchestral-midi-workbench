# Syllable-builder design rationale

Authoring-only design and evidence. The runtime contract is [SKILL.md](SKILL.md)
and its four directly routed references; this document is not a runtime dependency.

## Why the workflow has separate owners

The design separates library facts, musical interpretation, stock receiver loading,
Kontakt dashboard configuration and native MIDI edits because those surfaces have
different evidence and failure modes. A vocabulary catalogue cannot prove cursor
behavior, and a dashboard cannot prove native note preservation. The retained
[owner decisions](sources/DECISIONS.md) explain the accepted division of work.
The runtime entrypoint and shared workflow express its operating consequences.

Finite sampled vocabularies motivated an explicit approximation stage rather
than arbitrary text-to-speech assumptions. Whole-syllable vowel passages offer a
musical alternative with different losses; they are not inferred from tempo alone.
The executable selection policy is in the phrase-planning reference.

## Why interval segmentation was chosen

The implemented optimizer models each reviewed gesture as a forward interval
graph. A consumed interval can carry any compatible configuration label whose cells begin with that interval. Selected configurations form a strict-prefix antichain; beat-zero single-cell constraints restrict the eligible labels, not merely the cuts. Binary x[e]
selects an occurrence and y[p] activates a configuration. Unit flow across each
gesture produces an ordered cover; x[e] ≤ y[label(e)] and
y[p] ≤ sum(x[e] with label p) charge each used configuration once.

The three sequential objectives model the accepted priorities: distinct
configurations, occurrences and programmed cells. A fixed solved optimum becomes
a constraint for the next phase. This explains the implementation, rather than
providing a second operating procedure. The script interface, planning boundaries
and interpretation of results live in the runtime phrase-planning reference.

The mathematical antecedent is Kumar, [Computing a Minimum Color Path in
Edge-Colored Graphs](https://sud03r.github.io/papers/sea-19.pdf), sections 1 and 4.
Musical boundaries, settings identity and channel budgets are local model choices,
not results of that paper. [CP-SAT documentation](https://developers.google.com/optimization/cp/cp_solver)
provides the solver-status semantics. [phrase_plan.py](scripts/phrase_plan.py)
contains the actual implementation; [offline checks](scripts/test_phrase_plan.py)
compare bounded instances against exhaustive partitions.

## Unimplemented design alternative

Selectable word-entry reuse was recorded as a possible future extension in the
owner decisions. A configured superstring could serve shorter contiguous fragments
through assigned word selectors, potentially reducing receiver count. The current
helper supports first-word prefixes of candidate configurations. It has no
nonfirst-entry or arbitrary superstring model. This proposal does not change runtime permissions.

The outstanding research questions are exact word/cell numbering, selector budget,
interior-fragment termination, HLS behavior, independent cursor demands and saved
assignment persistence. A future model could jointly select configured strings
and fragment intervals, adding selector cost to the existing receiver/cell costs.
No global optimum or live qualification for that richer model has been established.

## Source lineage and observed limits

[Strezov research](../../research/strezov/SOURCES.md) identifies installed and
public manual editions, vocabulary images and sequencing evidence. The MusicXML
[lyric definition](https://www.w3.org/2021/06/musicxml40/musicxml-reference/elements/lyric/)
explains the extractor's ordered-child representation; the existing
[companion model](../../convert/src/preserve.rs) supplies source associations.

[Execution findings](sources/ITERATION.md) retain the cell-entry, deletion,
selector, HLS, single-instrument and routing experiments that motivated runtime
procedures. The [rack loader](../../controller/adapters/kontakt-ctl/kontakt_ctl/rack.py)
and [native setup implementation](../../controller/client-adapter/src/main/java/com/midiworkbench/notes/internal/Instrument61.java)
are implementation evidence, not alternate runtime interfaces. Per-project
outcomes remain in project evidence directories.

The 2026-11-19 authoring repair removed duplicated operating clauses from this
file. Runtime instructions were inspected for coverage before removal. No
fresh-context model comparison was run; this is a placement and wording repair.


## Prefix reuse and direct execution repair — 2026-11-19

The prior exact-sequence label model selected both AB and A. The user requires
strict-prefix receivers to be eliminated, source-lyric slot names and a tight
plan-once execution path. Prefix-compatible edges now retain the existing
three objectives and exclude simultaneously selected strict prefixes. Independent
reconstruction checks coverage, cells, settings and single-cell entry constraints.
The exhaustive small-dictionary oracle and dedicated prefix/divergent-tail/
settings/entry tests exercise this change; no fresh-context model comparison is
claimed. Runtime references now distinguish source naming from sampled tokens
and give vowel, consonant, stress and melisma criteria, citing GALA Choruses’
Diction guidance and Friedrichs et al., ICPhS 2015 §§2–4. These sources support
interpretation priorities and register sensitivity, not the phonetics of a stock
sample label. The direct execution addition reuses completed evidence and scopes
replanning to invalidated decisions; required target and preservation checks remain.
