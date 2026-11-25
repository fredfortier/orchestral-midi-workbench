//! M2-123 Tables 8–15, with explicit native rules and private residual facets.
use crate::{Diagnostic, Result, model::Descriptor};
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Vocabulary {
    pub at: u8,
    pub sub: u8,
    pub id: &'static str,
    pub term: &'static str,
    pub name: &'static str,
    pub rule: &'static str,
}
pub const PROFILE_ID: [u8; 5] = [0x7e, 0x21, 1, 1, 1];
macro_rules! vocabulary {
    ($($at:literal / $sub:literal => $id:literal, $term:literal, $name:literal, $rule:literal;)*) => {
        pub const VOCABULARY: &[Vocabulary] = &[$(Vocabulary {at:$at,sub:$sub,id:$id,term:$term,name:$name,rule:$rule}),*];
    };
}
vocabulary! {
    0x10 / 0x0 => "core.normal", "ordinary", "Normal, first bank of variations", "N0";
    0x10 / 0x1 => "core.normal-extra", "ordinary additional variations", "Normal, second bank of variations", "T";
    0x10 / 0x2 => "core.legato", "legato", "Legato / slurred legato", "N1";
    0x10 / 0x3 => "core.molto-legato", "molto legato", "Molto legato", "T";
    0x10 / 0x4 => "core.glissando", "portamento", "Glissando / string portamento sound", "N2";
    0x10 / 0x5 => "core.detache", "detaché", "Detaché", "T";
    0x10 / 0x6 => "core.marcato", "marcato", "Marcato sustain", "N3";
    0x10 / 0x7 => "core.martele", "martelé", "Martelé", "T";
    0x10 / 0x8 => "core.senza-vibrato", "senza vibrato", "Explicitly without vibrato", "T";
    0x10 / 0x9 => "core.con-vibrato", "con vibrato", "Explicitly with vibrato", "T";
    0x10 / 0xa => "core.synchronized-vibrato", "synchronized vibrato", "Synchronized vibrato", "T";
    0x11 / 0x0 => "short.staccato-off", "staccato off string", "Ordinary staccato, off-string for strings", "N4";
    0x11 / 0x1 => "short.staccato-on", "staccato on string", "Ordinary staccato, on-string", "T";
    0x11 / 0x2 => "short.slurred-staccato", "slurred staccato", "Single-bow slurred staccato", "N5";
    0x11 / 0x3 => "short.accented-staccato", "accented staccato", "Accented staccato", "N6";
    0x11 / 0x4 => "short.staccatissimo", "staccatissimo", "Staccatissimo / deliberately damped short", "N7";
    0x11 / 0x5 => "short.spiccato", "spiccato", "Spiccato", "N8";
    0x11 / 0x6 => "short.sautille", "sautillé", "Sautillé", "T";
    0x11 / 0x7 => "short.martellato", "martellato", "Martellato", "T";
    0x11 / 0x8 => "short.long-staccato", "long staccato", "Long / mezzo staccato, louré", "T";
    0x11 / 0x9 => "short.portato", "portato", "Portato", "N9";
    0x11 / 0xa => "short.pizzicato", "pizzicato", "Pizzicato", "N10";
    0x11 / 0xb => "short.snap-pizzicato", "snap pizzicato", "Snap / Bartók pizzicato", "N11";
    0x11 / 0xc => "short.col-legno-battuto", "col legno battuto", "Col legno battuto", "T";
    0x11 / 0xd => "short.col-legno-gestrichen", "col legno gestrichen", "Col legno gestrichen", "T";
    0x11 / 0xe => "short.string-hand-tap", "string hand tap", "Pitched string-hand tap", "T";
    0x11 / 0xf => "short.jete", "jeté", "Jeté", "G1";
    0x12 / 0x0 => "repeat.tremolo", "unmeasured tremolo", "Tremolo / flutter tongue / equivalent roll", "N12";
    0x12 / 0x1 => "repeat.growl", "growl", "Growl / razz", "T";
    0x12 / 0x2 => "repeat.coloristic-tremolo", "coloristic tremolo", "Coloristic tremolo", "T";
    0x12 / 0x3 => "repeat.timbral-trill", "timbral trill", "Same-pitch timbral trill", "T";
    0x12 / 0x4 => "repeat.two", "two repeats", "Two-repeat sound", "N13";
    0x12 / 0x5 => "repeat.three", "three repeats", "Three-repeat sound", "N14";
    0x12 / 0x6 => "repeat.four", "four repeats", "Four-repeat sound", "P";
    0x12 / 0x7 => "repeat.five", "five repeats", "Five-repeat sound", "P";
    0x12 / 0x8 => "repeat.six", "six repeats", "Six-repeat sound", "P";
    0x12 / 0x9 => "repeat.fast", "fast repeats", "Faster repeat sound", "P";
    0x13 / 0x0 => "trill.semitone-main-first", "semitone trill main first", "Semitone trill, main note first", "N15";
    0x13 / 0x1 => "trill.semitone-upper-first", "semitone trill upper first", "Semitone trill, upper note first", "N15";
    0x13 / 0x2 => "trill.tone-main-first", "whole tone trill main first", "Whole-tone trill, main note first", "N15";
    0x13 / 0x3 => "trill.tone-upper-first", "whole tone trill upper first", "Whole-tone trill, upper note first", "N15";
    0x13 / 0x4 => "trill.minor-third", "minor third trill", "Minor-third alternation", "N16";
    0x13 / 0x5 => "trill.major-third", "major third trill", "Major-third alternation", "N16";
    0x13 / 0x6 => "trill.fourth", "perfect fourth trill", "Perfect-fourth alternation", "N16";
    0x13 / 0x7 => "trill.tritone", "tritone trill", "Tritone alternation", "N16";
    0x13 / 0x8 => "trill.fifth", "perfect fifth trill", "Perfect-fifth alternation", "N16";
    0x13 / 0x9 => "trill.minor-sixth", "minor sixth trill", "Minor-sixth alternation", "N16";
    0x13 / 0xa => "trill.major-sixth", "major sixth trill", "Major-sixth alternation", "N16";
    0x13 / 0xb => "trill.minor-seventh", "minor seventh trill", "Minor-seventh alternation", "N16";
    0x13 / 0xc => "trill.major-seventh", "major seventh trill", "Major-seventh alternation", "N16";
    0x13 / 0xd => "trill.octave", "octave trill", "Octave alternation", "N16";
    0x14 / 0x0 => "color.natural-harmonic", "natural harmonic", "Natural harmonic", "N17";
    0x14 / 0x1 => "color.artificial-harmonic", "artificial harmonic", "Artificial harmonic", "N18";
    0x14 / 0x2 => "color.col-legno-tratto", "col legno tratto", "Col legno tratto", "T";
    0x14 / 0x3 => "color.flautando", "flautando", "Flautando", "T";
    0x14 / 0x4 => "color.polyphonic-octaves", "polyphonic octaves", "Polyphonic octave sound", "P";
    0x14 / 0x5 => "color.polyphonic-chord", "polyphonic chord", "Polyphonic interval/chord sound", "P";
    0x14 / 0x6 => "color.cuivre", "cuivré", "Cuivré", "T";
    0x14 / 0x7 => "color.lontano", "lontano", "Lontano", "T";
    0x14 / 0x8 => "color.singing", "singing into instrument", "Singing into the instrument", "T";
    0x15 / 0x0 => "gesture.fall-end", "fall at end", "Fall at end, starting at note pitch", "N19";
    0x15 / 0x1 => "gesture.fall-start", "fall into note", "Fall into note from above", "N20";
    0x15 / 0x2 => "gesture.rise-end", "rise at end", "Rise at end, starting at note pitch", "N21";
    0x15 / 0x3 => "gesture.rise-start", "rise into note", "Rise into note from below", "N22";
    0x15 / 0x4 => "gesture.blue-down", "blue note down and return", "Dip and return to pitch", "P";
    0x15 / 0x5 => "gesture.blue-up", "blue note up and return", "Rise and return to pitch", "P";
    0x15 / 0x6 => "gesture.grace-below", "grace from below", "Leading grace sound from below", "N23";
    0x15 / 0x7 => "gesture.grace-above", "grace from above", "Leading grace sound from above", "N23";
    0x15 / 0x8 => "gesture.shake", "shake", "Shake sound", "N24";
    0x15 / 0x9 => "gesture.crescendo", "sampled crescendo", "Prerecorded crescendo", "N25";
    0x15 / 0xa => "gesture.decrescendo", "sampled decrescendo", "Prerecorded decrescendo", "N25";
    0x15 / 0xb => "gesture.swell", "sampled crescendo decrescendo", "Prerecorded swell then fade", "N25";
    0x15 / 0xc => "gesture.reverse-swell", "sampled decrescendo crescendo", "Prerecorded fade then swell", "N25";
    0x15 / 0xd => "gesture.sfz-crescendo", "sampled sforzando crescendo", "Sforzando then crescendo sound", "N25";
    0x16 / 0x0 => "run.playable", "playable runs", "Playable runs", "T";
    0x16 / 0x1 => "run.playable-tremolo", "playable tremolos", "Playable tremolos", "T";
    0x16 / 0x2 => "run.playable-glissando", "playable glissando", "Playable glissando", "T";
    0x16 / 0x3 => "run.major", "sampled major run", "Major scale/run", "P";
    0x16 / 0x4 => "run.minor", "sampled minor run", "Minor scale/run", "P";
    0x16 / 0x5 => "run.dominant-seventh", "sampled dominant seventh run", "Dominant-seventh scale/run", "P";
    0x16 / 0x6 => "run.diminished-whole-half", "sampled diminished whole half run", "Diminished whole–half / Other 1", "P";
    0x16 / 0x7 => "run.diminished-half-whole", "sampled diminished half whole run", "Diminished half–whole / Other 2", "P";
    0x16 / 0x8 => "run.whole-tone-c", "sampled whole tone run containing c", "Whole-tone collection containing C", "P";
    0x16 / 0x9 => "run.whole-tone-c-sharp", "sampled whole tone run containing c sharp", "Whole-tone collection containing C♯", "P";
    0x16 / 0xa => "run.pentatonic-major", "sampled major pentatonic run", "Major pentatonic scale/run", "P";
    0x16 / 0xb => "run.pentatonic-minor", "sampled minor pentatonic run", "Minor pentatonic scale/run", "P";
    0x16 / 0xc => "run.lydian", "sampled lydian run", "Lydian scale/run", "P";
    0x16 / 0xd => "run.lydian-flat-seven", "sampled lydian flat seven run", "Lydian ♭7 scale/run", "P";
    0x16 / 0xe => "run.chromatic", "sampled chromatic run", "Chromatic scale/run", "P";
    0x16 / 0xf => "run.other", "sampled other run", "Other scale/run", "P";
    0x17 / 0x0 => "effect.sustained-noise", "sustained noise", "Sustained assorted noise / air effect", "T";
    0x17 / 0x1 => "effect.behind-bridge", "bow behind bridge", "Sustained behind-bridge effect", "G2";
    0x17 / 0x2 => "effect.random-pizzicato", "random pizzicato", "Sustained random pizzicato", "P";
    0x17 / 0x3 => "effect.harmonic-glissando", "harmonic glissando effect", "Harmonic glissando effect", "P";
    0x17 / 0x4 => "effect.random-glissando", "random glissando effect", "Random glissando effect", "P";
    0x17 / 0x5 => "effect.short-air", "short air noise", "Short air / chiff / squeak / lip-pizz effect", "T";
    0x17 / 0x6 => "effect.mechanical-long", "sustained mechanical noise", "Sustained mechanical noise", "T";
    0x17 / 0x7 => "effect.mechanical-short", "short mechanical noise", "Short mechanical noise", "T";
    0x17 / 0x8 => "effect.finger-glide", "finger glide noise", "Finger glide noise", "T";
    0x17 / 0x9 => "effect.fret-noise", "fret noise", "Fret noise", "T";
    0x17 / 0xa => "effect.thump", "thump", "Thump", "T";
    0x17 / 0xb => "effect.knock", "knock", "Knock", "T";
    0x17 / 0xc => "effect.slap", "body slap", "Slap", "T";
    0x17 / 0xd => "effect.pop", "pop", "Pop", "T";
    0x17 / 0xe => "effect.tap", "body tap", "Tap", "T";
    0x17 / 0xf => "effect.click", "click", "Click", "T";
}
#[must_use]
pub fn by_id(id: &str) -> Option<&'static Vocabulary> {
    VOCABULARY.iter().find(|r| r.id == id)
}
#[must_use]
pub fn by_coordinate(at: u8, sub: u8) -> Option<&'static Vocabulary> {
    VOCABULARY.iter().find(|r| r.at == at && r.sub == sub)
}
#[must_use]
pub fn normalize(text: &str) -> String {
    text.nfc()
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
#[must_use]
pub fn by_term(term: &str) -> Option<&'static Vocabulary> {
    let text = normalize(term);
    let alias = match text.as_str() {
        "legato slurred" => "legato",
        "staccato" | "stacc." => "staccato off string",
        "detache" => "detaché",
        "martele" => "martelé",
        "sautille" => "sautillé",
        "jete" | "gettato" => "jeté",
        "non vibrato" | "non-vibrato" | "senza vib." => "senza vibrato",
        "vibrato" | "con vib." => "con vibrato",
        "bartok pizzicato" | "bartók pizzicato" | "snap pizz." => "snap pizzicato",
        "ord." | "ordinario" => "ordinary",
        "louré" | "loure" | "mezzo staccato" => "long staccato",
        "tremolo" => "unmeasured tremolo",
        "razz" => "growl",
        _ => &text,
    };
    VOCABULARY.iter().find(|r| r.term == alias || r.id == alias)
}

