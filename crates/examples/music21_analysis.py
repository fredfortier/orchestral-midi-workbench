#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["music21==10.5.2"]
# ///
"""Audit an exported score before describing its harmony (music21 10.5.2).

Run with uv; --capture accepts the exact editorial assembly input, not live edit
authority. --reference optionally imports an earlier/source score for comparison.
Pitch counts expand Chords; attacks merge ties. Roman labels describe sampled
verticalities, not a human-approved harmonic reduction. No score is rewritten.

Usage: uv run examples/music21_analysis.py score.mxl --capture interpreted.json
       --output analysis.json [--reference source.mxl] [--check-offset 192]
The dependency is pinned to the latest PyPI release verified on 2026-11-19:
https://pypi.org/project/music21/10.5.2/

Sources: music21 User Guide chapters 9 (chordify), 15 (key analysis), 23 (Roman
numerals), 31 (ties): https://music21.org/music21docs/usersGuide/index.html
"""

import argparse
import copy
import csv
import hashlib
import json
import re
from collections import Counter, defaultdict
from pathlib import Path

import music21 as m21


def note_segments(part):
    """Expand chords and retain the actual containing voice when present."""
    for obj in part.recurse().notes:
        site = obj.activeSite
        voice = site.id if isinstance(site, m21.stream.Voice) else None
        onset = obj.getOffsetInHierarchy(part)
        release = m21.common.opFrac(onset + obj.quarterLength)
        atoms = obj.notes if isinstance(obj, m21.chord.Chord) else [obj]
        for atom in atoms:
            yield (
                onset,
                release,
                float(atom.pitch.ps),
                atom.tie.type if atom.tie else None,
                voice,
            )


def tied_attacks(segments):
    """Join contiguous sounding ties without conflating simultaneous voices."""
    pending = defaultdict(list)
    for onset, release, pitch, tie, voice in sorted(segments, key=lambda item: item[0]):
        start = onset
        if tie in ("stop", "continue"):
            candidates = pending[(onset, pitch)]
            match = next((i for i, (_, v) in enumerate(candidates) if v == voice), None)
            # A single-voice measure can lose its explicit Voice container on import.
            if match is None and len(candidates) == 1:
                if voice is None or candidates[0][1] is None:
                    match = 0
            if match is not None:
                start, _ = candidates.pop(match)
        if tie in ("start", "continue"):
            pending[(release, pitch)].append((start, voice))
        else:
            yield float(start), float(release), pitch
    for (release, pitch), candidates in pending.items():
        for start, _ in candidates:
            yield float(start), float(release), pitch


def summary(score):
    rows = []
    for part in score.parts:
        segments = list(note_segments(part))
        attacks = list(tied_attacks(segments))
        rows.append(
            {
                "part": part.partName or part.id,
                "measures": len(part.getElementsByClass(m21.stream.Measure)),
                "pitch_segments": len(segments),
                "attacks": len(attacks),
                "first_attack": min((n[0] for n in attacks), default=None),
                "last_release": max((n[1] for n in attacks), default=None),
                "pitch_range": [
                    min((n[2] for n in attacks), default=None),
                    max((n[2] for n in attacks), default=None),
                ],
                "pitch_beats": sum(float(n[1] - n[0]) for n in segments),
            }
        )
    return rows


def entities(entity):
    yield entity
    for child in entity["children"]:
        yield from entities(child)


def expected_capture(capture):
    """Interpret declared reset routing; expand placements, bounded by clip ends."""
    expected, resets, clipped = defaultdict(list), 0, []
    tracks = {
        e["metadata"]["id"]: e
        for e in entities(capture["entities"])
        if e["kind"] in {"track", "group"}
    }
    for clip in capture["midi"]["clips"]:
        if not clip["notes"]:
            continue
        track = tracks[clip["track"]]
        name = track["metadata"]["name"]
        reset_routes = set()
        for slot in entities(track):
            label = slot["metadata"].get("name", "")
            match = re.match(r"\[lyrics[^\]]*\breset=(\d+)[^\]]*\]", label)
            if not match:
                continue
            filters = [
                d["metadata"]["channelFilter"]
                for d in slot["children"]
                if "channelFilter" in d["metadata"]
            ]
            channels = (
                [
                    i
                    for i, enabled in enumerate(filters[0])
                    if enabled and (i != 0 or not any(filters[0][1:]))
                ]
                if filters
                else [slot["metadata"].get("channel", -1)]
            )
            reset_routes.update((c, int(match[1])) for c in channels)
        elapsed, source = 0.0, clip["sourceStart"]
        while elapsed < clip["duration"]:
            boundary = (
                clip["loopStart"] + clip["loopLength"]
                if clip["loopEnabled"]
                else clip["sourceEnd"]
            )
            length = min(boundary - source, clip["duration"] - elapsed)
            if length <= 0:
                raise ValueError("Nonpositive clip interval")
            for selected in clip["notes"]:
                n = selected["note"]
                if n["muted"] or not source <= n["start"] < source + length:
                    continue
                if (n["channel"], n["key"]) in reset_routes:
                    resets += 1
                    continue
                onset = clip["start"] + elapsed + n["start"] - source
                end = min(n["start"] + n["duration"], source + length)
                if end < n["start"] + n["duration"]:
                    clipped.append(
                        {
                            "part": name,
                            "pitch": n["key"],
                            "onset": onset,
                            "stored_duration": n["duration"],
                            "placement_release": clip["start"] + elapsed + end - source,
                        }
                    )
                expected[name].append(
                    (onset, clip["start"] + elapsed + end - source, n["key"])
                )
            elapsed += length
            if not clip["loopEnabled"]:
                break
            source = clip["loopStart"]
    return expected, resets, clipped


