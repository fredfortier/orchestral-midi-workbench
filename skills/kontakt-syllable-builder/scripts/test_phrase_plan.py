# /// script
# requires-python = ">=3.11"
# dependencies = ["ortools>=9.14,<10"]
# ///
"""Independent bounded checks for underlay preservation and dictionary optimality."""

import itertools
import math
import unittest

from phrase_plan import candidates, extract, review_solution, solve


def plan_for(strings, capacity=4, channels=None):
    return {
        "capacity": capacity,
        "available_channels": channels or list(range(1, 17)),
        "gestures": [
            {
                "id": str(i),
                "atoms": [{"sample": c} for c in text],
                "cuts": list(range(len(text) + 1)),
                "cut_reasons": {
                    str(j): "independent test attack" for j in range(1, len(text))
                },
                "settings": {"nki": "library/full.nki"},
            }
            for i, text in enumerate(strings)
        ],
    }


def brute_minimum(strings, capacity):
    # Enumerate dictionaries independently of the production graph and solver.
    words = sorted(
        {
            text[a:b]
            for text in strings
            for a in range(len(text))
            for b in range(a + 1, min(len(text), a + capacity) + 1)
        }
    )
    best = None
    for size in range(1, len(words) + 1):
        for dictionary in itertools.combinations(words, size):
            if any(a != b and b.startswith(a) for a in dictionary for b in dictionary):
                continue
            count = 0
            for text in strings:
                cost = [0] + [math.inf] * len(text)
                for end in range(1, len(text) + 1):
                    cost[end] = min(
                        (
                            cost[start] + 1
                            for start in range(end)
                            if any(
                                word.startswith(text[start:end]) for word in dictionary
                            )
                        ),
                        default=math.inf,
                    )
                count += cost[-1]
            if math.isfinite(count):
                objective = (size, count, sum(map(len, dictionary)))
                best = objective if best is None else min(best, objective)
        if best is not None:
            return best
    raise AssertionError("No dictionary covers input")


class PlanningTests(unittest.TestCase):
    def test_matches_exhaustive_partitions(self):
        for strings in (["AB", "BA", "ABA", "BAB"], ["ABC", "ABC"], ["AB", "BC", "AC"]):
            with self.subTest(strings=strings):
                result = solve(plan_for(strings, 3), 10)
                self.assertEqual(result["status"], "OPTIMAL")
                self.assertEqual(
                    tuple(p["objective"] for p in result["phases"]),
                    brute_minimum(strings, 3),
                )

    def test_required_boundary_not_crossed(self):
        plan = plan_for(["AB", "CD"], 4)
        for gesture in plan["gestures"]:
            gesture["cuts"] = [0, 2]
        result = solve(plan, 10)
        self.assertEqual(len(result["phrases"]), 2)
        self.assertEqual(len(result["occurrences"]), 2)

    def test_channel_exhaustion(self):
        result = solve(plan_for(["A", "B"], channels=[1]), 10)
        self.assertEqual(result["status"], "INFEASIBLE")
        self.assertNotIn("phrases", result)

    def test_standalone_suffix_requires_full_configuration(self):
        result = solve(plan_for(["AB", "B"]), 10)
        phrases = {p["id"]: p for p in result["phrases"]}
        suffix = next(o for o in result["occurrences"] if o["gesture"] == "1")
        self.assertEqual(suffix["start"], 0)
        self.assertEqual(phrases[suffix["phrase"]]["atoms"], [{"sample": "B"}])

    def test_shorter_prefix_uses_longer_receiver(self):
        result = solve(plan_for(["ABC", "AB", "A"]), 10)
        self.assertEqual(len(result["phrases"]), 1)
        self.assertEqual(result["phrases"][0]["atoms"], [{"sample": c} for c in "ABC"])
        self.assertEqual([o["end"] for o in result["occurrences"]], [3, 2, 1])

    def test_shared_opening_with_divergent_tail_is_not_a_prefix(self):
        plan = plan_for(["AB", "AC"])
        for gesture in plan["gestures"]:
            gesture["cuts"] = [0, 2]
        self.assertEqual(len(solve(plan, 10)["phrases"]), 2)

    def test_single_cell_entry_changes_segmentation_not_the_quality_bar(self):
        plan = plan_for(["A", "AB"])
        plan["gestures"][0]["single_cell_receiver"] = True
        result = solve(plan, 10)
        self.assertEqual(len(result["phrases"]), 2)
        self.assertEqual(
            {tuple(a["sample"] for a in p["atoms"]) for p in result["phrases"]},
            {("A",), ("B",)},
        )

    def test_cell_settings_prevent_false_prefix_match(self):
        plan = plan_for(["AB", "A"])
        plan["gestures"][1]["atoms"][0]["articulation"] = "staccato"
        result = solve(plan, 10)
        self.assertEqual(len(result["phrases"]), 2)

    def test_irreducibility_never_invents_a_cut(self):
        plan = plan_for(["A", "AB"])
        plan["gestures"][0]["single_cell_receiver"] = True
        plan["gestures"][1]["cuts"] = [0, 2]
        self.assertEqual(solve(plan, 10)["status"], "INFEASIBLE")

    def test_review_rejects_wrong_prefix_and_coverage(self):
        plan = plan_for(["ABC", "AB"])
        result = solve(plan, 10)
        result["occurrences"][1]["end"] = 1
        with self.assertRaises(ValueError):
            review_solution(plan, result)
        result = solve(plan, 10)
        result["phrases"].append(
            {
                "id": "redundant",
                "channel": 16,
                "atoms": [{"sample": "A"}],
                "settings": plan["gestures"][0]["settings"],
            }
        )
        with self.assertRaises(ValueError):
            review_solution(plan, result)
        result = solve(plan, 10)
        result["phrases"][0]["atoms"][0]["sample"] = "X"
        with self.assertRaises(ValueError):
            review_solution(plan, result)

    def test_settings_do_not_merge(self):
        plan = plan_for(["A", "A"])
        plan["gestures"][1]["settings"]["nki"] = "other/full.nki"
        self.assertEqual(len(solve(plan, 10)["phrases"]), 2)

    def test_unjustified_cut_refused(self):
        plan = plan_for(["AB"])
        plan["gestures"][0]["cut_reasons"] = {}
        with self.assertRaises(ValueError):
            candidates(plan)

    def test_lyrics_and_occurrences_preserved(self):
        source = "/score-partwise/part[@id='P1']/measure[1]/note[1]"
        associations = [
            {"source": source + "#occurrence=0", "onset": [0, 1]},
            {"source": source + "#occurrence=2", "onset": [6, 1]},
        ]
        result = extract(
            {
                "source_xml": '<score-partwise><part id="P1"><measure number="0">'
                '<note><tie type="start"/><lyric number="2"><syllabic>begin</syllabic>'
                "<text>a</text><elision> </elision><syllabic>end</syllabic><text>e</text>"
                '<extend type="start"/></lyric></note></measure></part></score-partwise>',
                "note_sources": associations,
            }
        )
        note = result["parts"][0]["notes"][0]
        self.assertEqual(note["measure"], "0")
        self.assertEqual(note["performed"], associations)
        self.assertEqual(
            [c["tag"] for c in note["lyrics"][0]["children"]],
            ["syllabic", "text", "elision", "syllabic", "text", "extend"],
        )


if __name__ == "__main__":
    unittest.main()
