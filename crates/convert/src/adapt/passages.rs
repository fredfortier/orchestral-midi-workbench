use super::{Plan, fail};
use crate::{Result, model::Performance};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Independent, reviewed expectation for a set of performed source notes.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Passage {
    pub id: String,
    pub source_prefixes: Vec<String>,
    pub track: String,
    pub instrument: String,
    pub expected_notes: usize,
    pub reason: String,
}

pub(super) fn check(performance: &mut Performance, plan: &Plan) -> Result<()> {
    if plan.passages.is_empty() {
        performance.report.record(
            "/",
            "passage review",
            "not supplied; no musical review asserted",
        );
        return Ok(());
    }
    validate(plan)?;
    let mut counts = vec![0; plan.passages.len()];
    let mut used_prefixes: Vec<Vec<bool>> = plan
        .passages
        .iter()
        .map(|p| vec![false; p.source_prefixes.len()])
        .collect();
    for note in &performance.notes {
        let mut selected = None;
        for (index, passage) in plan.passages.iter().enumerate() {
            for (prefix_index, prefix) in passage.source_prefixes.iter().enumerate() {
                if note.source.starts_with(prefix) {
                    if selected.replace(index).is_some() {
                        return Err(fail(format!(
                            "overlapping reviewed passage selectors: {}",
                            note.source
                        )));
                    }
                    used_prefixes[index][prefix_index] = true;
                }
            }
        }
        let index =
            selected.ok_or_else(|| fail(format!("unreviewed source note: {}", note.source)))?;
        let passage = &plan.passages[index];
        let binding = plan
            .rendition
            .bindings
            .iter()
            .find(|binding| binding.route == note.route)
            .ok_or_else(|| fail(format!("unbound reviewed note: {}", note.source)))?;
        if binding.instrument != passage.instrument {
            return Err(fail(format!(
                "reviewed passage {} expected instrument {}, observed {} for {}",
                passage.id, passage.instrument, binding.instrument, note.source
            )));
        }
        if binding.track != passage.track {
            return Err(fail(format!(
                "reviewed passage {} expected track {}, observed {} for {}",
                passage.id, passage.track, binding.track, note.source
            )));
        }
        counts[index] += 1;
    }
    for ((passage, count), used) in plan.passages.iter().zip(counts).zip(used_prefixes) {
        if count != passage.expected_notes || used.contains(&false) {
            return Err(fail(format!(
                "reviewed passage {} expected {} notes with every prefix used, observed {count}",
                passage.id, passage.expected_notes
            )));
        }
        performance.report.record(
            &passage.id,
            "reviewed passage",
            format!(
                "{count} notes -> {}: {}",
                passage.instrument, passage.reason
            ),
        );
    }
    Ok(())
}

fn validate(plan: &Plan) -> Result<()> {
    if plan.passages.len() > 4096
        || plan
            .passages
            .iter()
            .map(|p| p.source_prefixes.len())
            .sum::<usize>()
            > 4096
    {
        return Err(fail("reviewed passage budget"));
    }
    let mut ids = BTreeSet::new();
    for p in &plan.passages {
        if p.id.trim().is_empty()
            || p.id.len() > 128
            || !ids.insert(&p.id)
            || p.reason.trim().is_empty()
            || p.reason.len() > 4096
            || p.expected_notes == 0
            || p.source_prefixes.is_empty()
            || p.source_prefixes
                .iter()
                .any(|s| s.is_empty() || s.len() > 4096)
            || !plan.rendition.tracks.iter().any(|t| t.id == p.track)
            || !plan
                .rendition
                .instruments
                .iter()
                .any(|i| i.id == p.instrument)
        {
            return Err(fail(format!("invalid reviewed passage: {}", p.id)));
        }
    }
    Ok(())
}
