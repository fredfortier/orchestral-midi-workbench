//! Private refinements carried by standard assignable per-note controllers.
use crate::{Diagnostic, Result, model::Descriptor};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Write as _;
pub const XML_TYPE: &str = "urn:midi2workbench:articulation:v2";
pub const DYNAMIC_CURVE: &str = "urn:midi2workbench:dynamic-curve";
pub const DYNAMIC_LEVEL: &str = "urn:midi2workbench:dynamic-level";
/// # Errors
/// Returns a diagnostic for invalid schema data, unsupported vocabulary or missing dictionary entries.
/// Points are (relative quarter-note offset, value, same-time event order).
pub fn dynamic_curve(descriptor: &Descriptor) -> Result<Vec<(crate::model::Rational, u32, u32)>> {
    descriptor.qualifiers.get(DYNAMIC_CURVE).map_or_else(
        || Ok(vec![]),
        |v| {
            serde_json::from_value(v.clone())
                .map_err(|e| Diagnostic::new("E_DYNAMIC_CURVE", e.to_string()))
        },
    )
}
/// # Panics
/// Panics if a bundled schema/catalog or an internally validated invariant is inconsistent.
/// # Errors
/// Returns a diagnostic for invalid schema data, unsupported vocabulary or missing dictionary entries.
pub fn validate_annotation(value: &serde_json::Value) -> Result<()> {
    static SCHEMA: std::sync::OnceLock<jsonschema::Validator> = std::sync::OnceLock::new();
    let validator = SCHEMA.get_or_init(|| {
        jsonschema::validator_for(
            &serde_json::from_str(include_str!("../mapping/annotation.schema.json"))
                .expect("checked annotation schema"),
        )
        .expect("valid annotation schema")
    });
    validator
        .validate(value)
        .map_err(|e| Diagnostic::new("E_ANNOTATION", e.to_string()))
}
// Annotation fields such as direction/release/order belong to the XML reader,
// not to Descriptor. Call after validate_annotation; project the descriptor's
// fields explicitly instead of relying on serde to discard the other facets.
pub(crate) fn annotation_descriptor(value: &serde_json::Value) -> Result<Descriptor> {
    serde_json::from_value(serde_json::json!({
        "technique": value["technique"],
        "qualifiers": value.get("qualifiers").cloned().unwrap_or_else(|| serde_json::json!({})),
    }))
    .map_err(|error| Diagnostic::new("E_ANNOTATION", error.to_string()))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dictionary {
    pub version: u32,
    pub controller_index: u8,
    pub descriptors: BTreeMap<u32, Descriptor>,
    #[serde(default)]
    pub parameters: BTreeMap<u8, String>,
}
impl Default for Dictionary {
    fn default() -> Self {
        Self {
            version: 1,
            controller_index: 0x7f,
            descriptors: BTreeMap::new(),
            parameters: BTreeMap::new(),
        }
    }
}
impl Dictionary {
    /// # Panics
    /// Panics if a bundled schema/catalog or an internally validated invariant is inconsistent.
    pub fn from_descriptors<'a>(items: impl Iterator<Item = &'a Descriptor>) -> Self {
        let mut canonical = BTreeMap::new();
        for item in items {
            canonical.insert(
                serde_json::to_string(item).expect("descriptor JSON"),
                item.clone(),
            );
        }
        let parameters = if canonical
            .values()
            .any(|d| d.qualifiers.contains_key(DYNAMIC_CURVE))
        {
            BTreeMap::from([(126, DYNAMIC_LEVEL.into())])
        } else {
            BTreeMap::new()
        };
        Self {
            parameters,
            descriptors: canonical
                .into_values()
                .enumerate()
                .map(|(i, d)| (u32::try_from(i + 1).expect("bounded score"), d))
                .collect(),
            ..Self::default()
        }
    }
    /// # Errors
    /// Returns a diagnostic for invalid schema data, unsupported vocabulary or missing dictionary entries.
    pub fn token(&self, descriptor: &Descriptor) -> Result<u32> {
        self.descriptors
            .iter()
            .find_map(|(k, v)| (v == descriptor).then_some(*k))
            .ok_or_else(|| Diagnostic::new("E_EXTENSION_MAP", "descriptor absent from dictionary"))
    }
    /// # Errors
    /// Returns a diagnostic for invalid schema data, unsupported vocabulary or missing dictionary entries.
    pub fn lookup(&self, token: u32) -> Result<Option<Descriptor>> {
        if token == 0 {
            return Ok(None);
        }
        self.descriptors
            .get(&token)
            .cloned()
            .map(Some)
            .ok_or_else(|| {
                Diagnostic::new(
                    "E_EXTENSION_MAP_REQUIRED",
                    format!("unknown descriptor token {token}"),
                )
            })
    }
    /// # Errors
    /// Returns a diagnostic for invalid schema data, unsupported vocabulary or missing dictionary entries.
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 || self.descriptors.contains_key(&0) {
            return Err(Diagnostic::new(
                "E_EXTENSION_MAP",
                "unsupported version or reserved token zero",
            ));
        }
        if self.parameters.contains_key(&self.controller_index)
            || self
                .parameters
                .iter()
                .any(|(index, name)| *index != 126 || name != DYNAMIC_LEVEL)
        {
            return Err(Diagnostic::new(
                "E_EXTENSION_MAP",
                "unsupported or conflicting parameter assignment",
            ));
        }
        for d in self.descriptors.values() {
            validate_annotation(
                &serde_json::to_value(d)
                    .map_err(|e| Diagnostic::new("E_ANNOTATION", e.to_string()))?,
            )?;
            crate::rules::by_id(&d.technique)
                .ok_or_else(|| Diagnostic::new("E_TECHNIQUE", &d.technique))?;
        }
        Ok(())
    }
    /// Decode the published PE representation without a second resource model.
    /// # Errors
    /// Returns a diagnostic for malformed data, duplicate IDs or invalid assignments.
    /// # Panics
    /// Panics if the bundled schema is invalid.
    pub fn from_property_data(value: &serde_json::Value) -> Result<Self> {
        static SCHEMA: std::sync::OnceLock<jsonschema::Validator> = std::sync::OnceLock::new();
        SCHEMA
            .get_or_init(|| {
                jsonschema::validator_for(
                    &serde_json::from_str(include_str!("../mapping/refinement.schema.json"))
                        .expect("resource schema JSON"),
                )
                .expect("valid resource schema")
            })
            .validate(value)
            .map_err(|e| Diagnostic::new("E_EXTENSION_MAP", e.to_string()))?;
        let mut dictionary = Self {
            controller_index: u8::try_from(
                value["controllerIndex"].as_u64().expect("validated index"),
            )
            .expect("8-bit index"),
            ..Self::default()
        };
        for entry in value["entries"].as_array().expect("validated entries") {
            let token = u32::try_from(entry["token"].as_u64().expect("validated token"))
                .expect("32-bit token");
            let descriptor = Descriptor {
                technique: entry["technique"]
                    .as_str()
                    .expect("validated technique")
                    .into(),
                qualifiers: qualifier_values(
                    entry["qualifiers"]
                        .as_array()
                        .expect("validated qualifiers"),
                )?,
            };
            if dictionary.descriptors.insert(token, descriptor).is_some() {
                return Err(Diagnostic::new("E_EXTENSION_MAP", "duplicate token"));
            }
        }
        for parameter in value["parameters"]
            .as_array()
            .expect("validated parameters")
        {
            let index = u8::try_from(parameter["index"].as_u64().expect("validated index"))
                .expect("8-bit index");
            if dictionary
                .parameters
                .insert(
                    index,
                    parameter["id"].as_str().expect("validated URI").into(),
                )
                .is_some()
            {
                return Err(Diagnostic::new(
                    "E_EXTENSION_MAP",
                    "duplicate parameter index",
                ));
            }
        }
        dictionary.validate()?;
        Ok(dictionary)
    }
    /// PE uses ASCII camelCase property names; URIs occur as values, not keys.
    #[must_use]
    pub fn property_data(&self) -> serde_json::Value {
        serde_json::json!({"version":self.version,"controllerIndex":self.controller_index,"parameters":self.parameters.iter().map(|(index,id)|serde_json::json!({"index":index,"id":id})).collect::<Vec<_>>(),
            "entries":self.descriptors.iter().map(|(token,d)|serde_json::json!({"token":token,"technique":d.technique,
                "qualifiers":d.qualifiers.iter().map(|(id,value)|serde_json::json!({"id":id,"valueJson":value.to_string()})).collect::<Vec<_>>() })).collect::<Vec<_>>()})
    }
}
#[must_use]
pub fn ascii_json(value: &serde_json::Value) -> Vec<u8> {
    let mut out = String::new();
    for ch in value.to_string().chars() {
        if ch.is_ascii() {
            out.push(ch);
        } else {
            for unit in ch.encode_utf16(&mut [0; 2]) {
                let _ = write!(&mut out, "\\u{unit:04x}");
            }
        }
    }
    out.into_bytes()
}

