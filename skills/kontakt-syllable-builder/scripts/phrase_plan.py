#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["ortools>=9.14,<10"]
# ///
"""Read preserved underlay; optimize reviewed phrase intervals. Never controls a host."""

import argparse
import json
import math
import sys
import time
import xml.etree.ElementTree as ET
from pathlib import Path


def extract(companion):
    xml = companion.get("source_xml")
    associations = companion.get("note_sources")
    if not isinstance(xml, str) or not xml or not isinstance(associations, list):
        raise ValueError("Expected nonempty source_xml and note_sources array")
    if len(xml) > 64_000_000 or "<!ENTITY" in xml.upper():
        raise ValueError("Oversized XML or entity declarations are unsupported")
    root = ET.fromstring(xml)
    if root.tag != "score-partwise":
        raise ValueError("Expected unnamespaced score-partwise MusicXML")
    by_source = {}
    for association in associations:
        source = association["source"].split("#occurrence=", 1)[0]
        by_source.setdefault(source, []).append(association)
    names = {
        part.attrib["id"]: part.findtext("part-name")
        for part in root.findall("part-list/score-part")
    }
    parts = []
    for part in root.findall("part"):
        notes = []
        part_id = part.attrib["id"]
        for measure_index, measure in enumerate(part.findall("measure"), 1):
            for note_index, note in enumerate(measure.findall("note"), 1):
                source = (
                    f"/score-partwise/part[@id='{part_id}']"
                    f"/measure[{measure_index}]/note[{note_index}]"
                )
                lyrics = [
                    {
                        "attributes": lyric.attrib,
                        "children": [
                            {
                                "tag": child.tag,
                                "text": child.text or "",
                                "attributes": child.attrib,
                            }
                            for child in lyric
                        ],
                    }
                    for lyric in note.findall("lyric")
                ]
                notes.append(
                    {
                        "source": source,
                        "measure": measure.attrib.get("number"),
                        "voice": note.findtext("voice"),
                        "staff": note.findtext("staff"),
                        "rest": note.find("rest") is not None,
                        "pitch": ET.tostring(note.find("pitch"), encoding="unicode")
                        if note.find("pitch") is not None
                        else None,
                        "lyrics": lyrics,
                        "ties": [tie.attrib for tie in note.findall("tie")],
                        "slurs": [
                            slur.attrib for slur in note.findall("notations/slur")
                        ],
                        "performed": by_source.get(source, []),
                    }
                )
        parts.append({"id": part_id, "name": names.get(part_id), "notes": notes})
    return {"kind": "preserved-underlay", "parts": parts}


def candidates(plan):
    capacity = plan.get("capacity")
    channels = plan.get("available_channels")
    gestures = plan.get("gestures")
    if type(capacity) is not int or not 1 <= capacity <= 4096:
        raise ValueError("capacity must be a verified positive integer <=4096")
    if (
        not isinstance(channels, list)
        or not channels
        or any(type(c) is not int or not 1 <= c <= 16 for c in channels)
        or len(set(channels)) != len(channels)
    ):
        raise ValueError("available_channels must contain unique CLI channels 1..16")
    if not isinstance(gestures, list) or not gestures or len(gestures) > 10000:
        raise ValueError("Expected 1..10000 reviewed gestures")
    identities, edges, ids = {}, [], set()
    for gesture in gestures:
        name, atoms, cuts = gesture["id"], gesture["atoms"], gesture["cuts"]
        settings = gesture["settings"]
        if type(gesture.get("single_cell_receiver", False)) is not bool:
            raise ValueError(f"{name}: single_cell_receiver must be boolean")
        if not isinstance(name, str) or not name or name in ids:
            raise ValueError("Gesture IDs must be unique nonempty strings")
        ids.add(name)
        if (
            not isinstance(atoms, list)
            or not atoms
            or len(atoms) > 4096
            or any(not isinstance(a, dict) or not a.get("sample") for a in atoms)
        ):
            raise ValueError(f"{name}: expected 1..4096 sampled-syllable objects")
        if (
            not isinstance(settings, dict)
            or not isinstance(settings.get("nki"), str)
            or not settings["nki"]
            or Path(settings["nki"]).is_absolute()
            or ".." in Path(settings["nki"]).parts
        ):
            raise ValueError(f"{name}: settings must identify the exact relative NKI")
        if (
            not isinstance(cuts, list)
            or not cuts
            or any(type(c) is not int for c in cuts)
            or cuts != sorted(set(cuts))
            or cuts[0] != 0
            or cuts[-1] != len(atoms)
        ):
            raise ValueError(
                f"{name}: cuts must be sorted unique boundaries including 0 and atom count"
            )
        reasons = gesture.get("cut_reasons", {})
        if any(not reasons.get(str(c)) for c in cuts[1:-1]):
            raise ValueError(f"{name}: each internal cut needs a musical reason")
        for i in cuts[:-1]:
            for j in cuts:
                if not i < j <= i + capacity:
                    continue
                definition = {"settings": settings, "atoms": atoms[i:j]}
                key = json.dumps(
                    definition, sort_keys=True, separators=(",", ":"), allow_nan=False
                )
                identities[key] = definition
                edges.append({"gesture": name, "start": i, "end": j, "key": key})
                if len(edges) > 100000:
                    raise ValueError(
                        "More than 100000 candidates; narrow reviewed scope explicitly"
                    )
    ordered = sorted(identities)
    labels = {key: index for index, key in enumerate(ordered)}
    definitions = [identities[key] for key in ordered]
    compatible = prefix_labels(definitions)
    single_cell = {g["id"] for g in gestures if g.get("single_cell_receiver", False)}
    expanded = []
    for edge in edges:
        required = labels[edge.pop("key")]
        for label in compatible[required]:
            if (
                edge["gesture"] not in single_cell
                or len(definitions[label]["atoms"]) == 1
            ):
                expanded.append({**edge, "label": label})
                if len(expanded) > 100000:
                    raise ValueError(
                        "More than 100000 prefix candidates; narrow reviewed scope"
                    )
    return definitions, expanded