/// Finite manual-grounded refinements. These are named musical conventions,
/// never inferred plugin addresses or executable user expressions.
#[must_use]
pub fn term(text: &str) -> Option<(Articulation, Option<Descriptor>)> {
    if let Some(row) = by_term(text) {
        return Some((Articulation::named(row.id).ok()?, None));
    }
    let text = normalize(text);
    // M2-123 §5.2.3 names analogous repeating techniques across instruments.
    // The common classification does not erase the actual production requested.
    if matches!(
        text.as_str(),
        "flutter tongue" | "fluttertongue" | "flutter-tongue" | "bisbigliando"
    ) {
        return Some((
            Articulation::named("repeat.tremolo").ok()?,
            Some(Descriptor {
                technique: "repeat.tremolo".into(),
                qualifiers: std::collections::BTreeMap::from([(
                    "urn:midi2workbench:production".into(),
                    serde_json::json!(if text == "bisbigliando" {
                        "bisbigliando"
                    } else {
                        "flutter-tongue"
                    }),
                )]),
            }),
        ));
    }
    if matches!(text.as_str(), "hotel mute" | "recorded hotel mute") {
        return Some((
            Articulation::default(),
            Some(Descriptor {
                technique: "core.normal".into(),
                qualifiers: std::collections::BTreeMap::from([
                    (
                        "urn:midi2workbench:mute-identity".into(),
                        serde_json::json!("urn:orchestraltools:berlin:hotel-mute"),
                    ),
                    (
                        "urn:midi2workbench:mute-provenance".into(),
                        serde_json::json!("recorded"),
                    ),
                ]),
            }),
        ));
    }
    let (id, facet, value) = match text.as_str() {
        "spiccato blurred" | "blurred spiccato" => {
            ("short.spiccato", "sample-character", "blurred")
        }
        "spiccato exposed" => ("short.spiccato", "sample-character", "exposed"),
        "staccato blurred" => ("short.staccato-off", "sample-character", "blurred"),
        "expressive legato" | "legato expressive" => {
            ("core.legato", "connection-style", "expressive")
        }
        "cinematic legato" | "legato cinematic" => ("core.legato", "connection-style", "cinematic"),
        "fingered legato" => ("core.legato", "connection-style", "fingered"),
        "evolving vibrato" => ("core.con-vibrato", "vibrato-envelope", "evolving"),
        "romantic vibrato" => ("core.con-vibrato", "vibrato-style", "romantic"),
        "molto vibrato" => ("core.con-vibrato", "vibrato-amount", "molto"),
        "harmonic staccato" => ("short.staccato-off", "production", "harmonic"),
        "harmonic tremolo" => ("repeat.tremolo", "production", "harmonic"),
        "portato short" => ("short.portato", "sample-length", "short"),
        "portato long" => ("short.portato", "sample-length", "long"),
        "sustains soft" => ("core.normal", "attack-character", "soft"),
        "sustains accented" => ("core.normal", "attack-character", "accented"),
        _ => return None,
    };
    let art = Articulation::named(id).ok()?;
    let descriptor = Descriptor {
        technique: id.into(),
        qualifiers: std::collections::BTreeMap::from([(
            format!("urn:midi2workbench:{facet}"),
            serde_json::json!(value),
        )]),
    };
    Some((art, Some(descriptor)))
}