/// Validate receiver-specific evidence before attaching it to a companion.
/// File conversion retains portable categories; a live adapter selects recipes.
/// # Panics
/// Panics if a bundled schema/catalog or an internally validated invariant is inconsistent.
/// # Errors
/// Returns a diagnostic for invalid schema data, unsupported vocabulary or missing dictionary entries.
pub fn validate_bindings(value: &serde_json::Value) -> Result<()> {
    static SCHEMA: std::sync::OnceLock<jsonschema::Validator> = std::sync::OnceLock::new();
    let validator = SCHEMA.get_or_init(|| {
        jsonschema::validator_for(
            &serde_json::from_str(include_str!("../mapping/binding.schema.json"))
                .expect("binding schema JSON"),
        )
        .expect("valid binding schema")
    });
    validator
        .validate(value)
        .map_err(|e| Diagnostic::new("E_BINDING", e.to_string()))?;
    if value["status"] != "verified"
        || ["playerVersion", "libraryRevision", "setupHash"]
            .iter()
            .any(|name| {
                value["target"][*name]
                    .as_str()
                    .is_none_or(|v| v.is_empty() || v.chars().all(|c| c == '0'))
            })
    {
        return Err(Diagnostic::new(
            "E_BINDING_UNVERIFIED",
            "provide a verified receiver/program binding with concrete versions and setup observation",
        ));
    }
    let mut descriptors = std::collections::BTreeSet::new();
    for entry in value["entries"].as_array().expect("validated entries") {
        let d = &entry["descriptor"];
        let qualifiers =
            qualifier_values(d["qualifiers"].as_array().expect("validated qualifiers"))?;
        let descriptor = Descriptor {
            technique: d["technique"].as_str().expect("validated technique").into(),
            qualifiers,
        };
        validate_annotation(&serde_json::to_value(&descriptor).expect("descriptor JSON"))?;
        if !descriptors.insert(serde_json::to_string(&descriptor).expect("descriptor JSON")) {
            return Err(Diagnostic::new(
                "E_BINDING",
                "ambiguous duplicate descriptor recipe",
            ));
        }
        crate::rules::Articulation {
            at: u8::try_from(entry["wire"]["at"].as_u64().expect("validated AT"))
                .expect("schema constrains the field to eight bits"),
            subclass: u8::try_from(entry["wire"]["sub"].as_u64().expect("validated subclass"))
                .expect("schema constrains the field to eight bits"),
            variation: u8::try_from(
                entry["wire"]["variation"]
                    .as_u64()
                    .expect("validated variation"),
            )
            .expect("schema constrains the field to eight bits"),
            ..Default::default()
        }
        .validate()?;
    }
    Ok(())
}

// Shared by the schema-validated PE dictionary and receiver binding records.
fn qualifier_values(records: &[serde_json::Value]) -> Result<BTreeMap<String, serde_json::Value>> {
    let mut qualifiers = BTreeMap::new();
    for q in records {
        let id = q["id"]
            .as_str()
            .ok_or_else(|| Diagnostic::new("E_QUALIFIER", "missing URI"))?;
        let json = serde_json::from_str(
            q["valueJson"]
                .as_str()
                .ok_or_else(|| Diagnostic::new("E_QUALIFIER", "missing JSON value"))?,
        )
        .map_err(|e| Diagnostic::new("E_QUALIFIER", e.to_string()))?;
        if qualifiers.insert(id.to_string(), json).is_some() {
            return Err(Diagnostic::new("E_QUALIFIER", "duplicate qualifier URI"));
        }
    }
    Ok(qualifiers)
}