def prefix_labels(definitions):
    """Index exact atom/settings prefixes; shared opening sounds alone do not match."""

    def encode(value):
        return json.dumps(value, sort_keys=True, allow_nan=False)

    keys = [
        (encode(d["settings"]), tuple(encode(a) for a in d["atoms"]))
        for d in definitions
    ]
    labels = {key: i for i, key in enumerate(keys)}
    compatible = [[] for _ in definitions]
    for longer, (settings, atoms) in enumerate(keys):
        for length in range(1, len(atoms) + 1):
            shorter = labels.get((settings, atoms[:length]))
            if shorter is not None:
                compatible[shorter].append(longer)
    return compatible


def review_solution(plan, result):
    """Reconstruct consumed prefixes and reject gaps, wrong cells or redundant banks."""
    phrases = {p["id"]: p for p in result["phrases"]}
    for short in phrases.values():
        for long in phrases.values():
            if (
                short["settings"] == long["settings"]
                and len(short["atoms"]) < len(long["atoms"])
                and short["atoms"] == long["atoms"][: len(short["atoms"])]
            ):
                raise ValueError(
                    "Selected dictionary contains a redundant strict prefix"
                )
    covered = 0
    for gesture in plan["gestures"]:
        end = 0
        occurrences = sorted(
            (o for o in result["occurrences"] if o["gesture"] == gesture["id"]),
            key=lambda o: o["start"],
        )
        for occurrence in occurrences:
            phrase = phrases[occurrence["phrase"]]
            start, stop = occurrence["start"], occurrence["end"]
            if (
                start != end
                or start not in gesture["cuts"]
                or stop not in gesture["cuts"]
                or stop <= start
                or phrase["settings"] != gesture["settings"]
                or gesture["atoms"][start:stop] != phrase["atoms"][: stop - start]
                or occurrence["channel"] != phrase["channel"]
                or (
                    gesture.get("single_cell_receiver", False)
                    and len(phrase["atoms"]) != 1
                )
            ):
                raise ValueError(
                    "Selected occurrence does not reconstruct the reviewed gesture"
                )
            end = stop
        if end != len(gesture["atoms"]):
            raise ValueError("Selected occurrences leave an uncovered gesture")
        covered += end
    return {"prefix_free": True, "covered_syllables": covered}