fn facet(art: Articulation) -> u8 {
    match (art.at, art.subclass) {
        (0x10, 0 | 1) => 0,
        (0x10, 2..=4) => 1,
        (0x10, 8..=10) | (0x14, 2..=15) => 2,
        (0x11, 10..=14) => 6,
        (0x12..=0x13 | 0x15..=0x16, _) => 4,
        (0x11, _) | (0x10, 6) => 5,
        _ => 3,
    }
}

/// SMuFL 1.4 canonical glyph identities (placement variants share semantics).
#[must_use]
pub fn glyph(name: &str) -> Option<(Articulation, Option<Descriptor>)> {
    let id = match name {
        "stringsJeteAbove" | "stringsJeteBelow" => "short.jete",
        "pluckedSnapPizzicatoAbove" | "pluckedSnapPizzicatoBelow" => "short.snap-pizzicato",
        "articAccentStaccatoAbove"
        | "articAccentStaccatoBelow"
        | "articMarcatoStaccatoAbove"
        | "articMarcatoStaccatoBelow" => "short.accented-staccato",
        "articTenutoStaccatoAbove" | "articTenutoStaccatoBelow" => "short.portato",
        "stringsBowBehindBridge"
        | "stringsBowBehindBridgeOneString"
        | "stringsBowBehindBridgeTwoStrings"
        | "stringsBowBehindBridgeThreeStrings"
        | "stringsBowBehindBridgeFourStrings" => "effect.behind-bridge",
        "stringsDownBow" | "stringsDownBowTurned" => {
            return Some((
                Articulation {
                    direction: 1,
                    ..Default::default()
                },
                None,
            ));
        }
        "pluckedLeftHandPizzicato" => {
            let mut art = Articulation::named("short.pizzicato").ok()?;
            art.direction = 2;
            return Some((art, None));
        }
        "stringsUpBow" | "stringsUpBowTurned" => {
            return Some((
                Articulation {
                    direction: 2,
                    ..Default::default()
                },
                None,
            ));
        }
        "stringsHarmonic"
        | "stringsHalfHarmonic"
        | "stringsOverpressureDownBow"
        | "stringsOverpressureUpBow"
        | "stringsBowOnTailpiece"
        | "stringsBowOnBridge" => {
            let art = Articulation {
                direction: if name.ends_with("DownBow") {
                    1
                } else if name.ends_with("UpBow") {
                    2
                } else {
                    0
                },
                ..Articulation::default()
            };
            return Some((
                art,
                Some(Descriptor {
                    technique: "core.normal".into(),
                    qualifiers: std::collections::BTreeMap::from([(
                        "urn:midi2workbench:production-glyph".into(),
                        serde_json::json!(name),
                    )]),
                }),
            ));
        }
        _ => return None,
    };
    let art = Articulation::named(id).ok()?;
    let count = match name {
        "stringsBowBehindBridgeOneString" => 1,
        "stringsBowBehindBridgeTwoStrings" => 2,
        "stringsBowBehindBridgeThreeStrings" => 3,
        "stringsBowBehindBridgeFourStrings" => 4,
        _ => 0,
    };
    let descriptor = (count > 0).then(|| Descriptor {
        technique: id.into(),
        qualifiers: std::collections::BTreeMap::from([(
            "urn:midi2workbench:string-count".into(),
            serde_json::json!(count),
        )]),
    });
    Some((art, descriptor))
}

