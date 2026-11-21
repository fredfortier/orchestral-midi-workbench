"""Compare retained runtime snapshots; this does not simulate or exercise Bitwig."""
import json
from pathlib import Path

root = Path(__file__).resolve().parent.parent / 'validation/editor-selection'
rows = [json.loads(line) for line in (root / 'replacement.jsonl').read_text().splitlines()]
replacement = next(row['result'] for row in rows if row['probe'] == 'replaceRange' and row['error'] is None)


def notes(snapshot):
    return {note['attributes']['managed_object_ID']: note
            for lane in snapshot['lanes'] for clip in lane['clips'] for note in clip['notes']}


def musical(note):
    return {key: note[key] for key in ['key', 'channel', 'onset', 'end', 'serialized']}


before, after = notes(replacement['before']), notes(replacement['after'])
removed = {int(identity) for identity in replacement['removedIds']}
created = {note['attributes']['managed_object_ID'] for note in replacement['created']}
assert set(after) == (set(before) - removed) | created
assert all(musical(before[identity]) == musical(after[identity]) for identity in set(before) - removed)
print(f'Replacement: {len(before)} → {len(after)} notes; {len(removed)} removed, {len(created)} created; '
      f'{len(before) - len(removed)} untouched notes identical')
coverage = [row['result'] for row in rows if row['probe'] == 'coverage' and row['error'] is None]
for phase, actual_snapshot, expected_snapshot in [
        ('Undo', coverage[-2], replacement['before']), ('Redo', coverage[-1], replacement['after'])]:
    # The independent reread observes the three selected lanes. The replacement
    # ledger above observes all four fixture lanes, including the unselected one.
    actual = notes(actual_snapshot)
    expected = notes({'lanes': expected_snapshot['lanes'][:3]})
    assert set(actual) == set(expected)
    assert all(musical(actual[identity]) == musical(expected[identity]) for identity in actual)
    print(f'{phase}: {len(actual)} notes match exact identity, row, timing and serialized content')

expression_rows = [json.loads(line) for line in (root / 'expression-channel.jsonl').read_text().splitlines()]
move = next(row['result'] for row in expression_rows if row['probe'] == 'channelOne' and row['error'] is None)
source, destination = move['beforeNote'], move['afterNote']
assert source['attributes']['managed_object_ID'] == destination['attributes']['managed_object_ID']
assert source['channel'] == 0 and destination['channel'] == 9
assert source['serialized'] == destination['serialized']
assert source['expressions'] == destination['expressions']
assert len(destination['expressions']['PRESSURE']) == 2
assert len(destination['expressions']['TIMBRE']) == 2
print('Channel move: identical note identity and serialized pressure/timbre points with spread')

context_rows = [json.loads(line) for line in (root / 'context.jsonl').read_text().splitlines()]
assert {row['result']['projectBeats'] for row in context_rows} == {16, 32, 64}
for row in context_rows:
    result = row['result']
    position = result['projectBeats']
    expected_seconds = min(position, 32) * 60 / 110 + max(0, position - 32) * 60 / 140
    assert abs(result['seconds'] - expected_seconds) < 1e-12
    assert result['tempoBpm'] == (110 if position < 32 else 140)
    assert result['numerator'] == (4 if position < 32 else 3)
    assert result['denominator'] == 4
    assert result['roundTripBeats'] == position
    assert result['playheadBefore'] == result['playheadAfter'] == 0
print('Context: 16/32/64-beat queries match stepped tempo/meter and independent elapsed time; playhead unchanged')

mapping_rows = [json.loads(line) for line in (root / 'clip-mapping.jsonl').read_text().splitlines()]
for row in mapping_rows[:2]:
    original, observed = notes(row['result']['before']), notes(row['result']['after'])
    assert set(original) == set(observed)
    assert all(musical(original[identity]) == musical(observed[identity]) for identity in original)