def audit_capture(score, path):
    capture = json.loads(path.read_text())
    expected, resets, clipped = expected_capture(capture)
    rows = []
    for part in score.parts:
        name = part.partName or part.id
        observed = tied_attacks(note_segments(part))
        # Match each pitch's successive occurrences; onset rounding cannot reorder
        # different pitches, unlike a global chronological zip at shared attacks.
        source_by_pitch, xml_by_pitch = defaultdict(list), defaultdict(list)
        for a, b, p in expected.pop(name, []):
            source_by_pitch[p].append((a, b))
        for a, b, p in observed:
            xml_by_pitch[p].append((a, b))
        counts_equal = {k: len(v) for k, v in source_by_pitch.items()} == {
            k: len(v) for k, v in xml_by_pitch.items()
        }
        errors = [
            abs(x - y)
            for p in source_by_pitch
            for src, xml in zip(sorted(source_by_pitch[p]), sorted(xml_by_pitch[p]))
            for x, y in zip(src, xml)
        ]
        maximum = max(errors, default=0)
        rows.append(
            {
                "part": name,
                "pitch_attack_counts_equal": counts_equal,
                "max_endpoint_error_beats": maximum,
                "within_declared_snap_tolerance": counts_equal
                and maximum <= 1 / 96 + 1 / 61440,
            }
        )
    return {
        "parts": rows,
        "missing_parts": list(expected),
        "declared_reset_commands": resets,
        "source_notes_clipped_at_placement_boundary": len(clipped),
        "clipped_tail_details": clipped,
        "passed": not expected
        and all(r["within_declared_snap_tolerance"] for r in rows),
    }


def compare_reference(score, reference, mappings):
    """Compare authored parts explicitly; do not equate a realized continuo with piano."""
    aliases = dict(item.split("=", 1) for item in mappings)
    rows = []
    for part in score.parts:
        name = part.partName or part.id
        target = aliases.get(name, name)
        matches = [p for p in reference.parts if p.partName == target]
        if not matches:
            continue
        if len(matches) != 1:
            raise ValueError(f"Ambiguous reference part: {target}")
        current_attacks = list(tied_attacks(note_segments(part)))
        original_attacks = list(tied_attacks(note_segments(matches[0])))
        current = sorted((p, a, b) for a, b, p in current_attacks)
        original = sorted((p, a, b) for a, b, p in original_attacks)
        pitches_equal = [n[0] for n in current] == [n[0] for n in original]
        differences = (
            [
                {
                    "pitch": x[0],
                    "export_onset": x[1],
                    "reference_onset": y[1],
                    "export_end": x[2],
                    "reference_end": y[2],
                }
                for x, y in zip(current, original)
                if x != y
            ]
            if pitches_equal
            else []
        )
        rows.append(
            {
                "part": name,
                "reference_part": target,
                "pitch_attack_counts_equal": pitches_equal,
                "same_pitch_sequence_by_onset": [
                    n[2] for n in sorted(current_attacks)
                ]
                == [n[2] for n in sorted(original_attacks)],
                "timing_differences": len(differences),
                "examples": differences[:5],
                "max_onset_difference": max(
                    (
                        abs(d["export_onset"] - d["reference_onset"])
                        for d in differences
                    ),
                    default=0,
                ),
                "max_end_difference": max(
                    (abs(d["export_end"] - d["reference_end"]) for d in differences),
                    default=0,
                ),
            }
        )
    return rows


def key_result(score):
    found = score.analyze("key")
    return {
        "key": str(found),
        "correlation": found.correlationCoefficient,
        "alternatives": [
            {"key": str(k), "correlation": k.correlationCoefficient}
            for k in found.alternateInterpretations[:3]
        ],
    }