def solve(plan, seconds=60):
    from ortools import __version__
    from ortools.sat.python import cp_model

    if not math.isfinite(seconds) or seconds <= 0:
        raise ValueError("seconds must be positive and finite")
    definitions, edges = candidates(plan)
    model = cp_model.CpModel()
    use = [model.new_bool_var(f"phrase_{i}") for i in range(len(definitions))]
    # A strict prefix receiver is redundant unless a placement needs one cell.
    # In that case segmentation must avoid activating its longer counterpart.
    for i, compatible in enumerate(prefix_labels(definitions)):
        for j in compatible:
            if i != j:
                model.add(use[i] + use[j] <= 1)
    take = [model.new_bool_var(f"occurrence_{i}") for i in range(len(edges))]
    for gesture in plan["gestures"]:
        for boundary in gesture["cuts"]:
            outgoing = [
                take[i]
                for i, e in enumerate(edges)
                if e["gesture"] == gesture["id"] and e["start"] == boundary
            ]
            incoming = [
                take[i]
                for i, e in enumerate(edges)
                if e["gesture"] == gesture["id"] and e["end"] == boundary
            ]
            flow = (
                1 if boundary == 0 else -1 if boundary == len(gesture["atoms"]) else 0
            )
            model.add(sum(outgoing) - sum(incoming) == flow)
    for i, edge in enumerate(edges):
        model.add(take[i] <= use[edge["label"]])
    for label, active in enumerate(use):
        model.add(
            active <= sum(take[i] for i, e in enumerate(edges) if e["label"] == label)
        )
    model.add(sum(use) <= len(plan["available_channels"]))
    objectives = [
        sum(use),
        sum(take),
        sum(len(d["atoms"]) * use[i] for i, d in enumerate(definitions)),
    ]
    started = time.monotonic()
    phases, chosen, primary_status, bound = [], None, None, None
    for objective in objectives:
        remaining = seconds - (time.monotonic() - started)
        if remaining <= 0:
            break
        model.minimize(objective)
        solver = cp_model.CpSolver()
        solver.parameters.max_time_in_seconds = remaining
        solver.parameters.num_search_workers = 1
        solver.parameters.random_seed = 0
        status = solver.solve(model)
        status_name = solver.status_name(status)
        phase = {"status": status_name}
        if primary_status is None:
            primary_status = status_name
            bound = solver.best_objective_bound
        if status not in (cp_model.OPTIMAL, cp_model.FEASIBLE):
            phases.append(phase)
            break
        chosen = [i for i, variable in enumerate(take) if solver.value(variable)]
        value = round(solver.objective_value)
        phase["objective"] = value
        phases.append(phase)
        if status != cp_model.OPTIMAL:
            break
        model.add(objective == value)
    result = {
        "kind": "phrase-dictionary-plan",
        "reuse": "first-word-prefix",
        "solver": f"OR-Tools {__version__}",
        "status": "OPTIMAL"
        if len(phases) == 3 and all(p["status"] == "OPTIMAL" for p in phases)
        else "FEASIBLE"
        if chosen is not None
        else primary_status,
        "primary_status": primary_status,
        "primary_lower_bound": bound,
        "phases": phases,
        "candidate_count": len(edges),
        "configuration_candidates": len(definitions),
        "seconds_limit": seconds,
        "elapsed_seconds": time.monotonic() - started,
    }
    if chosen is not None:
        selected_labels = sorted({edges[i]["label"] for i in chosen})
        channels = dict(zip(selected_labels, sorted(plan["available_channels"])))
        result["phrases"] = [
            {"id": f"p{label:04}", "channel": channels[label], **definitions[label]}
            for label in selected_labels
        ]
        result["occurrences"] = [
            {
                **edges[i],
                "phrase": f"p{edges[i]['label']:04}",
                "channel": channels[edges[i]["label"]],
            }
            for i in chosen
        ]
        result["review"] = review_solution(plan, result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    extract_parser = commands.add_parser("extract")
    extract_parser.add_argument("--companion", required=True, type=Path)
    solve_parser = commands.add_parser("solve")
    solve_parser.add_argument("--input", required=True, type=Path)
    solve_parser.add_argument("--seconds", type=float, default=60)
    for command in (extract_parser, solve_parser):
        command.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    try:
        if args.output.exists():
            raise ValueError("Output exists; use a fresh evidence filename")
        source = args.companion if args.command == "extract" else args.input
        data = json.loads(source.read_text())
        result = (
            extract(data) if args.command == "extract" else solve(data, args.seconds)
        )
        with args.output.open("x") as stream:
            json.dump(result, stream, indent=2, allow_nan=False)
            stream.write("\n")
        print(
            json.dumps(
                {
                    "output": str(args.output),
                    "status": result.get("status", "EXTRACTED"),
                }
            )
        )
        return 0 if args.command == "extract" or "phrases" in result else 2
    except (
        ValueError,
        KeyError,
        TypeError,
        IndexError,
        OSError,
        ET.ParseError,
    ) as error:
        print(f"phrase_plan: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
