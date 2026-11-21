//! File validity and score admissibility share the production decoder.
use crate::{
    arrangement, clip,
    model::{Performance, Route},
    remarks,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Analysis {
    pub valid: bool,
    pub admissible: bool,
    pub notes: usize,
    pub parts: usize,
    pub routes: Vec<Route>,
    pub pitch_range: Option<(u8, u8)>,
    pub duration_beats: String,
    pub controllers: BTreeMap<String, usize>,
    pub messages: BTreeMap<String, usize>,
    pub articulations: BTreeMap<String, usize>,
    pub diagnostics: Vec<String>,
}
/// Inspect without a companion. Failure remains a report suitable for CLI automation.
#[must_use]
pub fn inspect(bytes: &[u8]) -> Analysis {
    let mut report = Analysis {
        valid: false,
        admissible: false,
        notes: 0,
        parts: 0,
        routes: Vec::new(),
        pitch_range: None,
        duration_beats: "0".into(),
        controllers: BTreeMap::new(),
        messages: BTreeMap::new(),
        articulations: BTreeMap::new(),
        diagnostics: Vec::new(),
    };
    match clip::decode(bytes, None) {
        Err(error) => report.diagnostics.push(error.to_string()),
        Ok(score) => analyze(&score, &mut report),
    }
    report
}
fn analyze(score: &Performance, report: &mut Analysis) {
    report.valid = true;
    report.admissible = true;
    report.notes = score.notes.len();
    report.duration_beats = score.duration.to_string();
    report.routes = score
        .notes
        .iter()
        .map(|n| n.route)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    report.parts = report.routes.len();
    if let (Some(min), Some(max)) = (
        score.notes.iter().map(|n| n.key).min(),
        score.notes.iter().map(|n| n.key).max(),
    ) {
        report.pitch_range = Some((min, max));
    }
    match arrangement::read(score) {
        Err(e) => {
            report.admissible = false;
            report.diagnostics.push(e.to_string());
        }
        Ok(Some(a)) => {
            report.parts = a.parts.len();
            let routes: BTreeSet<_> = a
                .parts
                .iter()
                .flat_map(|p| p.routes.iter().copied())
                .collect();
            if report.routes.iter().any(|r| !routes.contains(r)) {
                report.admissible = false;
                report
                    .diagnostics
                    .push("Musical part metadata does not own every note route".into());
            }
            report.diagnostics.extend(a.decisions);
        }
        Ok(None) => report
            .diagnostics
            .push("No musical part metadata: canonical route-based score reconstruction".into()),
    }
    for n in &score.notes {
        let label = crate::rules::by_coordinate(n.articulation.at, n.articulation.subclass)
            .map_or("custom", |r| r.id);
        *report.articulations.entry(label.into()).or_default() += 1;
        if let Some(d) = &n.descriptor {
            for key in d.qualifiers.keys() {
                *report
                    .messages
                    .entry(format!("note qualifier {key}"))
                    .or_default() += 1;
            }
        }
    }
    inventory(score, report);
}
fn inventory(score: &Performance, report: &mut Analysis) {
    let mut tempo = false;
    let mut meter = false;
    for e in score.events.iter().chain(&score.configuration) {
        let first = e.words[0];
        if first >> 28 == 2 && matches!((first >> 20) & 15, 8 | 9) {
            report.admissible = false;
            report.diagnostics.push(format!("MIDI1 UMP note at beat {} is retained but not reconstructed by the MIDI2 note decoder",e.time));
        }
        let name = if first >> 28 == 0xd {
            match first & 0xffff {
                0 => {
                    tempo = true;
                    "tempo"
                }
                1 => {
                    meter = true;
                    "meter"
                }
                5 => "key signature",
                0x0100 => "metadata text",
                0x0200 => "performance remark",
                0x0201 => "lyrics",
                0x0202 => "lyric language",
                _ => "Flex Data",
            }
        } else if first >> 28 == 4 && (first >> 20) & 15 == 11 {
            *report
                .controllers
                .entry(format!(
                    "g{} ch{} CC{}",
                    (first >> 24) & 15,
                    ((first >> 16) & 15) + 1,
                    (first >> 8) & 127
                ))
                .or_default() += 1;
            "MIDI2 CC"
        } else {
            "other UMP"
        };
        *report.messages.entry(name.into()).or_default() += 1;
        if let Some((_, text)) = remarks::performance(&e.words) {
            if text.starts_with(arrangement::COMMAND) {
                *report
                    .messages
                    .entry("declared command".into())
                    .or_default() += 1;
            }
        }
    }
    if !tempo {
        report
            .diagnostics
            .push("Missing tempo: score playback defaults to 120 quarter notes/minute".into());
    }
    if !meter {
        report
            .diagnostics
            .push("Missing meter: editorial 4/4 required for measured notation".into());
    }
}