def analyze(args):
    score = m21.converter.parse(args.score, forceSource=True)
    parts = summary(score)
    report = {
        "music21_version": m21.__version__,
        "input": str(args.score),
        "sha256": hashlib.sha256(args.score.read_bytes()).hexdigest(),
        "parts": parts,
        "total_pitch_segments": sum(p["pitch_segments"] for p in parts),
        "total_attacks": sum(p["attacks"] for p in parts),
        "duration_beats": float(score.highestTime),
        "key_estimate": key_result(score),
        "key_signatures": [
            str(k) for k in score.recurse().getElementsByClass(m21.key.KeySignature)
        ],
        "limitations": [
            "Spelling is exporter/editorial, not recovered source spelling.",
            "Pitch-beats count doubled voices; key correlation is not a probability.",
            "Roman labels are sampled sonorities including suspensions/passing notes, not proven harmonic functions.",
        ],
    }
    if args.capture:
        report["capture_audit"] = audit_capture(score, args.capture)
    if args.reference:
        reference = m21.converter.parse(args.reference, forceSource=True)
        report["reference"] = {
            "path": str(args.reference),
            "parts": summary(reference),
            "duration_beats": float(reference.highestTime),
            "key_estimate": key_result(reference),
            "part_comparison": compare_reference(score, reference, args.reference_part),
        }
    harmonic_score = m21.stream.Score()
    for part in score.parts:
        if part.partName not in args.exclude_harmony_part:
            harmonic_score.insert(0, copy.deepcopy(part))
    if not harmonic_score.parts:
        raise ValueError("No parts remain for harmony")
    report["harmonic_parts"] = [p.partName for p in harmonic_score.parts]
    report["structural_key_estimate"] = key_result(harmonic_score)
    tonic = m21.key.Key(args.key) if args.key else harmonic_score.analyze("key")
    report["roman_key"] = str(tonic)
    report["roman_key_source"] = (
        "explicit argument" if args.key else "global statistical estimate"
    )
    reduction = harmonic_score.chordify()
    slices = [
        (float(c.getOffsetInHierarchy(reduction)), c)
        for c in reduction.recurse().getElementsByClass(m21.chord.Chord)
    ]
    rows, figures = [], Counter()
    # One sample per quarter-note beat limits passing-note overcount. Keep raw and
    # key-context enharmonic labels side by side; never respell the imported score.
    for beat in range(int(score.highestTime) + 1):
        active = next(
            (
                c
                for start, c in slices
                if start <= beat < start + float(c.quarterLength)
            ),
            None,
        )
        if active is None:
            continue
        respelled = active.simplifyEnharmonics(keyContext=tonic)
        raw_rn = m21.roman.romanNumeralFromChord(active, tonic)
        rn = m21.roman.romanNumeralFromChord(respelled, tonic)
        figures[rn.figure] += 1
        rows.append(
            {
                "offset": beat,
                "measure": active.measureNumber,
                "pitches": " ".join(p.nameWithOctave for p in active.pitches),
                "bass": active.bass().nameWithOctave,
                "raw_roman": raw_rn.figure,
                "respelled_roman": rn.figure,
                "secondary_roman": m21.roman.romanNumeralFromChord(
                    respelled, tonic, preferSecondaryDominants=True
                ).figure,
                "triad_or_seventh": active.isTriad() or active.isSeventh(),
            }
        )
    report["quarter_beat_roman_sample_counts"] = dict(figures.most_common(15))
    report["simple_tertian_sample_fraction"] = sum(
        r["triad_or_seventh"] for r in rows
    ) / len(rows)
    report["enharmonic_label_changes"] = sum(
        r["raw_roman"] != r["respelled_roman"] for r in rows
    )
    report["landmarks"] = []
    for beat in args.check_offset:
        report["landmarks"].append(
            {
                "offset": beat,
                "voices": {
                    p.partName: [
                        pitch.nameWithOctave
                        for n in p.flatten().notes
                        if float(n.offset) <= beat < float(n.offset + n.quarterLength)
                        for pitch in n.pitches
                    ]
                    for p in score.parts
                },
                "roman_sample": next((r for r in rows if r["offset"] == beat), None),
            }
        )
    report["accuracy_warnings"] = []
    if not report["key_signatures"]:
        report["accuracy_warnings"].append(
            "No encoded key signature; an inferred key is analytical context, not source metadata."
        )
    if report.get("capture_audit", {}).get(
        "source_notes_clipped_at_placement_boundary"
    ):
        report["accuracy_warnings"].append(
            "Stored note tails cross placement boundaries and are clipped there. Do not equate stored durations or old performance notes with current exported extent."
        )
    if report["enharmonic_label_changes"]:
        report["accuracy_warnings"].append(
            "Key-context respelling changes Roman labels without changing pitches; neither spelling is authenticated by MIDI."
        )
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    with args.output.with_suffix(".roman.csv").open("w") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)
    print(
        json.dumps(
            {
                "version": m21.__version__,
                "key": report["key_estimate"],
                "capture_audit": report.get("capture_audit"),
                "output": str(args.output),
            },
            indent=2,
        )
    )
    return 0 if report.get("capture_audit", {"passed": True})["passed"] else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("score", type=Path)
    parser.add_argument("--capture", type=Path)
    parser.add_argument("--reference", type=Path)
    parser.add_argument(
        "--reference-part",
        action="append",
        default=[],
        help="Explicit exported-name=reference-name comparison",
    )
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--key", help="music21 key spelling, e.g. b for B minor")
    parser.add_argument("--exclude-harmony-part", action="append", default=[])
    parser.add_argument("--check-offset", type=float, action="append", default=[])
    return analyze(parser.parse_args())


if __name__ == "__main__":
    raise SystemExit(main())
