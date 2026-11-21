#!/usr/bin/env python3
"""Evaluate the finite WORKPAD predicate; retain every command and case outcome.

Run from any directory. Failed evidence is written with complete=false; this
script never edits code, changes assertions, or grants itself narrower scope.
"""
import datetime
import hashlib
import json
import pathlib
import re
import subprocess
import tempfile
import time

ROOT = pathlib.Path(__file__).resolve().parents[3]
CRATE = ROOT / 'crates/convert'
MANIFEST = CRATE / 'tests/corpus-sample.json'
CORPUS = ROOT / 'music21/music21/corpus'
OUTPUT = CRATE / 'tests/acceptance.json'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def artifact_hashes():
    artifacts = [ROOT / 'crates/Cargo.lock', ROOT / 'crates/Cargo.toml', CRATE / 'Cargo.toml', CRATE / 'README.md', ROOT / 'crates/research/check.mjs', ROOT / 'crates/research/sources.json']
    for folder in ['src', 'mapping', 'tests']:
        artifacts += [p for p in (CRATE / folder).rglob('*') if p.is_file()
                      and p != OUTPUT and '__pycache__' not in str(p)]
    return {str(p.relative_to(ROOT)): sha(p.read_bytes()) for p in sorted(artifacts)}


def main():
    before = artifact_hashes()
    logs = pathlib.Path(tempfile.mkdtemp(prefix='convert-acceptance-'))
    cargo = ['cargo']
    common = ['--manifest-path', 'crates/Cargo.toml']
    commands = [
        ['node', 'crates/research/check.mjs'],
        cargo + ['fmt'] + common + ['--all', '--check'],
        cargo + ['check'] + common + ['--workspace', '--locked', '--offline'],
        cargo + ['test'] + common + ['-p', 'musicxml-midi2-convert', '--locked', '--offline', '--', '--show-output'],
        cargo + ['clippy'] + common + ['--workspace', '--all-targets', '--locked', '--offline', '--', '-D', 'warnings'],
        cargo + ['doc'] + common + ['--workspace', '--no-deps', '--locked', '--offline'],
        ['cargo', '+1.87.0', 'check'] + common + ['-p', 'musicxml-midi2-convert', '--locked', '--offline'],
    ]
    evidence = []
    cases = []
    observations = {}
    for index, command in enumerate(commands):
        print('Checking:', ' '.join(command), flush=True)
        start = time.monotonic()
        result = subprocess.run(command, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=False)
        text = result.stdout.decode('utf-8', errors='replace')
        path = logs / f'{index}.log'
        path.write_bytes(result.stdout)
        evidence.append({'command': command, 'exitCode': result.returncode,
                         'seconds': round(time.monotonic() - start, 3),
                         'log': str(path), 'logSha256': sha(result.stdout)})
        if command[1] == 'test':
            harness = ''
            for line in text.splitlines():
                if line.startswith('CORPUS_OBSERVATION '):
                    item = json.loads(line.removeprefix('CORPUS_OBSERVATION '))
                    observations[item['path']] = item
                if 'Running tests/' in line:
                    harness = line.split('Running tests/')[1].split()[0]
                match = re.fullmatch(r'test (\S+) \.\.\. (ok|FAILED|ignored)', line)
                if match:
                    cases.append({'harness': harness, 'case': match[1], 'result': match[2]})
        print('  exit', result.returncode, f'({evidence[-1]["seconds"]}s)', flush=True)
    manifest = json.loads(MANIFEST.read_text())
    invalid = {}
    corpus_cases = {int(re.search(r'::case_(\d+)', item['case'])[1]): item
                    for item in cases if item['case'].startswith('corpus_roundtrip::case_')}
    inventory = []
    for index, item in enumerate(manifest['files'], 1):
        actual = sha((CORPUS / item['path']).read_bytes())
        inventory.append({**item, 'hashMatches': actual == item['sha256'],
                          'test': corpus_cases.get(index),
                          'disposition': 'invalid-input witness' if item['path'] in invalid else 'both inverse modes and XSD',
                          'diagnostic': invalid.get(item['path']), 'observed': observations.get(item['path'])})
    hashes = artifact_hashes()
    names = {item['case'] for item in cases if item['result'] == 'ok'}
    required = [
        'repeated_freehand_notation_keeps_provenance_without_duplicate_xml_ids',
        'a_piano_profile_is_not_lost_when_the_score_only_contains_a_rest',
        'ordinary_piano_declares_only_piano_and_emits_no_orchestral_attribute',
        'two_instruments_in_one_written_part_get_distinct_profile_bindings',
        'independent_bach_orchestral_note_and_ornament_counts',
        'corpus_sample_is_pinned_and_diversified', 'piano_ci_uses_its_published_id_and_31_byte_details',
        'musicxml_shake_is_an_ornament_schedule_not_automatically_a_brass_shake_sample',
        'scoped_other_ornament_remains_literal_and_does_not_invent_a_library_articulation',
        'voice_scoped_text_survives_without_a_companion_and_stays_in_the_correct_part',
        'all_named_coordinates_survive_both_directions', 'catalog_is_single_source',
        'independent_bwv267_facts', 'independent_note_wire_vector',
        'actual_binary_converts_bach_both_ways_and_schema_validates',
        'independently_authored_clip_has_a_real_inverse', 'editing_the_clip_defeats_stale_source_restoration',
        'source_pitch_replacement_is_explicit_and_rounding_is_accounted',
        'numbered_articulation_spans_are_half_open_and_voice_scoped',
        'xml_and_archive_boundaries_reject_independent_adversarial_inputs',
        'optional_capabilities_require_a_live_transaction_and_program_changes_invalidate_them',
        'invalidation_and_discovery_collisions_use_injected_entropy',
    ]
    complete = (hashes == before
                and all(item['exitCode'] == 0 for item in evidence)
                and len(corpus_cases) == len(inventory) == len(observations) == 12
                and all(item['hashMatches'] and item['test'] and item['test']['result'] == 'ok' and item['observed']['bothInverseXsdValid'] and item['observed']['bothInversePerformanceHashesMatch'] for item in inventory)
                and all(item['result'] == 'ok' for item in cases)
                and all(name in names for name in required)
                and sum(n.startswith('every_named_sound_remains_reachable_across_the_appendix_instrument_columns::') for n in names) == 9
                and sum(n.startswith('published_piano_dynamic_centers::') for n in names) == 10
                and sum(n.startswith('piano_inverse_uses_published_range_boundaries::') for n in names) == 22
                and sum(n.startswith('a_paired_tremolo_is_not_misread_as_two_single_note_rolls::') for n in names) == 4
                and sum(n.startswith('an_explicit_unmeasured_tremolo_is_not_given_an_invented_measured_rate::') for n in names) == 2
                and sum(n.startswith('profile_family_codecs::') for n in names) == 14
                and sum(n.startswith('release_subclasses_survive_canonical_musicxml::') for n in names) == 5
                and sum(n.startswith('berlin_conventions_survive::') for n in names) == 6)
    report = {
        'claim': 'T(mapping=2, policy=corpus-v1, sample=12-v1, M2-123 + M2-126, Cargo.lock)',
        'generatedUtc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'complete': complete, 'artifactUnchangedDuringChecks': hashes == before, 'artifactSha256': hashes,
        'artifactDigest': sha(json.dumps(hashes, sort_keys=True, separators=(',', ':')).encode()),
        'commands': evidence, 'tests': cases, 'corpus': inventory,
        'counts': {'discoveredHashedAttempted': len(inventory), 'validBothDirections': len(inventory)-len(invalid),
                   'invalidWitnesses': len(invalid), 'historicalSurveyFiles': manifest['surveyedFiles'], 'testCases': len(cases), 'namedCoordinates': 106,
                   'releaseSubclasses': 5, 'profileCodecCases': 14},
        'obligations': {
            'corpus': '12 diversified pinned hashes selected after surveying 535; all 12 per-file cases; both reconstructions compared and XSD validated',
            'mapping': '106 catalog coordinates; independently specified JSON; five release cases; field vectors',
            'granular': 'six Berlin refinement cases, hash-bound token dictionary, PE schema/chunks',
            'inverse': 'external clips, stale source, controller/note ordering and canonical re-encoding',
            'wire': 'independent byte vectors, truncation/reserved bits, scaling, header ordering',
            'piano': 'M2-126 ID and 31-byte details, ten literal dynamic centers and 22 inverse boundaries, key velocity and physical positions, natural decay and silent key actions',
            'interpretation': 'nine instrument columns × 106 named sounds; authored shortening counterexamples; unknown scoped remarks; harmonic pitch-role and ornament-start counterexamples',
            'profiles': '14 family cases plus discovery, state, details, timeout, invalidation, fallback and PE cases',
            'cli': 'process-level Bach conversion to all three formats; independent XSD and overwrite checks',
            'accounting': 'counted elements/attributes with dispositions; exact encoded magnitudes; rounding bound case',
            'reproducibility': 'this content-addressed report; offline Cargo, research, lint, docs, Rust 1.87 checks',
        },
        'numericBounds': {'time': 'exact or E_TPQ, no time rounding', 'nearestInteger': '<= 1/2 encoded unit',
                          'pitchEndpointClamp': '< 1 encoded 7.25 unit near 128', 'trace': 'exactMagnitude and encoded in conversion report'},
        'domainLimits': ['Finite corpus and named synthetic cases; not all possible MusicXML or receivers',
                         'No hardware/audio or installed sample-library certification',
                         'Accelerated/terminal ornaments, paired-note tremolo and numeric Fine schedules reject explicitly',
                         'Unresolved multi-instrument/note-instance bindings reject explicitly'],
    }
    OUTPUT.write_text(json.dumps(report, indent=2) + '\n')
    print(f'complete={complete}; {len(cases)} cases; {OUTPUT}', flush=True)
    return 0 if complete else 1


if __name__ == '__main__':
    raise SystemExit(main())
