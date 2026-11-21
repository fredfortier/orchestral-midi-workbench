//! Exact musical values and versioned interpretation choices.
use crate::{Diagnostic, Result};
use num_rational::Ratio;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub type Rational = Ratio<i64>;
/// # Errors
/// Returns a diagnostic for invalid numeric values or unsupported profile requirements.
pub fn decimal(s: &str) -> Result<Rational> {
    let ratio = if let Some((numerator, denominator)) = s.split_once('/') {
        let parse = |part: &str| {
            part.parse::<i64>()
                .map(i128::from)
                .map_err(|_| Diagnostic::new("E_NUMBER", s))
        };
        let (numerator, denominator) = (parse(numerator)?, parse(denominator)?);
        if denominator == 0 {
            return Err(Diagnostic::new("E_NUMBER", s));
        }
        // Normalize in the wider type: negating i64::MIN can overflow even
        // when reduction would produce a representable fraction (MIN/MIN).
        Ratio::new(numerator, denominator)
    } else {
        let value = rust_decimal::Decimal::from_str_exact(s.trim())
            .map_err(|e| Diagnostic::new("E_NUMBER", format!("{s}: {e}")))?;
        Ratio::new(value.mantissa(), 10_i128.pow(value.scale()))
    };
    Ok(Rational::new(
        i64::try_from(*ratio.numer()).map_err(|_| Diagnostic::new("E_PRECISION", s))?,
        i64::try_from(*ratio.denom()).map_err(|_| Diagnostic::new("E_PRECISION", s))?,
    ))
}
#[must_use]
pub fn round(value: Rational) -> i64 {
    value.round().to_integer()
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub group: u8,
    pub channel: u8,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Descriptor {
    pub technique: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub qualifiers: BTreeMap<String, serde_json::Value>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    /// Relative attack order among retained events at this onset; zero uses score order.
    #[serde(default)]
    pub on_order: u32,
    #[serde(default)]
    pub off_order: u32,
    pub onset: Rational,
    pub end: Rational,
    pub pitch: Rational,
    pub velocity: u16,
    pub release_velocity: u16,
    pub route: Route,
    pub key: u8,
    pub articulation: crate::rules::Articulation,
    pub descriptor: Option<Descriptor>,
    pub source: String,
}
impl Note {
    pub(crate) fn needs_orchestral_profile(&self) -> bool {
        self.articulation != crate::rules::Articulation::default()
            || self
                .descriptor
                .as_ref()
                .is_some_and(|d| d.qualifiers.contains_key("urn:midi2workbench:position"))
    }
}
/// Other messages retain authored presentation order.
/// `order` counts retained messages and note boundaries at the same time; zero
/// requests the default score ordering. Generated initialization is not counted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    #[serde(default)]
    pub order: u32,
    pub time: Rational,
    pub words: Vec<u32>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Performance {
    /// Performed extent, including trailing rests.
    #[serde(default)]
    pub duration: Rational,
    pub notes: Vec<Note>,
    pub events: Vec<Event>,
    pub configuration: Vec<Event>,
    /// Additional standard M2-126 Piano Profile declarations; M2-123 remains the
    /// articulation contract. These profiles may coexist under M2-102 §2.8.
    #[serde(default)]
    pub piano_routes: std::collections::BTreeSet<Route>,
    pub report: Report,
}
impl Performance {
    /// Standard profile requirements; private refinements and release samples
    /// additionally require a receiver/program-specific semantic binding.
    /// # Errors
    /// Returns a diagnostic for invalid numeric values or unsupported profile requirements.
    pub fn required_features(&self, route: Option<Route>) -> Result<crate::ci::Features> {
        let mut required = crate::ci::Features {
            bits: 0,
            standard: [0; 128],
            custom: [0; 96],
        };
        for n in self
            .notes
            .iter()
            .filter(|n| route.is_none_or(|r| r == n.route))
        {
            n.articulation.validate()?;
            if self.piano_routes.contains(&n.route) && !n.needs_orchestral_profile() {
                continue;
            }
            let counts = if n.articulation.at < 0x18 {
                &mut required.standard[..]
            } else {
                &mut required.custom[..]
            };
            let first = if n.articulation.at < 0x18 { 16 } else { 26 };
            let index =
                usize::from(n.articulation.at - first) * 16 + usize::from(n.articulation.subclass);
            if let Some(count) = counts.get_mut(index) {
                *count = (*count).max(n.articulation.variation + 1);
            }
            if n.articulation.at >= 26 {
                required.bits |= 16;
            }
            if n.release_velocity != 0x8000 {
                required.bits |= 8;
            }
            if let Some(d) = &n.descriptor {
                if d.qualifiers.contains_key("urn:midi2workbench:position") {
                    required.bits |= 4;
                }
                if let Some(mute) = d
                    .qualifiers
                    .get("urn:midi2workbench:mute")
                    .filter(|_| !self.piano_routes.contains(&n.route))
                {
                    required.bits |= 1;
                    if mute
                        .get("amount")
                        .and_then(serde_json::Value::as_u64)
                        .is_some_and(|v| v != u64::from(u32::MAX))
                    {
                        required.bits |= 2;
                    }
                }
            }
        }
        for e in self.events.iter().chain(&self.configuration) {
            if let Some(&word) = e.words.first() {
                if word >> 28 == 4
                    && route.is_none_or(|r| {
                        u32::from(r.group) == (word >> 24) & 15
                            && u32::from(r.channel) == (word >> 16) & 15
                    })
                {
                    if word & 0x00f0_0000 == 0 && word.to_le_bytes()[0] == 12 {
                        required.bits |= 4;
                    }
                    if (word >> 20) & 15 == 2 && (word >> 8) & 127 == 32 {
                        match word.to_le_bytes()[0] {
                            0x22 => required.bits |= 1,
                            0x23 => required.bits |= 2,
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(required)
    }
    #[must_use]
    /// # Panics
    /// Panics only if serialization of the in-memory performance violates its JSON representation.
    pub fn semantic_values(&self) -> String {
        let mut notes = self.notes.clone();
        for n in &mut notes {
            n.source.clear();
        }
        notes.sort_by_key(|n| (n.onset, n.end, n.route, n.key));
        serde_json::to_string(&(
            self.duration,
            notes,
            &self.events,
            &self.configuration,
            &self.piano_routes,
        ))
        .expect("serializable performance")
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub policy: String,
    pub decisions: Vec<Decision>,
    pub source_constructs: BTreeMap<String, usize>,
    #[serde(default)]
    pub numeric_changes: Vec<serde_json::Value>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub source: String,
    pub rule: String,
    pub disposition: String,
}
impl Report {
    pub(crate) fn quantized(&mut self, source: &str, kind: &str, exact: Ratio<i128>, encoded: u32) {
        if exact != Ratio::from_integer(i128::from(encoded)) {
            self.numeric_changes.push(serde_json::json!({"source":source,"kind":kind,"exactMagnitude":exact.to_string(),"encoded":encoded}));
        }
    }
    pub fn record(
        &mut self,
        source: impl Into<String>,
        rule: impl Into<String>,
        disposition: impl Into<String>,
    ) {
        self.decisions.push(Decision {
            source: source.into(),
            rule: rule.into(),
            disposition: disposition.into(),
        });
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RenderingPolicy {
    pub version: String,
    pub default_tempo: u32,
    pub fermata_factor: Rational,
    pub breath_gap: Rational,
    pub caesura_gap: Rational,
    pub grace_share: Rational,
    pub transient_span: Rational,
    pub arpeggio_span: Rational,
}
impl Default for RenderingPolicy {
    fn default() -> Self {
        Self {
            version: "corpus-v1".into(),
            default_tempo: 120,
            fermata_factor: Rational::new(3, 2),
            breath_gap: Rational::new(1, 8),
            caesura_gap: Rational::new(1, 2),
            grace_share: Rational::new(1, 8),
            transient_span: Rational::new(1, 8),
            arpeggio_span: Rational::new(1, 8),
        }
    }
}