/// Only invert predicates with an unambiguous native representation. A profile
/// label alone cannot establish a slur/glissando endpoint or a trill schedule.
#[must_use]
pub fn native_notation(art: Articulation) -> Option<&'static str> {
    Some(match (art.at, art.subclass) {
        (0x10, 0) | (0x11, 10) => "",
        (0x10, 6) => "<articulations><strong-accent/></articulations>",
        (0x11, 0) => "<articulations><staccato/></articulations>",
        (0x11, 3) => "<articulations><staccato/><accent/></articulations>",
        (0x11, 4) => "<articulations><staccatissimo/></articulations>",
        (0x11, 5) => "<articulations><spiccato/></articulations>",
        (0x11, 9) => "<articulations><detached-legato/></articulations>",
        (0x11, 11) => "<technical><snap-pizzicato/></technical>",
        (0x12, 0) => "<ornaments><tremolo type=\"unmeasured\">0</tremolo></ornaments>",
        (0x14, 0) => "<technical><harmonic><natural/><sounding-pitch/></harmonic></technical>",
        (0x14, 1) => "<technical><harmonic><artificial/><sounding-pitch/></harmonic></technical>",
        (0x15, 0) => "<articulations><falloff/></articulations>",
        (0x15, 1) => "<articulations><plop/></articulations>",
        (0x15, 2) => "<articulations><doit/></articulations>",
        (0x15, 3) => "<articulations><scoop/></articulations>",
        _ => return None,
    })
}
/// Resolve independent facets without an XML-order-dependent winner.
/// # Panics
/// Panics if a bundled schema/catalog or an internally validated invariant is inconsistent.
/// # Errors
/// Returns a diagnostic for reserved coordinates, incompatible facets or out-of-range values.
pub fn combine(
    a: Articulation,
    ad: Option<Descriptor>,
    b: Articulation,
    bd: Option<Descriptor>,
) -> Result<(Articulation, Option<Descriptor>)> {
    let same = (a.at, a.subclass) == (b.at, b.subclass);
    let fused = match ((a.at, a.subclass), (b.at, b.subclass)) {
        ((0x11, 0..=2), (0x10, 6)) | ((0x10, 6), (0x11, 0..=2)) => Some((0x11, 3)),
        ((0x11, 2), (0x11, 0)) | ((0x11, 0), (0x11, 2)) => Some((0x11, 2)),
        _ => None,
    };
    if !same && fused.is_none() && facet(a) != 0 && facet(a) == facet(b) {
        return Err(Diagnostic::new(
            "E_CONFLICT",
            format!(
                "incompatible primary sounds {:x}/{} and {:x}/{}",
                a.at, a.subclass, b.at, b.subclass
            ),
        ));
    }
    let mut primary = if !same && ((a.at, a.subclass) == (0x10, 0) || facet(b) > facet(a)) {
        b
    } else {
        a
    };
    if let Some((at, subclass)) = fused {
        primary.at = at;
        primary.subclass = subclass;
    }
    if (a.direction != 0 && b.direction != 0 && a.direction != b.direction)
        || (a.string != 0 && b.string != 0 && a.string != b.string)
    {
        return Err(Diagnostic::new("E_CONFLICT", "conflicting note modifiers"));
    }
    if a.release != 0 && b.release != 0 && a.release != b.release
        || same && a.variation != 0 && b.variation != 0 && a.variation != b.variation
    {
        return Err(Diagnostic::new(
            "E_CONFLICT",
            "conflicting release or variation",
        ));
    }
    primary.release = a.release.max(b.release);
    if same {
        primary.variation = a.variation.max(b.variation);
    }
    primary.direction = a.direction.max(b.direction);
    primary.string = a.string.max(b.string);
    primary.reset_round_robin = a.reset_round_robin || b.reset_round_robin;
    let mut descriptor = Descriptor {
        technique: by_coordinate(primary.at, primary.subclass)
            .ok_or_else(|| Diagnostic::new("E_TECHNIQUE", "unmapped primary"))?
            .id
            .into(),
        ..Descriptor::default()
    };
    for d in [ad, bd].into_iter().flatten() {
        for (key, value) in d.qualifiers {
            if descriptor
                .qualifiers
                .get(&key)
                .is_some_and(|old| *old != value)
            {
                return Err(Diagnostic::new(
                    "E_CONFLICT",
                    format!("conflicting qualifier {key}"),
                ));
            }
            descriptor.qualifiers.insert(key, value);
        }
    }
    if !same && fused.is_none() && facet(a) != 0 && facet(b) != 0 {
        let secondary = if (primary.at, primary.subclass) == (a.at, a.subclass) {
            b
        } else {
            a
        };
        let id = by_coordinate(secondary.at, secondary.subclass)
            .expect("validated secondary")
            .id;
        let values = descriptor
            .qualifiers
            .entry("urn:midi2workbench:facets".into())
            .or_insert_with(|| serde_json::json!([]));
        let values = values
            .as_array_mut()
            .ok_or_else(|| Diagnostic::new("E_CONFLICT", "facets must be an array"))?;
        if !values.contains(&serde_json::json!(id)) {
            values.push(serde_json::json!(id));
            values.sort_by_key(serde_json::Value::to_string);
        }
    }
    Ok((
        primary,
        (!descriptor.qualifiers.is_empty()).then_some(descriptor),
    ))
}
/// Validated orchestral note fields. No reserved values can be emitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Articulation {
    pub at: u8,
    pub subclass: u8,
    pub variation: u8,
    pub direction: u8,
    pub string: u8,
    pub reset_round_robin: bool,
    pub release: u8,
}
impl Default for Articulation {
    fn default() -> Self {
        Self {
            at: 0x10,
            subclass: 0,
            variation: 0,
            direction: 0,
            string: 0,
            reset_round_robin: false,
            release: 0,
        }
    }
}
impl Articulation {
    /// # Errors
    /// Returns a diagnostic for reserved coordinates, incompatible facets or out-of-range values.
    pub fn validate(self) -> Result<Self> {
        if (by_coordinate(self.at, self.subclass).is_none() && !(0x1a..=0x1f).contains(&self.at))
            || self.subclass > 15
            || self.variation > 15
            || self.direction > 2
            || self.string > 7
            || self.release > 4
        {
            return Err(Diagnostic::new(
                "E_ARTICULATION",
                format!("invalid or reserved fields {self:?}"),
            ));
        }
        Ok(self)
    }
    /// # Errors
    /// Returns a diagnostic for reserved coordinates, incompatible facets or out-of-range values.
    pub fn data(self) -> Result<u16> {
        self.validate()?;
        Ok(u16::from(self.subclass) << 12
            | u16::from(self.variation) << 8
            | u16::from(self.direction) << 6
            | u16::from(self.reset_round_robin) << 3
            | u16::from(self.string))
    }
    /// # Errors
    /// Returns a diagnostic for reserved coordinates, incompatible facets or out-of-range values.
    pub fn from_data(at: u8, data: u16) -> Result<Self> {
        if data & 0x30 != 0 {
            return Err(Diagnostic::new(
                "E_RESERVED",
                "Note On bits 5–4 must be zero",
            ));
        }
        Self {
            at,
            subclass: (data >> 12) as u8,
            variation: ((data >> 8) & 15) as u8,
            direction: ((data >> 6) & 3) as u8,
            string: (data & 7) as u8,
            reset_round_robin: data & 8 != 0,
            release: 0,
        }
        .validate()
    }
    /// # Errors
    /// Returns a diagnostic for reserved coordinates, incompatible facets or out-of-range values.
    pub fn named(id: &str) -> Result<Self> {
        let row = by_id(id)
            .or_else(|| by_term(id))
            .ok_or_else(|| Diagnostic::new("E_TECHNIQUE", id))?;
        Ok(Self {
            at: row.at,
            subclass: row.sub,
            ..Self::default()
        })
    }
}
/// Finite native predicates are Rust rules; compatible secondary facets survive.
/// See docs/architecture/convert.md N0–N25 for authority, interpretive choices and
/// counterexamples. In particular staccato does not prove a bouncing bow; 11/0
/// is this converter's ordinary-short default, and MusicXML spiccato is a stroke
/// mark. Do not derive either bow mechanics or a rate threshold from duration.
/// # Panics
/// Panics if a bundled schema/catalog or an internally validated invariant is inconsistent.
/// # Errors
/// Returns a diagnostic for reserved coordinates, incompatible facets or out-of-range values.
#[allow(clippy::too_many_lines)] // One finite native-notation decision table, with residual facets.
pub fn native(
    tags: &[String],
    slurred: bool,
    pizzicato: bool,
) -> Result<(Articulation, Option<Descriptor>)> {
    let has = |tag: &str| tags.iter().any(|s| s == tag);
    if has("down-bow") && has("up-bow") {
        return Err(Diagnostic::new(
            "E_CONFLICT",
            "simultaneous up-bow and down-bow",
        ));
    }
    if (has("stress") || has("accent") || has("strong-accent"))
        && (has("unstress") || has("soft-accent"))
    {
        return Err(Diagnostic::new(
            "E_CONFLICT",
            "simultaneous increased and decreased attack emphasis",
        ));
    }
    let coord = if (has("staccato") && has("tenuto")) || has("detached-legato") {
        (0x11, 9)
    } else if has("staccato") && (has("accent") || has("strong-accent")) {
        (0x11, 3)
    } else if has("staccatissimo") {
        (0x11, 4)
    } else if has("spiccato") {
        (0x11, 5)
    } else if has("staccato") && slurred {
        (0x11, 2)
    } else if has("staccato") {
        (0x11, 0)
    } else if has("natural") {
        (0x14, 0)
    } else if has("artificial") {
        (0x14, 1)
    } else if has("falloff") {
        (0x15, 0)
    } else if has("plop") {
        (0x15, 1)
    } else if has("doit") {
        (0x15, 2)
    } else if has("scoop") {
        (0x15, 3)
    } else if has("glissando") || has("slide") {
        (0x10, 4)
    } else if has("strong-accent") {
        (0x10, 6)
    } else if slurred {
        (0x10, 2)
    } else {
        (0x10, 0)
    };
    let mut art = Articulation {
        at: coord.0,
        subclass: coord.1,
        ..Articulation::default()
    };
    art.direction = if has("down-bow") {
        1
    } else if has("up-bow") || has("left-hand-pizzicato") {
        2
    } else {
        0
    };
    let mut residual = Descriptor {
        technique: by_coordinate(art.at, art.subclass)
            .expect("native coordinate")
            .id
            .into(),
        ..Descriptor::default()
    };
    let marks = ["tenuto", "stress", "unstress", "soft-accent"]
        .into_iter()
        .filter(|tag| has(tag))
        .collect::<Vec<_>>();
    if !marks.is_empty() {
        residual.qualifiers.insert(
            "urn:midi2workbench:source-articulation-marks".into(),
            serde_json::json!(marks),
        );
    }
    if (has("harmonic") || has("natural") || has("artificial"))
        && !(art.at == 0x14 && art.subclass <= 1)
    {
        residual.qualifiers.insert(
            "urn:midi2workbench:production".into(),
            serde_json::json!(if has("artificial") {
                "artificial-harmonic"
            } else {
                "harmonic"
            }),
        );
    }
    let descriptor = (!residual.qualifiers.is_empty()).then_some(residual);
    if has("snap-pizzicato") || pizzicato || has("left-hand-pizzicato") {
        combine(
            art,
            descriptor,
            Articulation::named(if has("snap-pizzicato") {
                "short.snap-pizzicato"
            } else {
                "short.pizzicato"
            })?,
            None,
        )
    } else {
        Ok((art, descriptor))
    }
}
/// Explicit attack schedules consume sampled-pattern triggers while retaining
/// their requested identity in the per-note refinement convention.
pub fn consume_pattern(art: &mut Articulation, descriptor: &mut Descriptor) {
    if matches!(art.at, 0x12 | 0x13 | 0x16) {
        descriptor.qualifiers.insert(
            "urn:midi2workbench:consumed-pattern".into(),
            serde_json::json!(art),
        );
        art.at = 0x10;
        art.subclass = 0;
        art.variation = 0;
        descriptor.technique = "core.normal".into();
    }
}