loop_clip = mapping_rows[1]['result']['after']['lanes'][3]['clips'][1]
assert loop_clip['mapping']['sourceStart'] == 0.375
assert loop_clip['mapping']['loopEnabled']
assert loop_clip['mapping']['loopRegion']['time'] == 0.25
assert loop_clip['mapping']['loopRegion']['duration'] == 2
samples = loop_clip['mapping']['samples']
assert [(sample['sourceBeats'], sample['loopIteration']) for sample in samples] == [
    (0.375, 0), (1.375, 0), (1.375, 2), (0.375, 32)]
occurrences = mapping_rows[2]['result']['occurrences']
visible = [item for item in occurrences if item['visibleEnd'] > item['visibleOnset']]
assert len(visible) == 7
assert visible[0]['projectOnset'] == 83.75 and visible[0]['visibleOnset'] == 84
assert visible[1]['projectOnset'] == 85.75 and visible[1]['visibleOnset'] == 85.875
print('Clip mapping: offset/loop changes retain source note payloads; seven visible occurrences have explicit clipped bounds')

boundary_rows = [json.loads(line) for line in (root / 'loop-boundary-replacement.jsonl').read_text().splitlines()]
initial = boundary_rows[0]['result']
independent_reads = [row['result'] for row in boundary_rows if row['probe'] == 'arrangerOccurrences']


def occurrence_identity(items):
    return sorted((item['clipId'], item['sourceId'], item['iteration'], item['visibleOnset'], item['visibleEnd'],
                   item['note']['key'], item['note']['channel'], item['note']['serialized']) for item in items)


assert occurrence_identity(independent_reads[0]) == occurrence_identity(initial['beforeOccurrences'])
assert occurrence_identity(independent_reads[1]) == occurrence_identity(initial['afterOccurrences'])
expressive = boundary_rows[-1]['result']


def material_outside(occurrences, document):
    owners = {str(clip['attributes']['managed_object_ID']): index
              for index, lane in enumerate(document['lanes']) for clip in lane['clips']}
    result = []
    for item in occurrences:
        onset, end = item['visibleOnset'], item['visibleEnd']
        for left, right in [(onset, min(end, 85)), (max(onset, 86), end)]:
            if right > left:
                result.append((owners[item['clipId']], left, right, item['note']['key'],
                               item['note']['channel'], item['note']['serialized']))
    return sorted(result)


assert material_outside(expressive['beforeOccurrences'], expressive['before']) == material_outside(
    expressive['afterOccurrences'], expressive['after'])
assert len(material_outside(expressive['beforeOccurrences'], expressive['before'])) == 37
assert any(len(item['note']['expressions']['PRESSURE']) == 2
           for item in expressive['afterOccurrences'] if item['visibleOnset'] == 86)
print('Loop boundary: native clip splitting preserves all 37 outside interval pieces by lane, timing and payload; Undo/Redo match')

grouped_rows = [json.loads(line) for line in (root / 'grouped-selection.jsonl').read_text().splitlines()]
selection = grouped_rows[0]['result']['views']['arranger']
assert selection['rangeBounds'] == [0, 92] and len(selection['lanes']) == 4
snapshot = grouped_rows[1]['result']
assert len(snapshot['lanes']) == 4
assert sum(len(lane['clips']) for lane in snapshot['lanes']) == 8
assert len(notes(snapshot)) == 34
assert any(clip['object']['path'].count('owned_main_tracks#') == 2
           for lane in snapshot['lanes'] for clip in lane['clips'])
print('Grouped selection: tool 2 returns four leaf lanes, including two nested tracks, with no duplicate group-overview clips')

closure_rows = [json.loads(line) for line in (root / 'staging-failure-closure.jsonl').read_text().splitlines()]
assert [row['probe'] for row in closure_rows] == ['stageRange', 'stageCloneClipRange', 'commitStaged', 'arrangerOccurrences']
assert closure_rows[1]['error'] is not None
assert closure_rows[2]['error'].endswith('No admitted staged fixture range')
assert closure_rows[0]['result']['beforeOccurrences'] == closure_rows[0]['result']['afterOccurrences'] == closure_rows[3]['result']
print('Staging failure: prior prepared edit is cleared; commit refuses; independent complete-source reread is unchanged')
