//! Composer/editor remarks are instructions to a performer, not executable code.
//! MusicXML 4.0 direction/words and editorial footnote retain distinct scopes.
//! M2-104 §7.5.9 bank 2/status 0 carries timed performance text without pretending
//! it is MIDI-CI configuration. Note attachment uses the existing descriptor map.
use crate::model::{Descriptor, Route};
use midi2::prelude::*;
use roxmltree::Node;
const PREFIX: &str = "urn:midi2workbench:direction-remark:v1:";
pub const CONTEXT: &str = "urn:midi2workbench:score-context";
pub const QUALIFIER: &str = "urn:midi2workbench:score-remarks";
#[must_use]
pub fn text(route: Option<Route>, value: &str, metadata: bool) -> Vec<u32> {
    if metadata {
        let mut message = midi2::flex_data::UnknownMetadataText::<Vec<u32>>::new();
        message.set_optional_channel(route.map(|r| u4::new(r.channel)));
        message.set_group(u4::new(route.map_or(0, |r| r.group)));
        message.set_text(value);
        message.data().to_vec()
    } else {
        let mut message = midi2::flex_data::UnknownPerformanceText::<Vec<u32>>::new();
        message.set_optional_channel(route.map(|r| u4::new(r.channel)));
        message.set_group(u4::new(route.map_or(0, |r| r.group)));
        message.set_text(value);
        message.data().to_vec()
    }
}
#[must_use]
pub fn performance(words: &[u32]) -> Option<(Option<Route>, String)> {
    let checked = validation_words(words)?;
    let message = midi2::flex_data::UnknownPerformanceText::try_from(checked.as_ref()).ok()?;
    Some((
        message.optional_channel().map(|channel| Route {
            group: message.group().into(),
            channel: channel.into(),
        }),
        scope(words)
            .and_then(|s| s["text"].as_str().map(str::to_owned))
            .or_else(|| flex_text(words))?,
    ))
}
#[must_use]
pub fn raw_direction(node: Node<'_, '_>) -> bool {
    node.descendants().any(|n| {
        n.has_tag_name("other-play") && n.attribute("type") == Some("urn:midi2workbench:ump:v1")
    })
}
/// Collect literal editorial/extended remarks, including unknown technique words.
/// Known technique names still have deterministic Rust rules; unknown prose does
/// not silently become a claim that a sample player can perform that technique.
#[must_use]
pub fn note_values(node: Node<'_, '_>) -> Vec<serde_json::Value> {
    let voice = node
        .children()
        .find(|n| n.has_tag_name("voice"))
        .and_then(|n| n.text());
    let staff = node
        .children()
        .find(|n| n.has_tag_name("staff"))
        .and_then(|n| n.text());
    node.descendants()
        .filter(Node::is_element)
        .filter(|n| {
            let technical = n.parent().is_some_and(|p| p.has_tag_name("technical"));
            match n.tag_name().name() {
                "footnote" => true,
                "other-technical" => crate::rules::term(n.text().unwrap_or("")).is_none(),
                "other-articulation" | "other-notation" | "other-ornament" | "other-dynamics" => {
                    technical || crate::rules::term(n.text().unwrap_or("")).is_none()
                }
                "up-bow" | "down-bow" | "harmonic" | "snap-pizzicato" | "string" => false,
                _ => technical,
            }
        })
        .map(|n| {
            serde_json::json!({
                "kind": n.tag_name().name(),
                "text": n.text().unwrap_or(""),
                "xml": &node.document().input_text()[n.range()],
                "type": n.attribute("type"),
                "number": n.attribute("number"),
                "smufl": n.attribute("smufl"),
                "voice": voice,
                "staff": staff,
                "language": n.attribute(("http://www.w3.org/XML/1998/namespace", "lang")),
            })
        })
        .collect()
}
pub fn attach(
    descriptor: &mut Option<Descriptor>,
    technique: &str,
    values: Vec<serde_json::Value>,
) {
    if values.is_empty() {
        return;
    }
    let d = descriptor.get_or_insert_with(|| Descriptor {
        technique: technique.into(),
        ..Default::default()
    });
    // On canonical re-import the structured original already supplies provenance.
    d.qualifiers
        .entry(QUALIFIER.into())
        .or_insert_with(|| serde_json::Value::Array(values));
}
/// Exposes text for explicit user/receiver regex bindings; the converter does not
/// execute arbitrary prose or vendor keyswitch recipes during portable conversion.
#[must_use]
pub fn literal_values(descriptor: &Descriptor) -> Vec<&str> {
    descriptor
        .qualifiers
        .get(QUALIFIER)
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|r| r["text"].as_str())
        .collect()
}

