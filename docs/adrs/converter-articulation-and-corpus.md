# Converter articulation and corpus review

Status: Accepted (recovered)

## Context

Recovered from the original converter reviews. The original migration snapshot is
`897be9d:crates/convert/`. The retired repository's `2f753a9`, `f166924`
and `c22958c` record the conversion, Piano Profile and adversarial review.

## Decision

Represent score intent as independent typed articulation facets and preserve
the exact performed values. Qualify the interpretation on a pinned corpus and
counterexamples. A direct sound lookup cannot preserve combinations and source
distinctions reliably.

## Consequences and evidence

- Reviewer counterexamples caught hand direction alone
  being treated as pizzicato, ornament start being lost, shake being read as
  brass production and early release being treated as a shorter sample.
- Standard Piano support remained separate from orchestral attributes.
  Rest-only Piano exposed the need to retain profile declarations independently
  of attacks. Repeated notation and lexical whitespace exposed provenance and
  remark-reconstruction errors.
- The finite calibration selected 12 scores from a 535-file survey. It covered
  timing, scope, wire vectors and both inverses against independent MusicXML XSD
  checks. Retained [acceptance evidence](../../crates/convert/tests/acceptance.json)
  describes its own artifact hashes and scope. It is evidence for that run.
  Paired native tremolo, undetermined ornament timing and unresolved bindings
  remain distinct from the named-vocabulary coverage.
- DawProject informed placement/source/point separation and music21 informed
  measured reconstruction order. Bounded binary/triplet inference, embedded
  musical identities, and a shared canonical serializer remain project choices.

[Current converter contract](../../crates/convert/ARCHITECTURE.md)