/// M2-115 min-center-max bit replication (ordinary velocity and controls).
/// # Panics
/// Panics unless `2 <= from <= to <= 32` and the value fits `from` bits.
#[must_use]
pub fn upscale(value: u32, from: u32, to: u32) -> u32 {
    assert!(
        (2..=32).contains(&from) && from <= to && to <= 32 && u64::from(value) < (1_u64 << from),
        "valid bit widths and source magnitude required"
    );
    let shift = to - from;
    let mut result = u64::from(value) << shift;
    if value > (1 << (from - 1)) {
        let bits = from - 1;
        let mut repeat = u64::from(value & ((1 << bits) - 1));
        repeat = if shift > bits {
            repeat << (shift - bits)
        } else {
            repeat >> (bits - shift)
        };
        while repeat != 0 {
            result |= repeat;
            repeat >>= bits;
        }
    }
    u32::try_from(result).expect("scaled magnitude fits destination width")
}

/// Preserve decimal percentages by interpolation between M2-115 lookup entries.
/// # Errors
/// Returns a diagnostic for reserved coordinates, incompatible facets or out-of-range values.
pub fn level(value: crate::model::Rational, bits: u32) -> Result<u32> {
    u32::try_from(level_exact(value, bits)?.round().to_integer())
        .map_err(|_| Diagnostic::new("E_DYNAMICS", "scaled level out of range"))
}
pub(crate) fn level_exact(
    value: crate::model::Rational,
    bits: u32,
) -> Result<num_rational::Ratio<i128>> {
    use crate::model::Rational;
    if !(7..=32).contains(&bits)
        || value < Rational::from_integer(0)
        || value > Rational::from_integer(127)
    {
        return Err(Diagnostic::new(
            "E_DYNAMICS",
            format!("nominal velocity {value} outside 0..127"),
        ));
    }
    let lo = u32::try_from(value.floor().to_integer())
        .map_err(|_| Diagnostic::new("E_DYNAMICS", "level out of range"))?;
    let a = upscale(lo, 7, bits);
    let b = upscale((lo + 1).min(127), 7, bits);
    let wide = num_rational::Ratio::new(i128::from(*value.numer()), i128::from(*value.denom()));
    let encoded = num_rational::Ratio::from_integer(i128::from(a))
        + (wide - num_rational::Ratio::from_integer(i128::from(lo)))
            * num_rational::Ratio::from_integer(i128::from(b - a));
    Ok(encoded)
}