/// M2-104 has channel/time addressing but no MusicXML voice/staff identifiers.
/// This declared UTF-8 text convention retains those editorial scopes. It does
/// not add a MIDI Profile, controller, or executable articulation vocabulary.
#[must_use]
pub fn direction(
    route: Route,
    value: &str,
    kind: &str,
    voice: Option<&str>,
    staff: Option<&str>,
) -> Vec<u32> {
    if kind == "words" && voice.is_none() && staff.is_none() {
        return text(Some(route), value, false);
    }
    text(
        Some(route),
        &format!(
            "{PREFIX}{}",
            serde_json::json!({"text":value,"kind":kind,"voice":voice,"staff":staff})
        ),
        false,
    )
}
#[must_use]
pub fn scope(words: &[u32]) -> Option<serde_json::Value> {
    let checked = validation_words(words)?;
    let message = midi2::flex_data::UnknownPerformanceText::try_from(checked.as_ref()).ok()?;
    let _ = message;
    serde_json::from_str(flex_text(words)?.strip_prefix(PREFIX)?).ok()
}
#[must_use]
pub fn metadata(words: &[u32]) -> Option<String> {
    let checked = validation_words(words)?;
    midi2::flex_data::UnknownMetadataText::try_from(checked.as_ref()).ok()?;
    flex_text(words)
}

/// Read all twelve text bytes per packet. Upstream 0.11.1 truncates a full final
/// payload byte; musical identities and UTF-8 cannot tolerate that loss.
#[must_use]
pub fn flex_text(words: &[u32]) -> Option<String> {
    if words.is_empty()
        || !words.len().is_multiple_of(4)
        || words[0] >> 28 != 0xd
        || (words[0] >> 8).trailing_zeros() >= 8
    {
        return None;
    }
    let mut bytes: Vec<u8> = words
        .chunks_exact(4)
        .flat_map(|p| p[1..].iter().flat_map(|w| w.to_be_bytes()))
        .collect();
    while bytes.last() == Some(&0) {
        bytes.pop();
    }
    String::from_utf8(bytes).ok()
}

/// Validate original packet sequencing and UTF-8, then work around upstream's
/// final-byte iterator defect with an additional zero-filled validation packet.
/// The original bytes remain authoritative and are never rewritten in the file.
pub(crate) fn validation_words(words: &[u32]) -> Option<std::borrow::Cow<'_, [u32]>> {
    let &head = words.first()?;
    if head >> 28 != 0xd || (head >> 8).trailing_zeros() >= 8 || !words.len().is_multiple_of(4) {
        return None;
    }
    let count = words.len() / 4;
    for (i, p) in words.chunks_exact(4).enumerate() {
        let form = if count == 1 {
            0
        } else if i == 0 {
            1
        } else if i + 1 == count {
            3
        } else {
            2
        };
        if (p[0] >> 22) & 3 != form || p[0] & !0x00c0_0000 != head & !0x00c0_0000 {
            return None;
        }
    }
    flex_text(words)?;
    if words.last().copied().unwrap_or(0).trailing_zeros() >= 8 {
        return Some(std::borrow::Cow::Borrowed(words));
    }
    let mut padded = words.to_vec();
    let at = words.len() - 4;
    padded[at] = (padded[at] & !0x00c0_0000) | if count == 1 { 0x0040_0000 } else { 0x0080_0000 };
    padded.extend([(head & !0x00c0_0000) | 0x00c0_0000, 0, 0, 0]);
    Some(std::borrow::Cow::Owned(padded))
}
