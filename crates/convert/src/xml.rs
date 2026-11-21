//! musicxml is the typed score parser/writer. The lexical envelope retains exact
//! decimals and source data that its floating-point datatypes cannot preserve.
use crate::{
    Diagnostic, Result,
    model::{
        Descriptor, Event, Note, Performance, Rational, RenderingPolicy, Route, decimal, round,
    },
    rules::{self, Articulation},
    wire,
};
use musicxml::elements::{AudibleType, GraceType, MeasureElement, NoteType, PartElement};
use musicxml_internal::{ElementDeserializer, XmlElement};
use roxmltree::{Document, Node, ParsingOptions};
use std::fmt::Write as _;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Read, Write},
};

const MAX_XML: u64 = 64 * 1024 * 1024;
fn err(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("E_MUSICXML", message)
}
// Paths identify actual lexical nodes, including timewise measure/part order.
fn xml_path(element: Node<'_, '_>) -> String {
    let mut components = Vec::new();
    for n in element.ancestors().filter(Node::is_element) {
        let name = n.tag_name().name();
        let component = if name == "part" {
            format!("part[@id='{}']", n.attribute("id").unwrap_or(""))
        } else if n.parent().is_some_and(|p| p.is_root()) {
            name.to_string()
        } else {
            format!(
                "{name}[{}]",
                n.prev_siblings().filter(|p| p.has_tag_name(name)).count()
            )
        };
        components.push(component);
    }
    components.reverse();
    format!("/{}", components.join("/"))
}
fn mute_slot(value: &str, piano: bool) -> Result<u8> {
    // M2-126 §8.3.12 is a piano muffler/sordino switch, not a mechanism
    // accepting brass mute identities. MusicXML's generic on/off/practice has
    // a plausible piano counterpart; an authored straight/cup/Harmon does not.
    if piano && !matches!(value, "on" | "off" | "practice") {
        return Err(Diagnostic::new(
            "E_PIANO_MUTE",
            format!("{value}: not a defined piano mute mechanism"),
        ));
    }
    Ok(match value {
        "off" => 0,
        "on" | "straight" => 1,
        "practice" => 2,
        "cup" => 3,
        "harmon-stem" => 4,
        "harmon-no-stem" => 6,
        "plunger" => 7,
        "bucket" => 8,
        "solotone" => 10,
        "whisper" => 11,
        "hat" => 12,
        "stop-hand" => 13,
        "stop-mute" => 14,
        _ => return Err(Diagnostic::new("E_MUTE_BINDING", value)),
    })
}
// Borrow source terms; no duplicate notation or span model is needed.
fn notation_terms<'a>(root: Node<'a, 'a>) -> Result<BTreeMap<usize, Vec<&'a str>>> {
    let mut active = BTreeMap::new();
    let mut result = BTreeMap::new();
    let notes = root
        .descendants()
        .filter(|n| n.has_tag_name("note"))
        .collect::<Vec<_>>();
    let mut index = 0;
    while index < notes.len() {
        let start = index;
        index += 1;
        while index < notes.len()
            && source_part(notes[index]) == source_part(notes[start])
            && notes[index].parent() == notes[start].parent()
            && child(notes[index], "chord").is_some()
        {
            index += 1;
        }
        let chord = &notes[start..index];
        // A stop on any chord member closes the interval for every attack at
        // that onset; XML member order cannot choose which sounds are tagged.
        for kind in ["stop", "start"] {
            for &n in chord {
                let context = (
                    source_part(n),
                    text(n, "staff").unwrap_or("1"),
                    text(n, "voice").unwrap_or("1"),
                );
                for other in n.descendants().filter(|n| n.has_tag_name("other-notation")) {
                    let term = other.text().unwrap_or("");
                    if rules::term(term).is_none() {
                        continue;
                    }
                    let action = other.attribute("type").unwrap_or("single");
                    if !["start", "stop", "single"].contains(&action) {
                        return Err(err("invalid articulation span type"));
                    }
                    if action != kind {
                        continue;
                    }
                    let key = (
                        context,
                        other.attribute("number").unwrap_or("1"),
                        rules::normalize(term),
                    );
                    if kind == "stop" {
                        if active.remove(&key).is_none() {
                            return Err(err("articulation span stop without start"));
                        }
                    } else if active.insert(key, term).is_some() {
                        return Err(err("duplicate articulation span start"));
                    }
                }
            }
        }
        for &n in chord {
            let context = (
                source_part(n),
                text(n, "staff").unwrap_or("1"),
                text(n, "voice").unwrap_or("1"),
            );
            let mut terms = active
                .iter()
                .filter(|((scope, ..), _)| *scope == context)
                .map(|(_, term)| *term)
                .collect::<Vec<_>>();
            terms.extend(
                n.descendants()
                    .filter(|n| {
                        n.has_tag_name("other-notation")
                            && n.attribute("type").unwrap_or("single") == "single"
                            && rules::term(n.text().unwrap_or("")).is_some()
                    })
                    .map(|n| n.text().unwrap_or("")),
            );
            if !terms.is_empty() {
                result.insert(n.range().start, terms);
            }
        }
    }
    if !active.is_empty() {
        return Err(err("unclosed articulation span"));
    }
    Ok(result)
}

fn child<'a, 'b>(element_node: Node<'a, 'b>, name: &str) -> Option<Node<'a, 'b>> {
    element_node.children().find(|c| c.has_tag_name(name))
}
fn text<'a>(element_node: Node<'a, 'a>, name: &str) -> Option<&'a str> {
    child(element_node, name).and_then(|n| n.text())
}
fn number(element_node: Node<'_, '_>, name: &str, default: Rational) -> Result<Rational> {
    text(element_node, name).map_or(Ok(default), decimal)
}
fn attr_number(element_node: Node<'_, '_>, name: &str, default: Rational) -> Result<Rational> {
    element_node.attribute(name).map_or(Ok(default), decimal)
}
fn zero() -> Rational {
    Rational::from_integer(0)
}
fn integer(v: i64) -> Rational {
    Rational::from_integer(v)
}
fn document(source: &str) -> Result<Document<'_>> {
    if source.contains("<!ENTITY") {
        return Err(err(
            "entity declarations are unsupported; external fetching is disabled",
        ));
    }
    Document::parse_with_options(
        source,
        ParsingOptions {
            allow_dtd: true,
            nodes_limit: 2_000_000,
        },
    )
    .map_err(|e| err(e.to_string()))
}

// Upstream's 1.1.2 tokenizer only recognizes double-quoted attributes. Feed its
// public typed deserializer with the already validated lexical tree instead.
// This also avoids its tokenizer's entity/attribute-whitespace discrepancies.
// Upstream 1.1.2 models decimal semitones/divisions as integers and its Vec
// deserializers discard failed children. Use structural numeric placeholders only
// in that typed view; all musical numbers are read exactly from the lexical tree.
fn typed_element(element_node: Node<'_, '_>) -> Result<XmlElement> {
    let name = element_node.tag_name().name();
    let mut text = element_node
        .children()
        .filter(Node::is_text)
        .filter_map(|n| n.text())
        .collect::<String>()
        .trim()
        .to_string();
    if ["alter", "chromatic", "divisions", "duration", "offset"].contains(&name)
        && text.parse::<i16>().is_err()
    {
        decimal(&text)?;
        text = "1".into();
    }
    if name == "duration"
        && text == "0"
        && element_node
            .parent()
            .is_some_and(|p| p.has_tag_name("forward") || p.has_tag_name("backup"))
    {
        text = "1".into();
    }
    let attributes = element_node
        .attributes()
        .map(|a| {
            let mut value = a.value().to_string();
            if ["attack", "release", "make-time", "divisions"].contains(&a.name())
                && value.parse::<i32>().is_err()
            {
                decimal(&value)?;
                value = "0".into();
            }
            if a.name() == "time-only" {
                value = value
                    .split(',')
                    .map(str::trim)
                    .collect::<Vec<_>>()
                    .join(",");
            }
            Ok((a.name().into(), value))
        })
        .collect::<Result<Vec<_>>>()?;
    let element = XmlElement {
        name: name.into(),
        attributes,
        elements: element_node
            .children()
            .filter(Node::is_element)
            .map(typed_element)
            .collect::<Result<Vec<_>>>()?,
        text,
    };
    if [
        "note",
        "direction",
        "sound",
        "attributes",
        "backup",
        "forward",
        "barline",
    ]
    .contains(&name)
    {
        musicxml::elements::MeasureElement::deserialize(&element)
            .map_err(|e| Diagnostic::new("E_TYPED_ELEMENT", format!("{name}: {e}")))?;
    }
    Ok(element)
}

// Rearrange the upstream lexical container directly. Serializing a temporary
// typed score would round its numeric fields and leave text entities unescaped.
fn partwise_tree(mut tree: XmlElement) -> Result<XmlElement> {
    if tree.name != "score-timewise" {
        return Ok(tree);
    }
    musicxml::elements::ScoreTimewise::deserialize(&tree).map_err(err)?;
    let order = tree
        .elements
        .iter()
        .find(|e| e.name == "part-list")
        .into_iter()
        .flat_map(|e| &e.elements)
        .filter(|e| e.name == "score-part")
        .filter_map(|e| {
            e.attributes
                .iter()
                .find(|(k, _)| k == "id")
                .map(|(_, v)| v.clone())
        })
        .collect::<Vec<_>>();
    let mut parts = BTreeMap::<String, XmlElement>::new();
    let mut metadata = Vec::new();
    for measure in tree.elements {
        if measure.name != "measure" {
            metadata.push(measure);
            continue;
        }
        for part in measure.elements {
            if part.name != "part" {
                return Err(err("unexpected timewise measure element"));
            }
            let id = part
                .attributes
                .iter()
                .find(|(k, _)| k == "id")
                .map(|(_, v)| v.clone())
                .ok_or_else(|| err("timewise part without id"))?;
            let destination = parts.entry(id).or_insert_with(|| XmlElement {
                name: "part".into(),
                attributes: part.attributes,
                elements: Vec::new(),
                text: String::new(),
            });
            destination.elements.push(XmlElement {
                name: "measure".into(),
                attributes: measure.attributes.clone(),
                elements: part.elements,
                text: String::new(),
            });
        }
    }
    for id in order {
        metadata.push(
            parts
                .remove(&id)
                .ok_or_else(|| err("listed timewise part has no measures"))?,
        );
    }
    if !parts.is_empty() {
        return Err(err("unlisted timewise part"));
    }
    tree.name = "score-partwise".into();
    tree.elements = metadata;
    Ok(tree)
}

/// # Errors
/// Returns a diagnostic for malformed archives/XML or playback semantics that cannot be resolved.
pub fn extract(bytes: &[u8]) -> Result<String> {
    if bytes.len() as u64 > MAX_XML {
        return Err(err("input exceeds 64 MiB limit"));
    }
    let data = if bytes.starts_with(b"PK\x03\x04") {
        let mut archive =
            zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| err(e.to_string()))?;
        if archive.len() > 4096 {
            return Err(err("too many archive members"));
        }
        let mut total = 0_u64;
        for i in 0..archive.len() {
            let f = archive.by_index(i).map_err(|e| err(e.to_string()))?;
            total = total
                .checked_add(f.size())
                .ok_or_else(|| err("archive size overflow"))?;
            if total > MAX_XML {
                return Err(err("expanded archive exceeds 64 MiB"));
            }
        }
        let mut container = String::new();
        archive
            .by_name("META-INF/container.xml")
            .map_err(|e| err(e.to_string()))?
            .take(MAX_XML + 1)
            .read_to_string(&mut container)
            .map_err(|e| err(e.to_string()))?;
        let d = document(&container)?;
        let roots = d
            .descendants()
            .filter(|n| {
                n.has_tag_name("rootfile")
                    && n.attribute("media-type")
                        .is_none_or(|t| t == "application/vnd.recordare.musicxml+xml")
            })
            .collect::<Vec<_>>();
        if roots.len() != 1 {
            return Err(err("select exactly one MusicXML root"));
        }
        let path = roots[0]
            .attribute("full-path")
            .ok_or_else(|| err("rootfile has no full-path"))?;
        if path.starts_with('/') || path.split('/').any(|c| c == "..") {
            return Err(err("unsafe rootfile path"));
        }
        let mut data = Vec::new();
        archive
            .by_name(path)
            .map_err(|e| err(e.to_string()))?
            .take(MAX_XML + 1)
            .read_to_end(&mut data)
            .map_err(|e| err(e.to_string()))?;
        data
    } else {
        bytes.to_vec()
    };
    let (encoding, skip) = encoding_rs::Encoding::for_bom(&data).unwrap_or((encoding_rs::UTF_8, 0));
    let (decoded, errors) = encoding.decode_without_bom_handling(&data[skip..]);
    if errors {
        return Err(err("invalid XML character encoding"));
    }
    let mut source = decoded.into_owned();
    if source.starts_with("<?xml") {
        if let Some(end) = source.find("?>") {
            source.replace_range(..end + 2, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
        }
    }
    Ok(source)
}

type PlaybackSetting<'a> = (
    Rational,
    Route,
    Option<String>,
    Option<String>,
    Node<'a, 'a>,
);
#[derive(Default, Clone)]
struct Measure<'a> {
    navigation: Vec<(Rational, Node<'a, 'a>)>,
    settings: Vec<PlaybackSetting<'a>>,
    notes: Vec<(Note, String, bool, bool, Vec<u8>)>,
    event_passes: BTreeMap<usize, Vec<u32>>,
    events: Vec<Event>,
    length: Rational,
    repeat_start: bool,
    repeat_times: Option<u32>,
    ending: BTreeSet<u32>,
    ending_stop: bool,
    holds: Vec<(Rational, Rational)>,
}

/// Normalize typed parts while retaining exact lexical values at each source path.
/// # Panics
/// Panics if a bundled schema/catalog or an internally validated invariant is inconsistent.
/// # Errors
/// Returns a diagnostic for malformed archives/XML or playback semantics that cannot be resolved.
#[allow(
    clippy::too_many_lines,
    reason = "Keep the stateful traversal together; semantic rules are delegated without duplicating transient models."
)]
pub fn read(bytes: &[u8], policy: &RenderingPolicy) -> Result<(String, Performance)> {
    let source = extract(bytes)?;
    let lexical = document(&source)?;
    let root = lexical.root_element();
    if !["score-partwise", "score-timewise"].contains(&root.tag_name().name()) {
        return Err(err("expected a partwise or timewise score"));
    }
    // Collection deserializers may discard invalid optional children. Validate
    // authored playback values before handing any tree to the upstream parser.
    for n in root.descendants().filter(Node::is_element) {
        if n.has_tag_name("sound") {
            if let Some(raw) = n.attribute("dynamics") {
                let level = decimal(raw)? * Rational::new(9, 10);
                if level < zero() {
                    return Err(Diagnostic::new(
                        "E_INPUT_DYNAMIC",
                        format!("sound@dynamics={raw} is negative"),
                    ));
                }
                velocity(level)?;
            }
        }
        if n.has_tag_name("midi-program") {
            let value = n
                .text()
                .unwrap_or("")
                .parse::<u16>()
                .map_err(|_| Diagnostic::new("E_INPUT_PROGRAM", "invalid midi-program"))?;
            if !(1..=128).contains(&value) {
                return Err(Diagnostic::new(
                    "E_INPUT_PROGRAM",
                    "midi-program outside 1..128",
                ));
            }
        }
    }
    let typed_tree = partwise_tree(typed_element(root)?)?;
    let typed = musicxml::elements::ScorePartwise::deserialize(&typed_tree).map_err(err)?;
    let mut out = Performance::default();
    out.report.policy.clone_from(&policy.version);
    for n in root.descendants().filter(Node::is_element) {
        *out.report
            .source_constructs
            .entry(n.tag_name().name().into())
            .or_default() += 1;
    }
    for n in root.descendants().filter(Node::is_element) {
        for attribute in n.attributes() {
            *out.report
                .source_constructs
                .entry(format!("{}@{}", n.tag_name().name(), attribute.name()))
                .or_default() += 1;
        }
    }
    for (construct, count) in out.report.source_constructs.clone() {
        let tag = construct.split('@').next().expect("construct name");
        let disposition = match tag {
            "pitch" | "step" | "alter" | "octave" | "duration" | "divisions" | "backup"
            | "forward" | "transpose" | "chromatic" | "diatonic" | "octave-change" | "rest"
            | "cue" | "grace" | "tie" | "tied" | "chord" => {
                "timed realization; lexical precision retained; written notation archived"
            }
            "sound" | "midi-instrument" | "midi-channel" | "midi-device" | "midi-port"
            | "midi-bank" | "midi-program" | "midi-unpitched" | "volume" | "pan" | "time"
            | "beats" | "beat-type" | "pedal" | "swing" | "first" | "second" | "swing-type"
            | "straight" | "repeat" | "ending" | "segno" | "coda" => {
                "playback state/routing/navigation interpretation; layout archived"
            }
            "notations" | "articulations" | "technical" | "ornaments" | "play" | "other-play"
            | "other-technical" | "other-articulation" | "other-notation" | "dynamics"
            | "wedge" | "words" | "slur" | "arpeggiate" | "fermata" | "breath-mark" | "caesura" => {
                "semantic rule/policy traversal; specific effects recorded separately; source glyph/layout archived"
            }
            _ => {
                "source structure/notation archived; no separate playback claim for this construct"
            }
        };
        out.report.record(
            format!("construct:{construct}"),
            "source disposition",
            format!("{count} occurrence(s): {disposition}"),
        );
    }
    out.report.record(
        "/",
        "MusicXML version",
        format!(
            "{} normalized through musicxml 1.1.2",
            root.attribute("version")
                .unwrap_or("1.0 (MusicXML default)")
        ),
    );
    let has_raw_ump = root.descendants().any(|n| {
        n.has_tag_name("other-play") && n.attribute("type") == Some("urn:midi2workbench:ump:v1")
    });
    if !has_raw_ump {
        for n in root
            .descendants()
            .filter(|n| n.has_tag_name("miscellaneous-field") || n.has_tag_name("credit-words"))
        {
            if let Some(value) = n.text().filter(|v| !v.is_empty()) {
                out.configuration.push(Event {
                    order: 0,
                    time: zero(),
                    words: crate::remarks::text(None, value, true),
                });
                out.report.record(xml_path(n),"score-level text","M2-104 §7.5.9 Unknown Metadata Text; original MusicXML role retained in companion");
            }
        }
    }
    let mut parts = Vec::new();
    let mut instrument_routes = BTreeMap::new();
    let mut profile_bindings = BTreeMap::new();
    let has_instrument_metadata = root
        .descendants()
        .filter(|n| {
            n.has_tag_name("other-play") && n.attribute("type") == Some("urn:midi2workbench:ump:v1")
        })
        .filter_map(|n| serde_json::from_str::<serde_json::Value>(n.text().unwrap_or("")).ok())
        .any(|v| {
            serde_json::from_value::<Vec<u32>>(v["words"].clone())
                .ok()
                .is_some_and(|w| wire::instrument_from_text(&w).is_some())
        });
    let mut made_time = BTreeMap::<String, (Rational, Rational, Rational)>::new();
    let mut note_nodes = BTreeMap::new();
    let mut note_positions = BTreeMap::new();
    let trill_sources = trill_sources(root);
    let notation_terms = notation_terms(root)?;
    let mut note_scopes = BTreeMap::<String, (String, String, bool, bool)>::new();
    for (index, part) in typed.content.part.iter().enumerate() {
        if index >= 256 {
            return Err(Diagnostic::new("E_ROUTE", "more than 256 logical parts"));
        }
        let id = &part.attributes.id.0;
        let lexical_measures = if root.has_tag_name("score-partwise") {
            let p = root
                .children()
                .find(|n| n.has_tag_name("part") && n.attribute("id") == Some(id))
                .ok_or_else(|| err("typed/lexical part mismatch"))?;
            p.children()
                .filter(|n| n.has_tag_name("measure"))
                .collect::<Vec<_>>()
        } else {
            root.children()
                .filter(|n| n.has_tag_name("measure"))
                .filter_map(|m| {
                    m.children()
                        .find(|p| p.has_tag_name("part") && p.attribute("id") == Some(id))
                })
                .collect::<Vec<_>>()
        };
        // Preserve note membership when a remark names a voice/staff; UMP has
        // no such address. Do not add redundant context to every ordinary score.
        let has_scoped_remarks = lexical_measures.iter().any(|m| {
            m.descendants().any(|n| {
                n.has_tag_name("direction")
                    && (child(n, "voice").is_some() || child(n, "staff").is_some())
                    && n.descendants()
                        .any(|d| ["words", "rehearsal", "footnote"].contains(&d.tag_name().name()))
            })
        });
        let mut measures = Vec::new();
        let mut divisions = integer(1);
        let mut dynamic = integer(72);
        let mut transpose = zero();
        let score_part = root
            .descendants()
            .find(|n| n.has_tag_name("score-part") && n.attribute("id") == Some(id));
        let declared_channel = score_part
            .and_then(|n| n.descendants().find(|n| n.has_tag_name("midi-channel")))
            .and_then(|n| n.text())
            .and_then(|s| s.parse::<u8>().ok());
        let declared_port = score_part
            .and_then(|n| {
                n.descendants()
                    .find(|n| n.has_tag_name("midi-device") && n.attribute("id").is_none())
            })
            .and_then(|n| n.attribute("port"))
            .and_then(|s| s.parse::<u8>().ok());
        let route = Route {
            group: declared_port
                .map_or(u8::try_from(index / 16).expect("bounded part count"), |p| {
                    p.saturating_sub(1)
                }),
            channel: declared_channel
                .map_or(u8::try_from(index % 16).expect("channel remainder"), |c| {
                    c.saturating_sub(1)
                }),
        };
        if route.group > 15 || route.channel > 15 {
            return Err(err("declared route out of range"));
        }
        if let Some(score_part) = score_part {
            for instrument in score_part
                .children()
                .filter(|n| n.has_tag_name("score-instrument") || n.has_tag_name("midi-instrument"))
            {
                if let Some(id) = instrument.attribute("id") {
                    let channel = text(instrument, "midi-channel")
                        .map(|v| v.parse::<u8>().map_err(|_| err("invalid MIDI channel")))
                        .transpose()?
                        .unwrap_or(route.channel + 1);
                    if !(1..=16).contains(&channel) {
                        return Err(err("MIDI channel outside 1..16"));
                    }
                    let port = score_part
                        .children()
                        .find(|n| n.has_tag_name("midi-device") && n.attribute("id") == Some(id))
                        .and_then(|n| n.attribute("port"))
                        .map(|v| v.parse::<u8>().map_err(|_| err("invalid MIDI port")))
                        .transpose()?
                        .unwrap_or(route.group + 1);
                    if !(1..=16).contains(&port) {
                        return Err(err("MIDI port outside 1..16"));
                    }
                    instrument_routes.insert(
                        id.to_string(),
                        Route {
                            channel: channel - 1,
                            group: port - 1,
                        },
                    );
                }
            }
        }
        let part_routes = score_part
            .into_iter()
            .flat_map(|s| s.children())
            .filter(|n| n.has_tag_name("score-instrument") || n.has_tag_name("midi-instrument"))
            .filter_map(|n| n.attribute("id"))
            .filter_map(|id| instrument_routes.get(id).copied())
            .chain(std::iter::once(route))
            .collect::<BTreeSet<_>>();
        let part_name = score_part.and_then(|n| text(n, "part-name")).unwrap_or("");
        for &target in &part_routes {
            let instruments = score_part
                .into_iter()
                .flat_map(|p| p.children())
                .filter(|n| {
                    n.has_tag_name("score-instrument")
                        && n.attribute("id")
                            .and_then(|id| instrument_routes.get(id))
                            .is_some_and(|r| *r == target)
                })
                .collect::<Vec<_>>();
            let mut bindings = Vec::new();
            for instrument in instruments
                .iter()
                .copied()
                .map(Some)
                .chain(instruments.is_empty().then_some(None))
            {
                let instrument_name = instrument
                    .and_then(|n| text(n, "instrument-name"))
                    .unwrap_or("");
                let sound_id = instrument
                    .and_then(|n| text(n, "instrument-sound"))
                    .unwrap_or("");
                let is_piano = crate::piano::identifies(part_name, instrument_name, sound_id);
                if profile_bindings
                    .insert(target, is_piano)
                    .is_some_and(|old| old != is_piano)
                {
                    return Err(Diagnostic::new(
                        "E_PROFILE_ROUTE",
                        "piano and non-piano instruments share one channel; an explicit route split is required",
                    ));
                }
                if is_piano {
                    out.piano_routes.insert(target);
                }
                let identity = if (!sound_id.is_empty()
                    || instruments.len() > 1
                    || ["", "MusicXML Part", "Part"].contains(&part_name))
                    && !instrument_name.is_empty()
                {
                    instrument_name
                } else {
                    part_name
                };
                if !bindings.contains(&identity) {
                    bindings.push(identity);
                }
                out.report.record(format!("/score-partwise/part-list/score-part[@id='{id}']"),"instrument/profile interpretation",format!("{target:?}: {part_name:?}, instrument {instrument_name:?}, sound {sound_id:?}: {}",if is_piano{"M2-126 acoustic Piano; M2-123 added only when its attributes/controllers are required"}else{"M2-123 articulation semantics; no instrument substitution"}));
            }
            if !has_instrument_metadata {
                let identity = bindings.join(" / ");
                if !identity.is_empty()
                    && identity
                        .strip_prefix("Part ")
                        .is_none_or(|n| n.parse::<usize>().is_err())
                {
                    out.configuration.push(Event {
                        order: 0,
                        time: zero(),
                        words: wire::instrument_text(
                            target,
                            &identity,
                            out.piano_routes.contains(&target),
                        ),
                    });
                }
            }
        }
        let mut slurs = BTreeSet::new();
        let mut grace_by_voice = BTreeMap::<String, Vec<(Note, Node<'_, '_>)>>::new();
        for (mi, tm) in part
            .content
            .iter()
            .filter_map(|el| {
                if let PartElement::Measure(m) = el {
                    Some(m)
                } else {
                    None
                }
            })
            .enumerate()
        {
            let lm = *lexical_measures
                .get(mi)
                .ok_or_else(|| err("typed/lexical measure mismatch"))?;
            let path = xml_path(lm);
            let mut measure = Measure::default();
            if mi == 0 {
                for instrument in score_part
                    .into_iter()
                    .flat_map(|s| s.children())
                    .filter(|n| n.has_tag_name("midi-instrument"))
                {
                    let target = instrument
                        .attribute("id")
                        .and_then(|id| instrument_routes.get(id))
                        .copied()
                        .unwrap_or(route);
                    for words in instrument_messages(instrument, target, &mut out.report, &path)? {
                        out.configuration.push(Event {
                            order: 0,
                            time: zero(),
                            words,
                        });
                    }
                }
            }
            let mut cursor = zero();
            let mut previous = zero();
            let mut counters = BTreeMap::<&str, usize>::new();
            for element in &tm.content {
                let name = match element {
                    MeasureElement::Note(_) => "note",
                    MeasureElement::Attributes(_) => "attributes",
                    MeasureElement::Backup(_) => "backup",
                    MeasureElement::Forward(_) => "forward",
                    MeasureElement::Direction(_) => "direction",
                    MeasureElement::Sound(_) => "sound",
                    MeasureElement::Barline(_) => "barline",
                    _ => continue,
                };
                let count = counters.entry(name).or_default();
                let element_node = lm
                    .children()
                    .filter(|n| n.has_tag_name(name))
                    .nth(*count)
                    .ok_or_else(|| err("typed/lexical element mismatch"))?;
                *count += 1;
                let source_path = format!("{path}/{name}[{count}]");
                let phase_start = measure.events.len();
                match element {
                    MeasureElement::Attributes(_) => {
                        divisions = number(element_node, "divisions", divisions)?;
                        if divisions <= zero() {
                            return Err(err("divisions must be positive"));
                        }
                        if let Some(tr) = child(element_node, "transpose") {
                            transpose = number(tr, "chromatic", zero())?
                                + number(tr, "octave-change", zero())? * integer(12);
                        }
                        if let Some(time) = child(element_node, "time") {
                            if let (Some(b), Some(unit)) =
                                (text(time, "beats"), text(time, "beat-type"))
                            {
                                let beats = b
                                    .split('+')
                                    .map(str::parse::<u16>)
                                    .collect::<std::result::Result<Vec<_>, _>>()
                                    .map_err(|_| err("invalid beats"))?
                                    .iter()
                                    .sum::<u16>();
                                let unit =
                                    unit.parse::<u32>().map_err(|_| err("invalid beat-type"))?;
                                if unit.is_power_of_two() && beats <= 255 {
                                    measure.events.push(Event {
                                        order: 0,
                                        time: cursor,
                                        words: wire::meter(
                                            u8::try_from(beats).expect("validated numerator"),
                                            u8::try_from(unit.trailing_zeros())
                                                .expect("32-bit exponent"),
                                        ),
                                    });
                                } else {
                                    out.report.record(
                                        &source_path,
                                        "meter",
                                        "structured metadata: nonrepresentable meter",
                                    );
                                }
                            }
                        }
                        out.report.record(
                            &source_path,
                            "attributes",
                            "score context and notation preserved",
                        );
                    }
                    MeasureElement::Backup(_) => {
                        let duration = number(element_node, "duration", zero())? / divisions;
                        if duration < zero() {
                            return Err(err("backup duration must be nonnegative"));
                        }
                        cursor -= duration;
                        if cursor < zero() {
                            return Err(err(format!("backup precedes measure at {source_path}")));
                        }
                    }
                    MeasureElement::Forward(_) => {
                        let duration = number(element_node, "duration", zero())? / divisions;
                        if duration < zero() {
                            return Err(err("forward duration must be nonnegative"));
                        }
                        cursor += duration;
                        measure.length = measure.length.max(cursor);
                    }
                    MeasureElement::Direction(_) | MeasureElement::Sound(_) => {
                        let event_start = measure.events.len();
                        let offset = playback_offset(element_node, zero())? / divisions;
                        let time = cursor + offset;
                        let scope_voice = text(element_node, "voice").map(str::to_string);
                        let scope_staff = text(element_node, "staff").map(str::to_string);
                        if !crate::remarks::raw_direction(element_node) {
                            for remark in element_node.descendants().filter(|n| {
                                n.has_tag_name("words")
                                    || n.has_tag_name("rehearsal")
                                    || n.has_tag_name("footnote")
                            }) {
                                if let Some(value) = remark.text().filter(|v| !v.is_empty()) {
                                    for &target in &part_routes {
                                        measure.events.push(Event {
                                            order: 0,
                                            time,
                                            words: crate::remarks::direction(
                                                target,
                                                value,
                                                remark.tag_name().name(),
                                                scope_voice.as_deref(),
                                                scope_staff.as_deref(),
                                            ),
                                        });
                                    }
                                    let term = rules::normalize(value);
                                    if matches!(
                                        term.as_str(),
                                        "una corda" | "tre corde" | "tutte le corde"
                                    ) {
                                        for &target in &part_routes {
                                            if out.piano_routes.contains(&target) {
                                                measure.events.push(Event {
                                                    order: 0,
                                                    time,
                                                    words: wire::cc(
                                                        target,
                                                        67,
                                                        if term == "una corda" {
                                                            u32::MAX
                                                        } else {
                                                            0
                                                        },
                                                    ),
                                                });
                                            }
                                        }
                                    }
                                    out.report.record(xml_path(remark),"timed performer text","M2-104 §7.5.9 Unknown Performance Text; channel address and score time retained; voice/staff and editorial role retained by declared text convention");
                                }
                            }
                        }
                        for sound in element_node
                            .descendants()
                            .filter(|n| n.has_tag_name("sound"))
                        {
                            for other in
                                sound.descendants().filter(|n| n.has_tag_name("other-play"))
                            {
                                if other.attribute("type") == Some("urn:midi2workbench:ump:v1") {
                                    let value: serde_json::Value =
                                        serde_json::from_str(other.text().unwrap_or(""))
                                            .map_err(|e| err(e.to_string()))?;
                                    let words: Vec<u32> = serde_json::from_value(
                                        value.get("words").unwrap_or(&value).clone(),
                                    )
                                    .map_err(|e| err(e.to_string()))?;
                                    let configuration = value
                                        .get("configuration")
                                        .map(|v| {
                                            v.as_bool().ok_or_else(|| {
                                                err("configuration flag must be boolean")
                                            })
                                        })
                                        .transpose()?
                                        .unwrap_or(false);
                                    wire::validate_event(&words, configuration)?;
                                    let event = Event {
                                        order:value.get("order").map(|v|v.as_u64().and_then(|x|u32::try_from(x).ok()).ok_or_else(||err("order must be a 32-bit unsigned integer"))).transpose()?.unwrap_or(0),
                                        time: cursor
                                            + playback_offset(sound,offset*divisions)?
                                                / divisions,
                                        words,
                                    };
                                    if configuration {
                                        out.configuration.push(event);
                                    } else {
                                        measure.events.push(event);
                                    }
                                } else if other.attribute("type")
                                    != Some(crate::extension::XML_TYPE)
                                {
                                    return Err(Diagnostic::new(
                                        "E_PLAYBACK_EXTENSION",
                                        &source_path,
                                    ));
                                }
                            }
                            let stime =
                                cursor + playback_offset(sound, offset * divisions)? / divisions;
                            if ["segno", "coda", "dacapo", "dalsegno", "tocoda", "fine"]
                                .iter()
                                .any(|a| sound.attribute(*a).is_some())
                            {
                                measure.navigation.push((stime, sound));
                                out.report.record(&source_path,"navigation","MusicXML sound labels and jumps rendered in the bounded occurrence graph");
                            }
                            for &target in &part_routes {
                                measure.settings.push((stime, target, None, None, sound));
                            }
                            for instrument in sound
                                .children()
                                .filter(|n| n.has_tag_name("midi-instrument"))
                            {
                                let target = instrument
                                    .attribute("id")
                                    .and_then(|id| instrument_routes.get(id))
                                    .copied()
                                    .unwrap_or(route);
                                for words in instrument_messages(
                                    instrument,
                                    target,
                                    &mut out.report,
                                    &source_path,
                                )? {
                                    measure.events.push(Event {
                                        order: 0,
                                        time: stime,
                                        words,
                                    });
                                }
                            }
                            if let Some(tempo) = sound.attribute("tempo") {
                                let bpm = decimal(tempo)?;
                                if bpm <= zero() {
                                    return Err(err("tempo must be positive"));
                                }
                                measure.events.push(Event {
                                    order: 0,
                                    time: stime,
                                    words: tempo_words(bpm, &mut out.report, &source_path)?,
                                });
                            }
                            if let Some(d) = sound.attribute("dynamics") {
                                let level = decimal(d)? * Rational::new(9, 10);
                                velocity(level).map_err(|_|Diagnostic::new("E_INPUT_DYNAMIC",format!("{source_path}/sound@dynamics={d} is outside the MusicXML nonnegative playback range")))?;
                                if transient_kind(element_node).is_none() {
                                    dynamic = level;
                                }
                            }
                            for (a, cc) in [
                                ("damper-pedal", 64),
                                ("sostenuto-pedal", 66),
                                ("soft-pedal", 67),
                            ] {
                                if let Some(value) = sound.attribute(a) {
                                    let amount = match value {
                                        "yes" => u32::MAX,
                                        "no" => 0,
                                        _ => u32::try_from(round(
                                            decimal(value)? / integer(100)
                                                * integer(i64::from(u32::MAX)),
                                        ))
                                        .map_err(|_| err("pedal percentage out of range"))?,
                                    };
                                    for &target in &part_routes {
                                        measure.events.push(Event {
                                            order: 0,
                                            time: stime,
                                            words: wire::cc(target, cc, amount),
                                        });
                                    }
                                }
                            }
                        }
                        if let Some(dyns) = element_node
                            .descendants()
                            .find(|n| n.has_tag_name("dynamics"))
                            .filter(|_| {
                                !element_node.descendants().any(|n| {
                                    n.has_tag_name("sound") && n.attribute("dynamics").is_some()
                                })
                            })
                        {
                            if let Some(mark) = dyns.children().find(Node::is_element) {
                                if let Some(level) = dynamic_mark(mark.tag_name().name()) {
                                    dynamic = integer(level);
                                }
                            }
                        }
                        if transient_kind(element_node) == Some("fp") {
                            dynamic = integer(45);
                        }
                        if element_node.has_tag_name("direction")
                            && !crate::remarks::raw_direction(element_node)
                        {
                            for &target in &part_routes {
                                measure.settings.push((
                                    time,
                                    target,
                                    scope_voice.clone(),
                                    scope_staff.clone(),
                                    element_node,
                                ));
                            }
                        }
                        if let Some(metro) = element_node
                            .descendants()
                            .find(|n| n.has_tag_name("metronome"))
                            .filter(|_| {
                                !element_node.descendants().any(|n| {
                                    n.has_tag_name("sound") && n.attribute("tempo").is_some()
                                })
                            })
                        {
                            if let (Some(unit), Some(rate)) =
                                (text(metro, "beat-unit"), text(metro, "per-minute"))
                            {
                                if let Ok(rate) = decimal(rate) {
                                    let dot_count = metro
                                        .children()
                                        .filter(|n| n.has_tag_name("beat-unit-dot"))
                                        .count();
                                    let beat = note_type(unit)?
                                        * (integer(2) - Rational::new(1, 1 << dot_count));
                                    let bpm = rate * beat;
                                    measure.events.push(Event {
                                        order: 0,
                                        time,
                                        words: tempo_words(bpm, &mut out.report, &source_path)?,
                                    });
                                }
                            }
                        }
                        if let Some(filter) = element_node
                            .descendants()
                            .filter(|n| n.has_tag_name("sound"))
                            .find_map(|n| n.attribute("time-only"))
                        {
                            let passes = filter
                                .split(',')
                                .map(|s| {
                                    s.trim()
                                        .parse::<u32>()
                                        .map_err(|_| err("invalid sound time-only"))
                                })
                                .collect::<Result<Vec<_>>>()?;
                            for index in event_start..measure.events.len() {
                                measure.event_passes.insert(index, passes.clone());
                            }
                        }
                        out.report.record(
                            &source_path,
                            "direction",
                            "playback fields normalized; original direction retained",
                        );
                    }
                    MeasureElement::Barline(_) => {
                        for rep in element_node.children().filter(|n| n.has_tag_name("repeat")) {
                            if rep.attribute("direction") == Some("forward") {
                                measure.repeat_start = true;
                            } else {
                                measure.repeat_times = Some(
                                    rep.attribute("times")
                                        .unwrap_or("2")
                                        .parse()
                                        .map_err(|_| err("invalid repeat count"))?,
                                );
                            }
                        }
                        if let Some(ending) = child(element_node, "ending")
                            .filter(|e| e.attribute("type") == Some("start"))
                        {
                            for pass in ending.attribute("number").unwrap_or("").split(',') {
                                if let Ok(p) = pass.trim().parse() {
                                    measure.ending.insert(p);
                                }
                            }
                        }
                        measure.ending_stop |= element_node.children().any(|n| {
                            n.has_tag_name("ending")
                                && matches!(n.attribute("type"), Some("stop" | "discontinue"))
                        });
                    }
                    MeasureElement::Note(typed_note) => {
                        if typed_note.content.instrument.len() > 1 {
                            return Err(Diagnostic::new(
                                "E_INSTRUMENT_BINDING",
                                "multiple instruments on one note require a split binding",
                            ));
                        }
                        let route = typed_note
                            .content
                            .instrument
                            .first()
                            .map(|i| {
                                instrument_routes
                                    .get(&i.attributes.id.0)
                                    .copied()
                                    .ok_or_else(|| {
                                        Diagnostic::new(
                                            "E_INSTRUMENT_BINDING",
                                            format!("unknown instrument {}", i.attributes.id.0),
                                        )
                                    })
                            })
                            .transpose()?
                            .unwrap_or(route);
                        let voice = typed_note
                            .content
                            .voice
                            .as_ref()
                            .map_or("1", |v| v.content.as_str());
                        let grace = grace_by_voice.entry(voice.into()).or_default();
                        note_nodes.insert(source_path.clone(), element_node);
                        note_scopes.insert(
                            source_path.clone(),
                            (
                                voice.into(),
                                typed_note
                                    .content
                                    .staff
                                    .as_ref()
                                    .map_or_else(|| "1".into(), |s| s.content.0.to_string()),
                                element_node.attribute("dynamics").is_some()
                                    || element_node
                                        .descendants()
                                        .any(|n| n.has_tag_name("other-play")),
                                element_node.descendants().any(|n| n.has_tag_name("accent")),
                            ),
                        );
                        let (audible, chord, duration, cue, is_grace) =
                            match &typed_note.content.info {
                                NoteType::Normal(info) => (
                                    &info.audible,
                                    info.chord.is_some(),
                                    number(element_node, "duration", zero())? / divisions,
                                    false,
                                    false,
                                ),
                                NoteType::Cue(info) => (
                                    &info.audible,
                                    info.chord.is_some(),
                                    number(element_node, "duration", zero())? / divisions,
                                    true,
                                    false,
                                ),
                                NoteType::Grace(info) => match &info.info {
                                    GraceType::Normal(n) => {
                                        (&n.audible, n.chord.is_some(), zero(), false, true)
                                    }
                                    GraceType::Cue(n) => {
                                        (&n.audible, n.chord.is_some(), zero(), true, true)
                                    }
                                },
                            };
                        if !is_grace && duration <= zero() {
                            return Err(err("note duration must be positive"));
                        }
                        let onset = if chord { previous } else { cursor };
                        if !chord && !is_grace {
                            previous = cursor;
                            cursor += duration;
                        }
                        measure.length = measure.length.max(onset + duration);
                        note_positions.insert(source_path.clone(), (onset, divisions));
                        if cue || matches!(audible, AudibleType::Rest(_)) {
                            continue;
                        }
                        let mut p = match audible {
                            AudibleType::Pitch(_) => {
                                read_pitch(
                                    child(element_node, "pitch")
                                        .ok_or_else(|| err("missing lexical pitch"))?,
                                )? + transpose
                            }
                            AudibleType::Unpitched(_) => {
                                let instrument = typed_note
                                    .content
                                    .instrument
                                    .first()
                                    .map(|i| i.attributes.id.0.as_str());
                                let midi = score_part
                                    .and_then(|s| {
                                        s.descendants().find(|n| {
                                            n.has_tag_name("midi-instrument")
                                                && instrument
                                                    .is_none_or(|id| n.attribute("id") == Some(id))
                                        })
                                    })
                                    .and_then(|m| text(m, "midi-unpitched"))
                                    .ok_or_else(|| {
                                        Diagnostic::new("E_PERCUSSION_MAP", &source_path)
                                    })?;
                                let key = midi
                                    .parse::<u8>()
                                    .map_err(|_| err("invalid midi-unpitched"))?;
                                if !(1..=128).contains(&key) {
                                    return Err(err("midi-unpitched outside 1..128"));
                                }
                                integer(i64::from(key - 1))
                            }
                            AudibleType::Rest(_) => unreachable!(),
                        };
                        // MusicXML harmonic pitch-role flags describe this written
                        // pitch, not a universally determined sounding partial.
                        // N17/N18: do not sound a base/touching pitch as the harmonic.
                        if element_node.descendants().any(|n| {
                            n.has_tag_name("base-pitch") || n.has_tag_name("touching-pitch")
                        }) {
                            return Err(Diagnostic::new(
                                "E_HARMONIC_PITCH",
                                format!(
                                    "{source_path}: base/touching harmonic pitch needs an explicit sounding-pitch realization; no universal partial inferred"
                                ),
                            ));
                        }
                        let mut tags = element_node
                            .descendants()
                            .filter(Node::is_element)
                            .filter(|n| n.ancestors().any(|a| a.has_tag_name("notations")))
                            .map(|n| n.tag_name().name().to_string())
                            .collect::<Vec<_>>();
                        for slur in element_node
                            .descendants()
                            .filter(|n| n.has_tag_name("slur"))
                        {
                            if slur.attribute("type") == Some("start") {
                                slurs.insert((
                                    voice.to_string(),
                                    slur.attribute("number").unwrap_or("1").to_string(),
                                ));
                            }
                        }
                        let slurred = slurs.iter().any(|(v, _)| v == voice);
                        let (mut articulation, mut descriptor) = rules::native(
                            &tags,
                            slurred,
                            element_node.attribute("pizzicato") == Some("yes"),
                        )?;
                        for term in notation_terms
                            .get(&element_node.range().start)
                            .into_iter()
                            .flatten()
                        {
                            let (candidate, refinement) = rules::term(term)
                                .ok_or_else(|| Diagnostic::new("E_TECHNIQUE", *term))?;
                            (articulation, descriptor) =
                                rules::combine(articulation, descriptor, candidate, refinement)?;
                        }
                        for symbol in element_node
                            .descendants()
                            .filter(|n| n.attribute("smufl").is_some())
                        {
                            let glyph = symbol.attribute("smufl").expect("filtered attribute");
                            if let Some((a, d)) = rules::glyph(glyph) {
                                (articulation, descriptor) =
                                    rules::combine(articulation, descriptor, a, d)?;
                            } else if symbol.has_tag_name("other-technical")
                                || symbol.has_tag_name("other-articulation")
                            {
                                out.report.record(&source_path,"unbound SMuFL remark",format!("{glyph}: retain glyph identity for explicit receiver binding"));
                            }
                        }
                        for other in element_node.descendants().filter(|n| {
                            n.has_tag_name("other-technical")
                                || n.has_tag_name("other-articulation")
                                || n.has_tag_name("other-ornament")
                        }) {
                            let term = other.text().unwrap_or("");
                            if term.trim().is_empty()
                                && other.attribute("smufl").and_then(rules::glyph).is_some()
                            {
                                continue;
                            }
                            if let Some((candidate, refinement)) = rules::term(term) {
                                (articulation, descriptor) = rules::combine(
                                    articulation,
                                    descriptor,
                                    candidate,
                                    refinement,
                                )?;
                            } else {
                                out.report.record(&source_path,"unbound performer remark",format!("{term}: literal text retained for an explicit receiver/library binding; no fabricated articulation"));
                            }
                        }
                        let mut on_order = 0;
                        let mut off_order = 0;
                        let mut literal_attack = false;
                        let mut explicit_attack = element_node.attribute("dynamics").is_some();
                        let mut attack = velocity(
                            element_node
                                .attribute("dynamics")
                                .map_or(Ok(dynamic), |v| Ok(decimal(v)? * Rational::new(9, 10)))?,
                        )?;
                        let mut release = 0x8000;
                        for other in element_node
                            .descendants()
                            .filter(|n| n.has_tag_name("other-play"))
                        {
                            if other.attribute("type") != Some(crate::extension::XML_TYPE) {
                                return Err(Diagnostic::new("E_PLAYBACK_EXTENSION", &source_path));
                            }
                            let value: serde_json::Value =
                                serde_json::from_str(other.text().unwrap_or(""))
                                    .map_err(|e| err(e.to_string()))?;
                            crate::extension::validate_annotation(&value)?;
                            on_order = value
                                .get("onOrder")
                                .and_then(serde_json::Value::as_u64)
                                .map_or(Ok(0), u32::try_from)
                                .map_err(|_| err("onOrder out of range"))?;
                            off_order = value
                                .get("offOrder")
                                .and_then(serde_json::Value::as_u64)
                                .map_or(Ok(0), u32::try_from)
                                .map_err(|_| err("offOrder out of range"))?;
                            let replacements =
                                value.get("replace").and_then(serde_json::Value::as_array);
                            if let Some(paths) = replacements {
                                for replaced in paths {
                                    let target = replaced.as_str().expect("validated source path");
                                    if !element_node
                                        .descendants()
                                        .filter(Node::is_element)
                                        .any(|n| xml_path(n) == target)
                                    {
                                        return Err(Diagnostic::new(
                                            "E_REPLACEMENT",
                                            format!(
                                                "{target}: replacement must identify an existing node on this note"
                                            ),
                                        ));
                                    }
                                    out.report.record(
                                        target,
                                        "explicit replacement",
                                        "superseded by the note's structured playback annotation",
                                    );
                                }
                            }
                            let replaces = |target: Node<'_, '_>| {
                                replacements.is_some_and(|paths| {
                                    paths
                                        .iter()
                                        .any(|v| v.as_str() == Some(xml_path(target).as_str()))
                                })
                            };
                            let mut d = crate::extension::annotation_descriptor(&value)?;
                            let candidate = Articulation::named(&d.technique)?;
                            if (articulation.at, articulation.subclass) != (0x10, 0)
                                && (articulation.at, articulation.subclass)
                                    != (candidate.at, candidate.subclass)
                            {
                                if replacements.is_none_or(Vec::is_empty) {
                                    return Err(Diagnostic::new("E_CONFLICT", &source_path));
                                }
                                for native in element_node.descendants().filter(|n| {
                                    n.is_element()
                                        && rules::native(
                                            &[n.tag_name().name().into()],
                                            false,
                                            false,
                                        )
                                        .is_ok_and(|(a, _)| a != Articulation::default())
                                }) {
                                    if !replaces(native) {
                                        return Err(Diagnostic::new(
                                            "E_REPLACEMENT",
                                            "conflicting native articulation must be explicitly replaced",
                                        ));
                                    }
                                }
                                articulation = Articulation::default();
                            }
                            if let Some(raw) = d
                                .qualifiers
                                .get("urn:midi2workbench:sounding-pitch")
                                .and_then(serde_json::Value::as_str)
                            {
                                let written = child(element_node, "pitch").ok_or_else(|| {
                                    Diagnostic::new(
                                        "E_REPLACEMENT",
                                        "sounding-pitch requires a written pitch",
                                    )
                                })?;
                                if !replaces(written) {
                                    return Err(Diagnostic::new(
                                        "E_REPLACEMENT",
                                        "sounding-pitch requires explicit replacement of the written pitch node",
                                    ));
                                }
                                p = decimal(raw)?;
                                if p < zero() || p >= integer(128) {
                                    return Err(err("sounding pitch outside [0,128)"));
                                }
                            }
                            articulation.at = candidate.at;
                            articulation.subclass = candidate.subclass;
                            for (field, target) in [
                                ("variation", &mut articulation.variation),
                                ("direction", &mut articulation.direction),
                                ("string", &mut articulation.string),
                            ] {
                                if let Some(v) = value.get(field) {
                                    *target = u8::try_from(
                                        v.as_u64().ok_or_else(|| err("integer field required"))?,
                                    )
                                    .map_err(|_| err("field out of range"))?;
                                }
                            }
                            if let Some(v) = value.get("resetRoundRobin") {
                                articulation.reset_round_robin =
                                    v.as_bool().ok_or_else(|| err("boolean reset required"))?;
                            }
                            if let Some(v) =
                                value.get("release").and_then(serde_json::Value::as_str)
                            {
                                articulation.release = match v {
                                    "none" => 0,
                                    "soft" => 1,
                                    "hard" => 2,
                                    "rise" => 3,
                                    "fall" => 4,
                                    _ => return Err(err("unknown release")),
                                };
                            }
                            if let Some(v) = d
                                .qualifiers
                                .get("urn:midi2workbench:attack-velocity")
                                .and_then(serde_json::Value::as_u64)
                            {
                                explicit_attack = true;
                                literal_attack = true;
                                attack =
                                    u16::try_from(v).map_err(|_| err("velocity out of range"))?;
                            }
                            if let Some(v) =
                                value.get("releaseTime").and_then(serde_json::Value::as_u64)
                            {
                                release =
                                    u16::try_from(v).map_err(|_| err("release out of range"))?;
                            }
                            d.qualifiers.remove("urn:midi2workbench:attack-velocity");
                            d.qualifiers.remove("urn:midi2workbench:sounding-pitch");
                            descriptor = (!d.qualifiers.is_empty()).then_some(d);
                        }
                        let mut extended_string = None;
                        for string in element_node
                            .descendants()
                            .filter(|n| n.has_tag_name("string"))
                        {
                            let value = string
                                .text()
                                .unwrap_or("")
                                .parse::<u32>()
                                .map_err(|_| err("invalid string number"))?;
                            if value == 0 {
                                return Err(err("string number must be positive"));
                            }
                            if value <= 7 {
                                articulation.string = u8::try_from(value).expect("3-bit string");
                            } else {
                                articulation.string = 0;
                                extended_string = Some(value);
                            }
                        }
                        articulation.validate()?;
                        let mut standard_facets = descriptor.take().unwrap_or_else(|| Descriptor {
                            technique: rules::by_coordinate(articulation.at, articulation.subclass)
                                .expect("validated articulation")
                                .id
                                .into(),
                            ..Descriptor::default()
                        });
                        if has_scoped_remarks {
                            standard_facets.qualifiers.entry(crate::remarks::CONTEXT.into()).or_insert_with(||serde_json::json!({"voice":voice,"staff":text(element_node,"staff").unwrap_or("1")}));
                        }
                        if let Some(value) = extended_string {
                            standard_facets
                                .qualifiers
                                .insert("urn:midi2workbench:string-number".into(), value.into());
                        }
                        for value in element_node.descendants().filter(|n| {
                            n.has_tag_name("mute") && n.ancestors().any(|a| a.has_tag_name("play"))
                        }) {
                            let slot = mute_slot(
                                value.text().unwrap_or(""),
                                out.piano_routes.contains(&route),
                            )?;
                            standard_facets.qualifiers.insert(
                                "urn:midi2workbench:mute".into(),
                                serde_json::json!({"slot":slot,"amount":u32::MAX}),
                            );
                        }
                        if let Some(value) = element_node.attribute("end-dynamics") {
                            release = velocity(decimal(value)? * Rational::new(9, 10))?;
                            standard_facets.qualifiers.insert(
                                "urn:midi2workbench:musicxml-end-dynamics".into(),
                                value.into(),
                            );
                            out.report.record(&source_path,"end-dynamics","MusicXML Note Off velocity percentage scaled into the MIDI 2.0 field; M2-123 interprets it as relative release time, M2-126 as damping response; no absolute decay duration or equal audio asserted");
                        }
                        descriptor =
                            (!standard_facets.qualifiers.is_empty()).then_some(standard_facets);
                        crate::remarks::attach(
                            &mut descriptor,
                            rules::by_coordinate(articulation.at, articulation.subclass)
                                .expect("validated coordinate")
                                .id,
                            crate::remarks::note_values(element_node),
                        );
                        let mut note = Note {
                            on_order,
                            off_order,
                            onset: onset + attr_number(element_node, "attack", zero())? / divisions,
                            end: onset
                                + duration
                                + attr_number(element_node, "release", zero())? / divisions,
                            pitch: p,
                            velocity: attack,
                            release_velocity: release,
                            route,
                            key: u8::try_from(p.floor().to_integer().clamp(0, 127))
                                .expect("clamped key"),
                            articulation,
                            descriptor,
                            source: source_path.clone(),
                        };
                        if tags.iter().any(|t| t == "accent") && !explicit_attack {
                            note.velocity = velocity((dynamic + integer(12)).min(integer(127)))?;
                            out.report.record(
                                &source_path,
                                "accent policy",
                                "+12 nominal velocity",
                            );
                        }
                        if let Some(scope) = note_scopes.get_mut(&source_path) {
                            scope.2 = explicit_attack;
                        }
                        if !literal_attack {
                            if let Some(raw) = element_node.attribute("dynamics") {
                                let _ = quantized_level(
                                    decimal(raw)? * Rational::new(9, 10),
                                    16,
                                    &mut out.report,
                                    &source_path,
                                )?;
                            }
                        }
                        if is_grace {
                            grace.push((note, element_node));
                            continue;
                        }
                        if !grace.is_empty() {
                            if grace.iter().any(|(_, n)| {
                                child(*n, "grace")
                                    .is_some_and(|g| g.attribute("make-time").is_some())
                            }) {
                                let mut allocated = Vec::new();
                                let mut offset = zero();
                                let mut chord_offset = zero();
                                for (mut g, gn) in grace.drain(..) {
                                    let attrs =
                                        child(gn, "grace").ok_or_else(|| err("missing grace"))?;
                                    if attrs.attribute("steal-time-following").is_some()
                                        || attrs.attribute("steal-time-previous").is_some()
                                    {
                                        return Err(Diagnostic::new(
                                            "E_GRACE",
                                            "make-time and steal-time contradict each other",
                                        ));
                                    }
                                    let span = attrs
                                        .attribute("make-time")
                                        .map(decimal)
                                        .transpose()?
                                        .ok_or_else(|| {
                                            Diagnostic::new(
                                                "E_GRACE",
                                                "mixed make-time and unspecified grace timing",
                                            )
                                        })?
                                        / divisions;
                                    if span <= zero() {
                                        return Err(Diagnostic::new(
                                            "E_GRACE",
                                            "make-time must be positive",
                                        ));
                                    }
                                    let start = if child(gn, "chord").is_some() {
                                        chord_offset
                                    } else {
                                        chord_offset = offset;
                                        offset += span;
                                        chord_offset
                                    };
                                    g.onset = onset;
                                    g.end = onset + span;
                                    allocated.push((g, gn, start, span));
                                }
                                measure.holds.push((onset, offset));
                                for (g, gn, start, span) in allocated {
                                    made_time.insert(g.source.clone(), (start, span, offset));
                                    out.report.record(
                                        &g.source,
                                        "grace timing",
                                        "explicit make-time inserted into the shared clock",
                                    );
                                    let _ = gn;
                                    measure
                                        .notes
                                        .push((g, voice.into(), false, false, Vec::new()));
                                }
                            } else {
                                let principal_duration = note.end - note.onset;
                                let groups = grace
                                    .iter()
                                    .filter(|(_, g)| child(*g, "chord").is_none())
                                    .count()
                                    .max(1);
                                let mut next = onset;
                                let mut previous_grace = onset;
                                let mut following_stolen = zero();
                                for (mut g, gn) in grace.drain(..) {
                                    let attrs =
                                        child(gn, "grace").ok_or_else(|| err("missing grace"))?;
                                    let preceding = attrs
                                        .attribute("steal-time-previous")
                                        .map(decimal)
                                        .transpose()?;
                                    let following = attrs
                                        .attribute("steal-time-following")
                                        .map(decimal)
                                        .transpose()?;
                                    if attrs.attribute("make-time").is_some() {
                                        return Err(Diagnostic::new(
                                            "E_GRACE",
                                            "make-time needs a score-wide clock insertion",
                                        ));
                                    }
                                    let chord = child(gn, "chord").is_some();
                                    let share = if let Some(percent) = preceding {
                                        let prev = measure
                                            .notes
                                            .iter_mut()
                                            .rev()
                                            .find(|(_, v, ..)| v == voice)
                                            .or_else(|| {
                                                measures.last_mut().and_then(
                                                    |m: &mut Measure<'_>| {
                                                        m.notes
                                                            .iter_mut()
                                                            .rev()
                                                            .find(|(_, v, ..)| v == voice)
                                                    },
                                                )
                                            })
                                            .ok_or_else(|| {
                                                Diagnostic::new(
                                                    "E_GRACE",
                                                    "steal-time-previous without preceding note",
                                                )
                                            })?;
                                        let span =
                                            (prev.0.end - prev.0.onset) * percent / integer(100);
                                        if !chord {
                                            prev.0.end -= span;
                                            next = onset - span;
                                        }
                                        span
                                    } else {
                                        principal_duration
                                            * following.map_or(
                                                policy.grace_share
                                                    / integer(
                                                        i64::try_from(groups)
                                                            .expect("bounded XML nodes"),
                                                    ),
                                                |p| p / integer(100),
                                            )
                                    };
                                    if share <= zero() || share >= principal_duration {
                                        return Err(Diagnostic::new(
                                            "E_GRACE",
                                            "grace time must be positive and leave a principal duration",
                                        ));
                                    }
                                    if chord {
                                        g.onset = previous_grace;
                                    } else {
                                        g.onset = next;
                                        previous_grace = next;
                                        next += share;
                                        if preceding.is_none() {
                                            following_stolen += share;
                                        }
                                    }
                                    g.end = g.onset + share;
                                    out.report.record(
                                        &g.source,
                                        "grace timing",
                                        if preceding.is_some() {
                                            "explicit steal-time-previous"
                                        } else if following.is_some() {
                                            "explicit steal-time-following"
                                        } else {
                                            "corpus-v1 default following share"
                                        },
                                    );
                                    let _ = gn;
                                    measure
                                        .notes
                                        .push((g, voice.into(), false, false, Vec::new()));
                                }
                                note.onset += following_stolen;
                            }
                        }
                        if tags.iter().any(|t| t == "fermata") {
                            measure.holds.push((
                                onset + duration,
                                duration * (policy.fermata_factor - integer(1)),
                            ));
                            out.report.record(
                                &source_path,
                                "fermata policy",
                                policy.fermata_factor.to_string(),
                            );
                        }
                        if tags.iter().any(|t| t == "caesura") {
                            measure.holds.push((onset + duration, policy.caesura_gap));
                            out.report.record(
                                &source_path,
                                "caesura policy",
                                policy.caesura_gap.to_string(),
                            );
                        }
                        if tags.iter().any(|t| t == "breath-mark") {
                            note.end -= policy.breath_gap.min(duration / 2);
                            out.report.record(
                                &source_path,
                                "breath policy",
                                policy.breath_gap.to_string(),
                            );
                        }
                        let tie_start = element_node
                            .children()
                            .any(|n| n.has_tag_name("tie") && n.attribute("type") == Some("start"));
                        let tie_stop = element_node
                            .children()
                            .any(|n| n.has_tag_name("tie") && n.attribute("type") == Some("stop"));
                        if attack > 0
                            || literal_attack
                            || element_node.attribute("dynamics").is_none()
                        {
                            measure.notes.push((
                                note,
                                voice.into(),
                                tie_start,
                                tie_stop,
                                typed_note
                                    .attributes
                                    .time_only
                                    .as_ref()
                                    .map_or_else(Vec::new, |t| t.0.clone()),
                            ));
                        }
                        for slur in element_node.descendants().filter(|n| {
                            n.has_tag_name("slur") && n.attribute("type") == Some("stop")
                        }) {
                            slurs.remove(&(
                                voice.into(),
                                slur.attribute("number").unwrap_or("1").into(),
                            ));
                        }
                        tags.clear();
                        out.report.record(
                            &source_path,
                            "note",
                            "pitch, lifecycle and articulation normalized; notation archived",
                        );
                    }
                    _ => {}
                }
                for e in &mut measure.events[phase_start..] {
                    // Native directions after an attack occupy a later authored slot.
                    // Raw events carry their explicit slot and must retain it.
                    if e.order == 0
                        && !element_node.descendants().any(|n| {
                            n.has_tag_name("other-play")
                                && n.attribute("type") == Some("urn:midi2workbench:ump:v1")
                        })
                        && measure
                            .notes
                            .iter()
                            .any(|(n, ..)| n.onset == e.time && n.route == route)
                    {
                        e.order = 1;
                    }
                }
            }
            measures.push(measure);
        }
        for (voice, grace) in grace_by_voice {
            if !grace.is_empty() {
                let previous = measures
                    .iter_mut()
                    .rev()
                    .find_map(|m| m.notes.iter_mut().rev().find(|(_, v, _, _, _)| *v == voice))
                    .ok_or_else(|| {
                        Diagnostic::new("E_GRACE", "grace group without a principal note")
                    })?;
                let mut shares = Vec::new();
                for (_, gn) in &grace {
                    let attrs = child(*gn, "grace").ok_or_else(|| err("missing grace"))?;
                    if attrs.attribute("make-time").is_some()
                        || attrs.attribute("steal-time-following").is_some()
                    {
                        return Err(Diagnostic::new(
                            "E_GRACE",
                            "terminal grace allocation requires a preceding principal; make-time/following allocation has no following boundary",
                        ));
                    }
                    shares.push(
                        attrs
                            .attribute("steal-time-previous")
                            .map(decimal)
                            .transpose()?
                            .map_or(
                                policy.grace_share
                                    / integer(
                                        i64::try_from(grace.len()).expect("bounded XML nodes"),
                                    ),
                                |v| v / integer(100),
                            ),
                    );
                }
                let total = shares.iter().copied().sum::<Rational>();
                if total <= zero() || total >= integer(1) || shares.iter().any(|s| *s <= zero()) {
                    return Err(Diagnostic::new(
                        "E_GRACE",
                        "terminal grace shares must be positive and leave preceding duration",
                    ));
                }
                let principal_duration = previous.0.end - previous.0.onset;
                let span = principal_duration * total;
                let onset = previous.0.end - span;
                previous.0.end = onset;
                let last = measures
                    .last_mut()
                    .ok_or_else(|| err("missing final measure"))?;
                let mut next = onset;
                for ((mut n, gn), share) in grace.into_iter().zip(shares) {
                    n.onset = next;
                    n.end = n.onset + principal_duration * share;
                    next = n.end;
                    out.report.record(
                        &n.source,
                        "terminal grace policy",
                        "steal declared share from preceding principal",
                    );
                    let _ = gn;
                    last.notes
                        .push((n, voice.clone(), false, false, Vec::new()));
                }
            }
        }
        parts.push(measures);
    }
    let count = parts.iter().map(Vec::len).max().unwrap_or(0);
    if count == 0 {
        return Err(err("score has no measures"));
    }
    let mut global = vec![Measure::default(); count];
    for part in parts {
        for (i, m) in part.into_iter().enumerate() {
            let g = &mut global[i];
            g.length = g.length.max(m.length);
            g.notes.extend(m.notes);
            let event_offset = g.events.len();
            g.event_passes.extend(
                m.event_passes
                    .into_iter()
                    .map(|(index, passes)| (index + event_offset, passes)),
            );
            g.events.extend(m.events);
            g.repeat_start |= m.repeat_start;
            g.repeat_times = g.repeat_times.or(m.repeat_times);
            g.ending.extend(m.ending);
            g.ending_stop |= m.ending_stop;
            g.holds.extend(m.holds);
            g.navigation.extend(m.navigation);
            g.settings.extend(m.settings);
        }
    }
    let mut active_ending = BTreeSet::new();
    for m in &mut global {
        if m.ending.is_empty() {
            m.ending.clone_from(&active_ending);
        } else {
            active_ending.clone_from(&m.ending);
        }
        if m.ending_stop {
            active_ending.clear();
        }
    }
    let swing_grid = swing_grid(&note_nodes, &note_positions)?;
    let order = occurrences(&global)?;
    let mut clock = zero();
    let mut ties = BTreeMap::<(Route, String, Rational), Vec<(usize, Rational)>>::new();
    let mut holds = BTreeMap::<Rational, Rational>::new();
    let mut settings = Vec::new();
    for (occurrence, &(mi, pass, lo, hi)) in order.iter().enumerate() {
        let contiguous = occurrence > 0 && {
            let (prev, _, _, end) = order[occurrence - 1];
            (prev == mi && end == lo)
                || (prev + 1 == mi && end == global[prev].length && lo == zero())
        };
        let continues = order
            .get(occurrence + 1)
            .is_some_and(|&(next, _, start, _)| {
                (next == mi && start == hi)
                    || (next == mi + 1 && hi == global[mi].length && start == zero())
            });
        let m = &global[mi];
        for &(t, r, ref v, ref staff, node) in &m.settings {
            if t < lo || t > hi || t == hi && hi != m.length || !setting_applies(node, pass)? {
                continue;
            }
            settings.push((t + clock - lo, r, v.clone(), staff.clone(), node));
        }
        for (n, voice, start, stop, passes) in &m.notes {
            if !passes.is_empty() && !passes.iter().any(|p| u32::from(*p) == pass) {
                continue;
            }
            if !(n.onset >= lo && n.onset < hi || !contiguous && n.onset < lo && n.end > lo) {
                continue;
            }
            let mut n = n.clone();
            n.onset = n.onset.max(lo) + clock - lo;
            n.end += clock - lo;
            let _ = write!(&mut n.source, "#occurrence={occurrence}");
            let source_path = n.source.split('#').next().unwrap_or(&n.source);
            let source_note = *note_nodes
                .get(source_path)
                .ok_or_else(|| err("missing source note"))?;
            let &(local, divisions) = note_positions
                .get(source_path)
                .ok_or_else(|| err("missing note position"))?;
            let nominal_start = clock - lo + local;
            let nominal_end = nominal_start + number(source_note, "duration", zero())? / divisions;
            apply_swing(
                &mut n,
                clock - lo,
                &settings,
                &note_nodes,
                &note_positions,
                &swing_grid,
            )?;
            let part = source_note
                .ancestors()
                .find(|n| n.has_tag_name("part"))
                .and_then(|n| n.attribute("id"))
                .unwrap_or("");
            let key = (n.route, format!("{part}/{voice}"), n.pitch);
            if *stop {
                if let Some(active) = ties.get_mut(&key) {
                    if let Some(index) = active.iter().position(|(_, end)| *end == nominal_start) {
                        let (previous, _) = active.remove(index);
                        let prior: &mut Note = &mut out.notes[previous];
                        prior.end = prior.end.max(n.end);
                        prior.off_order = n.off_order;
                        prior.release_velocity = n.release_velocity;
                        prior.articulation.release = n.articulation.release;
                        if *start {
                            active.push((previous, nominal_end));
                        }
                        continue;
                    }
                }
            }
            if *start {
                ties.entry(key)
                    .or_default()
                    .push((out.notes.len(), nominal_end));
            }
            out.notes.push(n);
        }
        for (index, e) in m.events.iter().enumerate() {
            if e.time < lo || e.time > hi || e.time == hi && hi != m.length {
                continue;
            }
            if m.event_passes
                .get(&index)
                .is_some_and(|p| !p.contains(&pass))
            {
                continue;
            }
            out.events.push(Event {
                order: e.order,
                time: e.time + clock - lo,
                words: e.words.clone(),
            });
        }
        for (time, extra) in &m.holds {
            if *time < lo || *time > hi || *time == lo && lo != zero() {
                continue;
            }
            let target = holds.entry(*time + clock - lo).or_default();
            *target = (*target).max(*extra);
        }
        clock += hi - lo;
        if !continues
            && (order.get(occurrence + 1).is_some() || hi < m.length || mi + 1 < global.len())
        {
            for n in &mut out.notes {
                n.end = n.end.min(clock);
            }
            ties.clear();
        }
    }
    let (mut dynamics, mut wedges) = expression_points(&settings, &out.piano_routes)?;
    for n in &mut out.notes {
        let onset = n.onset;
        let end = n.end;
        n.onset += holds.range(..=onset).map(|(_, v)| *v).sum::<Rational>();
        n.end += holds.range(..=end).map(|(_, v)| *v).sum::<Rational>();
        if let Some((offset, span, total)) =
            made_time.get(n.source.split('#').next().unwrap_or(&n.source))
        {
            n.onset = n.onset - *total + *offset;
            n.end = n.onset + *span;
        }
    }
    for e in &mut out.events {
        let t = e.time;
        e.time += holds.range(..=t).map(|(_, v)| *v).sum::<Rational>();
    }
    for (time, ..) in &mut dynamics {
        *time += holds.range(..=*time).map(|(_, v)| *v).sum::<Rational>();
    }
    for (time, ..) in &mut wedges {
        *time += holds.range(..=*time).map(|(_, v)| *v).sum::<Rational>();
    }
    for (time, ..) in &mut settings {
        *time += holds.range(..=*time).map(|(_, v)| *v).sum::<Rational>();
    }
    let mut realized = Vec::new();
    for note in std::mem::take(&mut out.notes) {
        let path = note.source.split('#').next().unwrap_or(&note.source);
        let node = *note_nodes
            .get(path)
            .ok_or_else(|| err("missing source note for pattern realization"))?;
        let pattern_node = trill_sources
            .get(&node.range().start)
            .copied()
            .unwrap_or(node);
        if pattern_node != node {
            out.report.record(
                &note.source,
                "trill span",
                "inherited explicitly named trill; numbered source span",
            );
        }
        realized.extend(realize_pattern(note, pattern_node, node, &mut out.report)?);
    }
    out.notes = realized;
    realize_arpeggios(&mut out, &note_nodes, policy)?;
    apply_settings(
        &mut out,
        &settings,
        &note_scopes,
        &note_nodes,
        &instrument_routes,
    )?;
    // D1 review: an authored early release fixes the performed gate, but does
    // not prove a different sampled attack/production (ordinary phrasing can end
    // early too). Do not promote it to staccato/martellato from timing alone.
    realize_dynamics(&mut out, &dynamics, &wedges, &note_scopes, &note_nodes)?;
    realize_envelopes(&mut out, policy, &note_scopes, &dynamics, &note_nodes)?;
    out.notes.retain(|n| {
        n.velocity != 0
            || note_scopes
                .get(n.source.split('#').next().unwrap_or(&n.source))
                .is_some_and(|scope| scope.2)
    });
    if !out
        .events
        .iter()
        .any(|e| e.time == zero() && e.words.first() == Some(&0xd010_0000))
    {
        out.events.insert(
            0,
            Event {
                order: 0,
                time: zero(),
                words: tempo_words(
                    integer(i64::from(policy.default_tempo)),
                    &mut out.report,
                    "/",
                )?,
            },
        );
        out.report
            .record("/", "default tempo", policy.default_tempo.to_string());
    }
    out.duration = (clock + holds.range(..=clock).map(|(_, v)| *v).sum::<Rational>()).max(
        out.notes
            .iter()
            .map(|n| n.end)
            .chain(out.events.iter().map(|e| e.time))
            .max()
            .unwrap_or(zero()),
    );
    out.events.sort_by_key(|e| (e.time, e.order));
    Ok((source, out))
}

type SwingGrid<'a> = BTreeSet<(usize, &'a str, &'a str, Rational, Rational)>;
fn swing_grid<'a>(
    nodes: &BTreeMap<String, Node<'a, 'a>>,
    positions: &BTreeMap<String, (Rational, Rational)>,
) -> Result<SwingGrid<'a>> {
    let mut grid = BTreeSet::new();
    for (path, &node) in nodes {
        if child(node, "grace").is_some() {
            continue;
        }
        let Some(kind) = text(node, "type").filter(|k| matches!(*k, "eighth" | "16th")) else {
            continue;
        };
        let &(onset, divisions) = positions
            .get(path)
            .ok_or_else(|| err("missing note position"))?;
        let unit = note_type(kind)?;
        if number(node, "duration", zero())? / divisions == unit {
            grid.insert((
                node.parent().expect("score note parent").range().start,
                text(node, "voice").unwrap_or("1"),
                text(node, "staff").unwrap_or("1"),
                onset,
                unit,
            ));
        }
    }
    Ok(grid)
}
fn apply_swing(
    note: &mut Note,
    origin: Rational,
    settings: &[PlaybackSetting<'_>],
    nodes: &BTreeMap<String, Node<'_, '_>>,
    positions: &BTreeMap<String, (Rational, Rational)>,
    grid: &SwingGrid<'_>,
) -> Result<()> {
    let path = note.source.split('#').next().unwrap_or(&note.source);
    let source_note = *nodes.get(path).ok_or_else(|| err("missing source note"))?;
    if child(source_note, "grace").is_some() {
        return Ok(());
    }
    let &(onset, divisions) = positions
        .get(path)
        .ok_or_else(|| err("missing note position"))?;
    let voice = text(source_note, "voice").unwrap_or("1");
    let staff = text(source_note, "staff").unwrap_or("1");
    let Some((_, _, _, _, sound)) = settings
        .iter()
        .enumerate()
        .filter(|(_, s)| {
            s.0 <= origin + onset
                && s.1 == note.route
                && source_part(s.4) == source_part(source_note)
                && s.2.as_ref().is_none_or(|v| v == voice)
                && s.3.as_ref().is_none_or(|v| v == staff)
                && s.4.has_tag_name("sound")
                && child(s.4, "swing").is_some()
        })
        .max_by_key(|(i, s)| (s.0, *i))
        .map(|(_, s)| s)
    else {
        return Ok(());
    };
    let swing = child(*sound, "swing").expect("swing setting");
    if child(swing, "straight").is_some() {
        return Ok(());
    }
    let kind = text(swing, "swing-type").unwrap_or("eighth");
    if text(source_note, "type") != Some(kind) {
        return Ok(());
    }
    let unit = note_type(kind)?;
    if number(source_note, "duration", zero())? / divisions != unit {
        return Ok(());
    }
    let period = unit * integer(2);
    let beat = (onset / period).floor() * period;
    if onset != beat && onset != beat + unit {
        return Ok(());
    }
    let parent = source_note
        .parent()
        .expect("score note parent")
        .range()
        .start;
    if !grid.contains(&(parent, voice, staff, beat, unit))
        || !grid.contains(&(parent, voice, staff, beat + unit, unit))
    {
        return Ok(());
    }
    let first = number(swing, "first", zero())?;
    let second = number(swing, "second", zero())?;
    if first <= zero() || second <= zero() {
        return Err(Diagnostic::new("E_SWING", "positive swing ratio required"));
    }
    let split = period * first / (first + second);
    let (start, end) = if onset == beat {
        (beat, beat + split)
    } else {
        (beat + split, beat + period)
    };
    note.onset += start - onset;
    note.end += end - (onset + unit);
    Ok(())
}

// XML nodes remain the source model; these maps bind existing nodes, not a new AST.
fn trill_sources<'a>(root: Node<'a, 'a>) -> BTreeMap<usize, Node<'a, 'a>> {
    let notes = root
        .descendants()
        .filter(|n| n.has_tag_name("note"))
        .collect::<Vec<_>>();
    let mut active = BTreeMap::new();
    let mut named = BTreeMap::new();
    let mut sources = BTreeMap::new();
    for (index, &note) in notes.iter().enumerate() {
        let part = note
            .ancestors()
            .find(|n| n.has_tag_name("part"))
            .and_then(|n| n.attribute("id"))
            .unwrap_or("");
        let has_trill = note.descendants().any(|n| n.has_tag_name("trill-mark"));
        if has_trill {
            named.insert(part, note);
        }
        for line in note.descendants().filter(|n| n.has_tag_name("wavy-line")) {
            let key = (part, line.attribute("number").unwrap_or("1"));
            let kind = line.attribute("type").unwrap_or("");
            if kind == "start" {
                let origin = if has_trill {
                    Some(note)
                } else {
                    active
                        .get(&key)
                        .map(|(_, n)| *n)
                        .or_else(|| named.get(part).copied())
                };
                if let Some(origin) = origin {
                    active.insert(key, (index, origin));
                    sources.insert(note.range().start, origin);
                }
            } else if let Some(&(start, origin)) = active.get(&key) {
                if kind == "continue" {
                    sources.insert(note.range().start, origin);
                }
                if kind == "stop" {
                    let voice = text(origin, "voice").unwrap_or("1");
                    let staff = text(origin, "staff").unwrap_or("1");
                    for &between in &notes[start..index] {
                        if between
                            .ancestors()
                            .any(|n| n.has_tag_name("part") && n.attribute("id") == Some(part))
                            && text(between, "voice").unwrap_or("1") == voice
                            && text(between, "staff").unwrap_or("1") == staff
                        {
                            sources.insert(between.range().start, origin);
                        }
                    }
                    active.remove(&key);
                }
            }
        }
    }
    sources
}

fn realize_arpeggios(
    score: &mut Performance,
    nodes: &BTreeMap<String, Node<'_, '_>>,
    policy: &RenderingPolicy,
) -> Result<()> {
    if policy.arpeggio_span <= zero() {
        return Err(Diagnostic::new(
            "E_POLICY",
            "arpeggio span must be positive",
        ));
    }
    let mut groups = BTreeMap::new();
    for (index, note) in score.notes.iter().enumerate() {
        let Some(node) = nodes.get(note.source.split('#').next().unwrap_or(&note.source)) else {
            continue;
        };
        let Some(sign) = node.descendants().find(|n| n.has_tag_name("arpeggiate")) else {
            continue;
        };
        if note
            .descriptor
            .as_ref()
            .and_then(|d| d.qualifiers.get("urn:midi2workbench:expanded-pattern"))
            .and_then(|p| p["index"].as_u64())
            .is_some_and(|i| i != 0)
        {
            continue;
        }
        let part = node
            .ancestors()
            .find(|n| n.has_tag_name("part"))
            .and_then(|n| n.attribute("id"))
            .unwrap_or("");
        groups
            .entry((part, note.onset, sign.attribute("number").unwrap_or("1")))
            .or_insert_with(Vec::new)
            .push((index, sign.attribute("direction")));
    }
    for group in groups.values_mut() {
        if group.len() < 2 {
            continue;
        }
        let down = group.iter().any(|(_, d)| *d == Some("down"));
        if group.iter().any(|(_, d)| *d == Some("up")) && down {
            return Err(Diagnostic::new(
                "E_CONFLICT",
                "contradictory arpeggio direction",
            ));
        }
        group.sort_by_key(|(i, _)| score.notes[*i].pitch);
        if down {
            group.reverse();
        }
        let duration = group
            .iter()
            .map(|(i, _)| score.notes[*i].end - score.notes[*i].onset)
            .min()
            .expect("nonempty group");
        let span = policy.arpeggio_span.min(duration / 2);
        let count = i64::try_from(group.len() - 1).map_err(|_| err("arpeggio chord too large"))?;
        for (rank, (index, _)) in group.iter().enumerate() {
            let note = &mut score.notes[*index];
            note.onset += span
                * Rational::new(
                    i64::try_from(rank).map_err(|_| err("arpeggio chord too large"))?,
                    count,
                );
            score.report.record(
                &note.source,
                "arpeggio policy",
                format!(
                    "{} chord spread over {span} quarter",
                    if down { "descending" } else { "ascending" }
                ),
            );
        }
    }
    Ok(())
}

// Playback settings are evaluated in performed order, so a navigation jump
// keeps current state and then applies instructions at its destination.
#[allow(
    clippy::too_many_lines,
    reason = "Keep the stateful traversal together; semantic rules are delegated without duplicating transient models."
)]
fn apply_settings(
    score: &mut Performance,
    settings: &[PlaybackSetting<'_>],
    scopes: &BTreeMap<String, (String, String, bool, bool)>,
    note_nodes: &BTreeMap<String, Node<'_, '_>>,
    instrument_routes: &BTreeMap<String, Route>,
) -> Result<()> {
    let mut order = (0..score.notes.len()).collect::<Vec<_>>();
    order.sort_by_key(|i| score.notes[*i].onset);
    let mut cache = BTreeMap::new();
    let mut envelope_targets = BTreeMap::new();
    for index in order {
        let note = &mut score.notes[index];
        let path = note.source.split('#').next().unwrap_or(&note.source);
        let (voice, staff, ..) = scopes
            .get(path)
            .cloned()
            .unwrap_or_else(|| ("1".into(), "1".into(), false, false));
        let (changes, next, state, pizzicato, envelope) = cache
            .entry((
                note.route,
                source_part(
                    *note_nodes
                        .get(path)
                        .ok_or_else(|| err("missing source note"))?,
                )
                .to_string(),
                voice.clone(),
                staff.clone(),
            ))
            .or_insert_with(|| {
                let mut changes = settings
                    .iter()
                    .filter(|(_, r, v, s, node)| {
                        *r == note.route
                            && source_part(*node)
                                == source_part(*note_nodes.get(path).expect("source note"))
                            && v.as_ref().is_none_or(|v| *v == voice)
                            && s.as_ref().is_none_or(|s| *s == staff)
                    })
                    .collect::<Vec<_>>();
                changes.sort_by_key(|s| s.0);
                (
                    changes,
                    0usize,
                    Descriptor {
                        technique: "core.normal".into(),
                        ..Default::default()
                    },
                    false,
                    None,
                )
            });
        while let Some(change) = changes.get(*next).filter(|change| change.0 <= note.onset) {
            let element_node = change.4;
            if setting_transient(element_node).is_some() {
                *envelope = Some((change.0, element_node));
            } else if steady_level(element_node, score.piano_routes.contains(&note.route))?
                .is_some()
            {
                *envelope = None;
            }
            for sound in std::iter::once(element_node).filter(|n| n.has_tag_name("sound")) {
                if let Some(value) = sound.attribute("pizzicato") {
                    *pizzicato = value == "yes";
                }
                for play in sound.children().filter(|n| n.has_tag_name("play")) {
                    if play
                        .attribute("id")
                        .is_some_and(|id| instrument_routes.get(id) != Some(&note.route))
                    {
                        continue;
                    }
                    if let Some(mute) = child(play, "mute") {
                        state.qualifiers.insert("urn:midi2workbench:mute".into(),serde_json::json!({"slot":mute_slot(mute.text().unwrap_or(""),score.piano_routes.contains(&note.route))?,"amount":u32::MAX}));
                    }
                }
                for other in sound.descendants().filter(|n| {
                    n.has_tag_name("other-play")
                        && n.attribute("type") == Some(crate::extension::XML_TYPE)
                }) {
                    if let Some(id) = other
                        .ancestors()
                        .find(|n| n.has_tag_name("play"))
                        .and_then(|n| n.attribute("id"))
                    {
                        if instrument_routes.get(id) != Some(&note.route) {
                            continue;
                        }
                    }
                    let value: serde_json::Value = serde_json::from_str(other.text().unwrap_or(""))
                        .map_err(|e| err(e.to_string()))?;
                    crate::extension::validate_annotation(&value)?;
                    let descriptor = crate::extension::annotation_descriptor(&value)?;
                    state.technique = descriptor.technique;
                    state.qualifiers.extend(descriptor.qualifiers);
                }
            }
            for words in element_node
                .descendants()
                .filter(|n| n.has_tag_name("words"))
            {
                let term = rules::normalize(words.text().unwrap_or(""));
                match term.as_str() {
                    "pizz." | "pizzicato" => *pizzicato = true,
                    "arco" => *pizzicato = false,
                    "sul ponticello" => {
                        state
                            .qualifiers
                            .insert("urn:midi2workbench:position".into(), 0x0800_0000_u32.into());
                    }
                    "sul tasto" => {
                        state
                            .qualifiers
                            .insert("urn:midi2workbench:position".into(), 0xf000_0000_u32.into());
                    }
                    "con sordino" | "con sord." if !score.piano_routes.contains(&note.route) => {
                        state.qualifiers.insert(
                            "urn:midi2workbench:mute".into(),
                            serde_json::json!({"slot":1,"amount":u32::MAX}),
                        );
                    }
                    "senza sordino" | "senza sord."
                        if !score.piano_routes.contains(&note.route) =>
                    {
                        state.qualifiers.insert(
                            "urn:midi2workbench:mute".into(),
                            serde_json::json!({"slot":0,"amount":u32::MAX}),
                        );
                    }
                    "ordinario" | "ord." => {
                        state.technique = "core.normal".into();
                        *pizzicato = false;
                        state.qualifiers.clear();
                        state
                            .qualifiers
                            .insert("urn:midi2workbench:position".into(), 0x8000_0000_u32.into());
                    }
                    _ => {
                        if let Some((a, d)) = rules::term(&term) {
                            state.technique = rules::by_coordinate(a.at, a.subclass)
                                .expect("catalog rule")
                                .id
                                .into();
                            if let Some(d) = d {
                                state.qualifiers.extend(d.qualifiers);
                            }
                        } else {
                            score.report.record(
                                path,
                                "text annotation",
                                format!("preserved verbatim: {term}"),
                            );
                        }
                    }
                }
            }
            *next += 1;
        }
        if let Some((time, node)) = *envelope {
            let target = *envelope_targets
                .entry((time, node.range().start))
                .or_insert(note.onset);
            if target == note.onset {
                let kind = setting_transient(node).expect("transient node");
                let peak = node
                    .descendants()
                    .filter(|n| n.has_tag_name("sound"))
                    .find_map(|n| n.attribute("dynamics"))
                    .map(decimal)
                    .transpose()?
                    .map(|p| p * Rational::new(9, 10));
                let d = note.descriptor.get_or_insert_with(|| Descriptor {
                    technique: rules::by_coordinate(
                        note.articulation.at,
                        note.articulation.subclass,
                    )
                    .expect("validated category")
                    .id
                    .into(),
                    ..Default::default()
                });
                d.qualifiers.insert(
                    "urn:midi2workbench:dynamic-envelope".into(),
                    serde_json::json!({"kind":kind,"peak":peak}),
                );
            }
        }
        let pizzicato = note_nodes
            .get(path)
            .and_then(|n| n.attribute("pizzicato"))
            .map_or(*pizzicato, |v| v == "yes");
        if pizzicato || state.technique != "core.normal" || !state.qualifiers.is_empty() {
            let mut a = if pizzicato {
                Articulation::named("short.pizzicato")?
            } else {
                Articulation::named(&state.technique)?
            };
            let mut d = state.clone();
            if note.descriptor.as_ref().is_some_and(|d| {
                d.qualifiers
                    .contains_key("urn:midi2workbench:expanded-pattern")
            }) {
                rules::consume_pattern(&mut a, &mut d);
            }
            // Explicit note-level parameters override inherited playback state.
            if let Some(local) = &note.descriptor {
                for key in local.qualifiers.keys() {
                    d.qualifiers.remove(key);
                }
            }
            (note.articulation, note.descriptor) =
                rules::combine(note.articulation, note.descriptor.take(), a, Some(d))?;
        }
    }
    Ok(())
}

// MusicXML 4.0 sound attributes define jump labels and default visit numbers.
// Segments permit labels and jumps inside measures without inventing score nodes.
#[allow(
    clippy::too_many_lines,
    reason = "Keep the stateful traversal together; semantic rules are delegated without duplicating transient models."
)]
fn occurrences(measures: &[Measure<'_>]) -> Result<Vec<(usize, u32, Rational, Rational)>> {
    let mut sections = Vec::new();
    let mut first = Vec::new();
    for (mi, m) in measures.iter().enumerate() {
        first.push(sections.len());
        let mut cuts = BTreeSet::from([zero(), m.length]);
        for (t, _) in &m.navigation {
            if *t < zero() || *t > m.length {
                return Err(Diagnostic::new(
                    "E_NAVIGATION",
                    "sound navigation outside measure",
                ));
            }
            cuts.insert(*t);
        }
        let cuts = cuts.into_iter().collect::<Vec<_>>();
        for w in cuts.windows(2) {
            sections.push((mi, w[0], w[1]));
        }
        if m.length == zero() {
            sections.push((mi, zero(), zero()));
        }
    }
    let mut labels = BTreeMap::new();
    for (mi, m) in measures.iter().enumerate() {
        for (t, n) in &m.navigation {
            for kind in ["segno", "coda"] {
                if let Some(label) = n.attribute(kind) {
                    let destination = sections
                        .iter()
                        .position(|(i, lo, _)| *i == mi && lo == t)
                        .unwrap_or_else(|| first.get(mi + 1).copied().unwrap_or(sections.len()));
                    if labels
                        .insert((kind, label), destination)
                        .is_some_and(|old| old != destination)
                    {
                        return Err(Diagnostic::new(
                            "E_NAVIGATION",
                            "ambiguous navigation label",
                        ));
                    }
                }
            }
        }
    }
    let mut repeat_stack = Vec::new();
    let mut starts = BTreeMap::new();
    let mut ends = BTreeMap::new();
    let mut implicit = 0;
    for (mi, m) in measures.iter().enumerate() {
        if m.repeat_start {
            repeat_stack.push(mi);
        }
        if m.repeat_times.is_some() {
            let start = repeat_stack.pop().unwrap_or(implicit);
            starts.insert(mi, start);
            ends.insert(start, mi);
            implicit = mi + 1;
        }
    }
    let mut out = Vec::new();
    let mut i = 0;
    let mut pass = 1;
    let mut repeats = BTreeMap::<usize, u32>::new();
    let mut visits = BTreeMap::<(usize, usize), u32>::new();
    let mut returned = false;
    let mut steps = 0;
    while i < sections.len() {
        steps += 1;
        if steps > sections.len().saturating_mul(128) {
            return Err(Diagnostic::new(
                "E_NAVIGATION",
                "navigation graph exceeds finite bound",
            ));
        }
        let (mi, lo, hi) = sections[i];
        let m = &measures[mi];
        if lo == zero() {
            if let Some(end) = ends.get(&mi) {
                pass = repeats.get(end).copied().unwrap_or(1);
            }
        }
        let play = m.ending.is_empty() || m.ending.contains(&pass);
        if play {
            out.push((mi, pass, lo, hi));
        }
        let mut jump = None;
        if play {
            for (ni, (t, n)) in m
                .navigation
                .iter()
                .enumerate()
                .filter(|(_, (t, _))| *t == hi)
            {
                let _ = t;
                let visit = visits.entry((mi, ni)).or_default();
                *visit += 1;
                let explicit = n
                    .attribute("time-only")
                    .map(|s| {
                        s.split(',')
                            .map(|v| {
                                v.trim()
                                    .parse::<u32>()
                                    .map_err(|_| err("invalid navigation time-only"))
                            })
                            .collect::<Result<Vec<_>>>()
                    })
                    .transpose()?;
                let applies = |default| {
                    explicit
                        .as_ref()
                        .map_or(*visit == default, |p| p.contains(visit))
                };
                if returned
                    && n.attribute("fine").is_some()
                    && explicit.as_ref().is_none_or(|p| p.contains(visit))
                {
                    if n.attribute("fine") != Some("yes") {
                        return Err(Diagnostic::new(
                            "E_NAVIGATION",
                            "numeric fine requires an explicit final-duration policy",
                        ));
                    }
                    return Ok(out);
                }
                if n.attribute("dacapo") == Some("yes") && applies(1) {
                    jump = Some(0);
                    returned = true;
                }
                if let Some(label) = n.attribute("dalsegno").filter(|_| applies(1)) {
                    jump =
                        Some(*labels.get(&("segno", label)).ok_or_else(|| {
                            Diagnostic::new("E_NAVIGATION", "missing segno label")
                        })?);
                    returned = true;
                }
                if let Some(label) = n.attribute("tocoda").filter(|_| applies(2)) {
                    jump =
                        Some(*labels.get(&("coda", label)).ok_or_else(|| {
                            Diagnostic::new("E_NAVIGATION", "missing coda label")
                        })?);
                }
            }
        }
        if let Some(target) = jump {
            i = target;
            repeats.clear();
            pass = 1;
            continue;
        }
        if hi == m.length {
            if let Some(times) = m.repeat_times {
                if times == 0 || times > 32 {
                    return Err(Diagnostic::new(
                        "E_NAVIGATION",
                        "repeat count outside 1..32",
                    ));
                }
                let current = repeats.entry(mi).or_insert(1);
                if *current < times {
                    *current += 1;
                    pass = *current;
                    let start = *starts.get(&mi).unwrap_or(&0);
                    repeats.retain(|end, _| *end < start || *end >= mi);
                    i = first[start];
                    continue;
                }
                pass = *current;
            }
        }
        i += 1;
    }
    Ok(out)
}
fn read_pitch(p: Node<'_, '_>) -> Result<Rational> {
    let semitone = match text(p, "step").unwrap_or("") {
        "C" => 0,
        "D" => 2,
        "E" => 4,
        "F" => 5,
        "G" => 7,
        "A" => 9,
        "B" => 11,
        _ => return Err(err("invalid pitch step")),
    };
    let octave = number(p, "octave", integer(4))?;
    Ok((octave + integer(1)) * integer(12) + integer(semitone) + number(p, "alter", zero())?)
}
fn note_type(name: &str) -> Result<Rational> {
    Ok(match name {
        "maxima" => integer(32),
        "long" => integer(16),
        "breve" => integer(8),
        "whole" => integer(4),
        "half" => integer(2),
        "quarter" => integer(1),
        "eighth" => Rational::new(1, 2),
        "16th" => Rational::new(1, 4),
        "32nd" => Rational::new(1, 8),
        "64th" => Rational::new(1, 16),
        "128th" => Rational::new(1, 32),
        "256th" => Rational::new(1, 64),
        _ => return Err(err("unknown note type")),
    })
}
fn transient_kind(node: Node<'_, '_>) -> Option<&'static str> {
    let node = if node.has_tag_name("sound") {
        node.parent().filter(|p| p.has_tag_name("direction"))?
    } else {
        node
    };
    node.descendants()
        .find(|n| n.has_tag_name("dynamics"))?
        .children()
        .filter(Node::is_element)
        .find_map(|n| match n.tag_name().name() {
            "sf" => Some("sf"),
            "sfz" => Some("sfz"),
            "sffz" => Some("sffz"),
            "rf" => Some("rf"),
            "rfz" => Some("rfz"),
            "fp" => Some("fp"),
            _ => None,
        })
}
fn setting_transient(node: Node<'_, '_>) -> Option<&'static str> {
    if node.has_tag_name("sound") && !node.has_attribute("dynamics") {
        return None;
    }
    if node.has_tag_name("direction")
        && node
            .children()
            .any(|n| n.has_tag_name("sound") && n.has_attribute("dynamics"))
    {
        return None;
    }
    transient_kind(node)
}
fn setting_applies(node: Node<'_, '_>, pass: u32) -> Result<bool> {
    let Some(filter) = node
        .descendants()
        .find(|n| n.has_tag_name("sound") && n.has_attribute("time-only"))
        .and_then(|n| n.attribute("time-only"))
    else {
        return Ok(true);
    };
    Ok(filter
        .split(',')
        .map(|p| {
            p.trim()
                .parse::<u32>()
                .map_err(|_| err("invalid time-only"))
        })
        .collect::<Result<Vec<_>>>()?
        .contains(&pass))
}
fn playback_offset(node: Node<'_, '_>, default: Rational) -> Result<Rational> {
    child(node, "offset")
        .filter(|n| n.attribute("sound") != Some("no"))
        .map_or(Ok(default), |n| decimal(n.text().unwrap_or("0")))
}
fn source_part<'a>(node: Node<'a, 'a>) -> &'a str {
    node.ancestors()
        .find(|n| n.has_tag_name("part"))
        .and_then(|n| n.attribute("id"))
        .unwrap_or("")
}
fn steady_level(node: Node<'_, '_>, piano: bool) -> Result<Option<Rational>> {
    let explicit = node
        .descendants()
        .any(|n| n.has_tag_name("sound") && n.has_attribute("dynamics"));
    let transient = setting_transient(node);
    let level = if transient == Some("fp") {
        Some(integer(if piano { 53 } else { 45 }))
    } else if node.has_tag_name("sound") && transient.is_none() {
        node.attribute("dynamics")
            .map(decimal)
            .transpose()?
            .map(|d| d * Rational::new(9, 10))
    } else if !explicit {
        node.descendants()
            .find(|n| n.has_tag_name("dynamics"))
            .and_then(|d| {
                d.children().find_map(|m| {
                    if piano {
                        crate::piano::dynamic(m.tag_name().name())
                    } else {
                        dynamic_mark(m.tag_name().name())
                    }
                })
            })
            .map(integer)
    } else {
        None
    };
    Ok(level)
}
fn expression_points(
    settings: &[PlaybackSetting<'_>],
    piano_routes: &BTreeSet<Route>,
) -> Result<(Vec<DynamicPoint>, Vec<WedgePoint>)> {
    let mut dynamics = Vec::new();
    let mut wedges = Vec::new();
    for &(time, route, ref voice, ref staff, node) in settings {
        let level = steady_level(node, piano_routes.contains(&route))?;
        if let Some(level) = level {
            dynamics.push((
                time,
                route,
                voice.clone(),
                staff.clone(),
                level,
                source_part(node).to_string(),
            ));
        }
        if node.has_tag_name("direction") {
            for wedge in node.descendants().filter(|n| n.has_tag_name("wedge")) {
                let kind = wedge.attribute("type").unwrap_or("stop");
                if kind != "continue" {
                    wedges.push((
                        time,
                        route,
                        voice.clone(),
                        staff.clone(),
                        wedge.attribute("number").unwrap_or("1").into(),
                        kind.into(),
                        wedge.attribute("niente") == Some("yes"),
                        source_part(node).to_string(),
                    ));
                }
            }
        }
    }
    Ok((dynamics, wedges))
}

// The envelope is rendered once; actual decoded controller edits remain authoritative.
#[allow(
    clippy::too_many_lines,
    reason = "One envelope transformation uses the existing note and report types."
)]
fn realize_envelopes(
    score: &mut Performance,
    policy: &RenderingPolicy,
    scopes: &BTreeMap<String, (String, String, bool, bool)>,
    points: &[DynamicPoint],
    nodes: &BTreeMap<String, Node<'_, '_>>,
) -> Result<()> {
    if policy.transient_span <= zero() {
        return Err(Diagnostic::new(
            "E_POLICY",
            "transient span must be positive",
        ));
    }
    for note in &mut score.notes {
        let Some(envelope) = note
            .descriptor
            .as_ref()
            .and_then(|d| d.qualifiers.get("urn:midi2workbench:dynamic-envelope"))
            .cloned()
        else {
            continue;
        };
        if envelope["rendered"] == true {
            continue;
        }
        let kind = envelope["kind"]
            .as_str()
            .ok_or_else(|| err("missing dynamic envelope kind"))?;
        let explicit = scopes
            .get(note.source.split('#').next().unwrap_or(&note.source))
            .is_some_and(|s| s.2);
        let supplied: Option<Rational> =
            serde_json::from_value(envelope["peak"].clone()).map_err(|e| err(e.to_string()))?;
        let nominal = integer(if kind == "fp" {
            if score.piano_routes.contains(&note.route) {
                92
            } else {
                90
            }
        } else {
            (i64::from(note.velocity >> 9) + if kind.starts_with("rf") { 12 } else { 18 }).min(127)
        });
        let peak = if explicit {
            note.velocity
        } else {
            u16::try_from(quantized_level(
                supplied.unwrap_or(nominal),
                16,
                &mut score.report,
                &note.source,
            )?)
            .map_err(|_| err("velocity out of range"))?
        };
        if score.piano_routes.contains(&note.route) {
            note.velocity = peak;
            note.descriptor.as_mut().expect("envelope descriptor").qualifiers.insert("urn:midi2workbench:dynamic-envelope".into(),serde_json::json!({"kind":kind,"peak":peak,"rendered":true,"instrument":"struck piano; emphasis changes hammer velocity, natural decay is retained"}));
            score.report.record(&note.source,"piano transient interpretation","M2-126 §§4.1/6.1.2: attack emphasis changes hammer velocity; no invented sustained-note gain controller");
            continue;
        }
        let peak32 = if explicit {
            rules::upscale(u32::from(peak), 16, 32)
        } else {
            quantized_level(
                supplied.unwrap_or(nominal),
                32,
                &mut score.report,
                &note.source,
            )?
        };
        let context = scopes.get(note.source.split('#').next().unwrap_or(&note.source));
        let part = nodes
            .get(note.source.split('#').next().unwrap_or(&note.source))
            .map_or("", |n| source_part(*n));
        let nominal_baseline = points
            .iter()
            .enumerate()
            .filter(|(_, (t, r, v, s, _, p))| {
                *p == part
                    && *t <= note.onset
                    && *r == note.route
                    && v.as_ref()
                        .is_none_or(|v| context.is_some_and(|scope| scope.0 == *v))
                    && s.as_ref()
                        .is_none_or(|s| context.is_some_and(|scope| scope.1 == *s))
            })
            .max_by_key(|(i, (t, ..))| (*t, *i))
            .map_or(integer(72), |(_, (_, _, _, _, level, _))| *level);
        let baseline = quantized_level(nominal_baseline, 32, &mut score.report, &note.source)?;
        let descriptor = note.descriptor.as_mut().expect("envelope descriptor");
        let underlying = crate::extension::dynamic_curve(descriptor)?;
        let span = policy.transient_span.min((note.end - note.onset) / 2);
        let sustain = underlying
            .iter()
            .rev()
            .find(|(t, ..)| *t <= span)
            .map_or(baseline, |(_, v, _)| *v);
        let mut times = BTreeSet::from([zero(), span]);
        let mut sample = 1;
        while Rational::new(sample, 96) < span {
            times.insert(Rational::new(sample, 96));
            sample += 1;
        }
        let mut curve = Vec::new();
        for time in times {
            let fraction = time / span;
            let wide = num_rational::Ratio::new(
                i128::from(*fraction.numer()),
                i128::from(*fraction.denom()),
            );
            let value = num_rational::Ratio::from_integer(i128::from(peak32))
                + wide
                    * num_rational::Ratio::from_integer(i128::from(sustain) - i128::from(peak32));
            let encoded = u32::try_from(value.round().to_integer())
                .map_err(|_| err("dynamic envelope out of range"))?;
            score.report.quantized(
                &format!("{}#dynamic@{time}", note.source),
                "envelope/32",
                value,
                encoded,
            );
            curve.push((time, encoded, 0));
        }
        curve.extend(underlying.into_iter().filter(|(t, ..)| *t > span));
        descriptor.qualifiers.insert(
            crate::extension::DYNAMIC_CURVE.into(),
            serde_json::to_value(curve).map_err(|e| err(e.to_string()))?,
        );
        descriptor.qualifiers.insert(
            "urn:midi2workbench:dynamic-envelope".into(),
            serde_json::json!({"kind":kind,"peak":peak32,"span":span,"rendered":true}),
        );
        note.velocity = peak;
        score.report.record(
            &note.source,
            "transient dynamic policy",
            format!("{kind}: linear peak-to-steady envelope over {span} quarter"),
        );
    }
    Ok(())
}

fn dynamic_mark(mark: &str) -> Option<i64> {
    Some(match mark {
        "pppppp" => 4,
        "ppppp" => 8,
        "pppp" => 12,
        "ppp" => 20,
        "pp" => 32,
        "p" => 45,
        "mp" => 58,
        "mf" => 72,
        "f" => 90,
        "ff" => 104,
        "fff" => 116,
        "ffff" => 120,
        "fffff" => 124,
        "ffffff" => 127,
        _ => return None,
    })
}
fn instrument_messages(
    node: Node<'_, '_>,
    route: Route,
    report: &mut crate::model::Report,
    path: &str,
) -> Result<Vec<Vec<u32>>> {
    let mut messages = Vec::new();
    let bank = text(node, "midi-bank")
        .map(|v| v.parse::<u16>().map_err(|_| err("invalid MIDI bank")))
        .transpose()?;
    if bank.is_some_and(|b| b == 0 || b > 16384) {
        return Err(err("MIDI bank outside 1..16384"));
    }
    if let Some(program) = text(node, "midi-program") {
        let program = program
            .parse::<u8>()
            .map_err(|_| err("invalid MIDI program"))?;
        if !(1..=128).contains(&program) {
            return Err(Diagnostic::new(
                "E_INPUT_PROGRAM",
                "MIDI program outside 1..128",
            ));
        }
        messages.push(wire::program(route, program - 1, bank.map(|b| b - 1)));
    } else if bank.is_some() {
        return Err(Diagnostic::new(
            "E_BANK_PROGRAM_REQUIRED",
            "bank selection needs an explicit program",
        ));
    }
    if let Some(volume) = text(node, "volume") {
        let volume = decimal(volume)?;
        if volume < zero() || volume > integer(100) {
            return Err(err("volume outside 0..100"));
        }
        messages.push(wire::cc(
            route,
            7,
            quantized_level(volume * integer(127) / integer(100), 32, report, path)?,
        ));
    }
    if let Some(pan) = text(node, "pan") {
        let pan = decimal(pan)?;
        if pan < integer(-90) || pan > integer(90) {
            return Err(Diagnostic::new(
                "E_SPATIAL_BINDING",
                "rear azimuth needs a spatial receiver binding",
            ));
        }
        let nominal = integer(64) + pan / integer(90) * integer(if pan < zero() { 64 } else { 63 });
        messages.push(wire::cc(
            route,
            10,
            quantized_level(nominal, 32, report, path)?,
        ));
    }
    if text(node, "elevation")
        .map(decimal)
        .transpose()?
        .is_some_and(|v| v != zero())
    {
        return Err(Diagnostic::new(
            "E_SPATIAL_BINDING",
            "elevation needs a spatial receiver binding",
        ));
    }
    Ok(messages)
}

fn quantized_level(
    value: Rational,
    bits: u32,
    report: &mut crate::model::Report,
    source: &str,
) -> Result<u32> {
    let exact = rules::level_exact(value, bits)?;
    let encoded =
        u32::try_from(exact.round().to_integer()).map_err(|_| err("level out of range"))?;
    report.quantized(source, &format!("level/{bits}"), exact, encoded);
    Ok(encoded)
}
fn tempo_words(bpm: Rational, report: &mut crate::model::Report, source: &str) -> Result<Vec<u32>> {
    let words = wire::tempo_from_bpm(bpm)?;
    let wide = num_rational::Ratio::new(i128::from(*bpm.numer()), i128::from(*bpm.denom()));
    report.quantized(
        source,
        "tempo/10ns",
        num_rational::Ratio::from_integer(6_000_000_000_i128) / wide,
        words[1],
    );
    Ok(words)
}
fn velocity(level: Rational) -> Result<u16> {
    u16::try_from(rules::level(level, 16)?).map_err(|_| err("velocity out of range"))
}

type DynamicPoint = (
    Rational,
    Route,
    Option<String>,
    Option<String>,
    Rational,
    String,
);
type WedgePoint = (
    Rational,
    Route,
    Option<String>,
    Option<String>,
    String,
    String,
    bool,
    String,
);
type DynamicSpan = (
    Rational,
    Rational,
    Route,
    Option<String>,
    Option<String>,
    bool,
    bool,
    String,
);
type CurveSpan = (Rational, Rational, Rational, Rational);
fn curve_level(
    time: Rational,
    points: &[(Rational, Rational, u8)],
    spans: &[CurveSpan],
) -> Rational {
    if let Some((start, end, from, to)) = spans
        .iter()
        .rev()
        .find(|(start, end, ..)| *start <= time && time <= *end)
    {
        return *from + (*to - *from) * (time - *start) / (*end - *start);
    }
    points
        .iter()
        .enumerate()
        .filter(|(_, p)| p.0 <= time)
        .max_by_key(|(i, p)| (p.0, p.2, *i))
        .map_or(integer(72), |(_, p)| p.1)
}
#[allow(
    clippy::too_many_lines,
    reason = "Keep the stateful traversal together; semantic rules are delegated without duplicating transient models."
)]
fn realize_dynamics(
    score: &mut Performance,
    points: &[DynamicPoint],
    actions: &[WedgePoint],
    scopes: &BTreeMap<String, (String, String, bool, bool)>,
    nodes: &BTreeMap<String, Node<'_, '_>>,
) -> Result<()> {
    let mut active = BTreeMap::new();
    let mut spans = Vec::<DynamicSpan>::new();
    let mut actions = actions.to_vec();
    actions.sort_by_key(|a| a.0);
    let final_time = score.notes.iter().map(|n| n.end).max().unwrap_or_default();
    for (time, route, voice, staff, number, kind, niente, part) in actions {
        let key = (route, part.clone(), voice.clone(), staff.clone(), number);
        if let Some((start, crescendo, start_niente)) = active.remove(&key) {
            if time > start {
                spans.push((
                    start,
                    time,
                    route,
                    voice.clone(),
                    staff.clone(),
                    crescendo,
                    niente || start_niente,
                    part.clone(),
                ));
            }
        }
        if kind == "crescendo" || kind == "diminuendo" {
            active.insert(key, (time, kind == "crescendo", niente));
        }
    }
    for ((route, part, voice, staff, _), (start, crescendo, niente)) in active {
        if final_time > start {
            spans.push((
                start, final_time, route, voice, staff, crescendo, niente, part,
            ));
            score.report.record(
                "/",
                "wedge policy",
                "unterminated wedge ends at performance end",
            );
        }
    }
    spans.sort_by_key(|s| (s.0, usize::from(s.3.is_some()) + usize::from(s.4.is_some())));
    let mut cache = BTreeMap::<
        (Route, String, String, String),
        (Vec<CurveSpan>, Vec<(Rational, Rational, u8)>),
    >::new();
    for n in &mut score.notes {
        let path = n.source.split('#').next().unwrap_or(&n.source);
        let (voice, staff, explicit, accent) = scopes
            .get(path)
            .cloned()
            .unwrap_or_else(|| ("1".into(), "1".into(), false, false));
        let part = nodes.get(path).map_or("", |node| source_part(*node));
        let applies = |route: Route, v: &Option<String>, s: &Option<String>, p: &str| {
            route == n.route
                && p == part
                && v.as_ref().is_none_or(|v| *v == voice)
                && s.as_ref().is_none_or(|s| *s == staff)
        };
        let (resolved, levels) = cache
            .entry((n.route, part.to_string(), voice.clone(), staff.clone()))
            .or_insert_with(|| {
                let mut levels = points
                    .iter()
                    .filter(|(_, r, v, s, _, p)| applies(*r, v, s, p))
                    .map(|(t, _, _, _, l, _)| (*t, *l, 1))
                    .collect::<Vec<_>>();
                if score.piano_routes.contains(&n.route)
                    && !levels.iter().any(|(t, ..)| *t <= zero())
                {
                    levels.insert(0, (zero(), integer(79), 0));
                }
                let mut resolved = Vec::new();
                for (start, end, r, v, s, crescendo, niente, p) in &spans {
                    if !applies(*r, v, s, p) {
                        continue;
                    }
                    let mut from = curve_level(*start, &levels, &resolved);
                    if *niente && *crescendo {
                        from = zero();
                    }
                    let explicit_end = levels
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| p.0 == *end && p.2 == 1)
                        .max_by_key(|(i, _)| *i)
                        .map(|(_, p)| p.1);
                    let to = explicit_end.unwrap_or_else(|| {
                        if *niente && !*crescendo {
                            zero()
                        } else {
                            let nominal: &[i64] = if score.piano_routes.contains(&n.route) {
                                &[10, 25, 40, 53, 66, 79, 92, 104, 117, 125]
                            } else {
                                &[4, 8, 12, 20, 32, 45, 58, 72, 90, 104, 116, 120, 124, 127]
                            };
                            if *crescendo {
                                nominal
                                    .iter()
                                    .copied()
                                    .map(integer)
                                    .find(|l| *l > from)
                                    .unwrap_or(integer(127))
                            } else {
                                nominal
                                    .iter()
                                    .copied()
                                    .rev()
                                    .map(integer)
                                    .find(|l| *l < from)
                                    .unwrap_or(zero())
                            }
                        }
                    });
                    resolved.push((*start, *end, from, to));
                    levels.push((*end, to, 0));
                }
                (resolved, levels)
            });
        if !explicit {
            let level = curve_level(n.onset, levels, resolved);
            // A1: relative attack emphasis, never a claimed standard velocity
            // increment. Authored numerical playback bypasses this convention.
            let marks = nodes.get(path);
            let less = marks.is_some_and(|node| {
                node.descendants()
                    .any(|n| n.has_tag_name("unstress") || n.has_tag_name("soft-accent"))
            });
            let more = accent
                || marks.is_some_and(|node| node.descendants().any(|n| n.has_tag_name("stress")));
            let change = if less {
                -12
            } else if more {
                12
            } else {
                0
            };
            let nominal = if level == zero() {
                zero()
            } else {
                (level + integer(change)).clamp(integer(1), integer(127))
            };
            n.velocity = u16::try_from(quantized_level(nominal, 16, &mut score.report, &n.source)?)
                .map_err(|_| err("velocity out of range"))?;
        }
        if score.piano_routes.contains(&n.route) {
            continue;
        }
        if n.descriptor
            .as_ref()
            .is_some_and(|d| d.qualifiers.contains_key(crate::extension::DYNAMIC_CURVE))
        {
            continue;
        }
        if !resolved
            .iter()
            .any(|(start, end, ..)| *start < n.end && *end > n.onset)
        {
            continue;
        }
        let mut times = BTreeSet::from([n.onset, n.end]);
        for (start, end, ..) in resolved.iter() {
            let lo = (*start).max(n.onset);
            let hi = (*end).min(n.end);
            if lo >= hi {
                continue;
            }
            times.insert(lo);
            times.insert(hi);
            let mut sample = ((lo - *start) * integer(96)).ceil().to_integer();
            while *start + Rational::new(sample, 96) < hi {
                times.insert(*start + Rational::new(sample, 96));
                sample += 1;
            }
        }
        let curve = times
            .into_iter()
            .map(|t| {
                Ok((
                    t - n.onset,
                    quantized_level(
                        curve_level(t, levels, resolved),
                        32,
                        &mut score.report,
                        &format!("{}#dynamic@{}", n.source, t - n.onset),
                    )?,
                    0,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let descriptor = n.descriptor.get_or_insert_with(|| Descriptor {
            technique: rules::by_coordinate(n.articulation.at, n.articulation.subclass)
                .expect("validated category")
                .id
                .into(),
            ..Descriptor::default()
        });
        descriptor.qualifiers.insert(
            crate::extension::DYNAMIC_CURVE.into(),
            serde_json::to_value(curve).expect("curve JSON"),
        );
        score.report.record(
            &n.source,
            "wedge rendering policy",
            "exact nominal-level interpolation; bound assignable per-note dynamic-level controller",
        );
    }
    Ok(())
}

/// Exact ornament attacks; descriptors state that the pattern is already expanded,
/// so a receiver must not select a sampled pattern and play it a second time.
#[allow(
    clippy::too_many_lines,
    reason = "Keep the stateful traversal together; semantic rules are delegated without duplicating transient models."
)]
fn realize_pattern(
    mut note: Note,
    element_node: Node<'_, '_>,
    source_node: Node<'_, '_>,
    report: &mut crate::model::Report,
) -> Result<Vec<Note>> {
    let pattern = element_node.descendants().find(|n| {
        [
            "trill-mark",
            "shake",
            "mordent",
            "inverted-mordent",
            "turn",
            "inverted-turn",
            "tremolo",
        ]
        .contains(&n.tag_name().name())
    });
    let Some(pattern) = pattern else {
        return Ok(vec![note]);
    };
    let kind = pattern.tag_name().name();
    // MusicXML tremolo@type distinguishes a paired gesture before the mark
    // count is interpreted. Zero marks must not turn a start/stop pair into
    // independent same-pitch rolls (MusicXML 4.0 tremolo-type/tremolo).
    if kind == "tremolo" && matches!(pattern.attribute("type"), Some("start" | "stop")) {
        return Err(Diagnostic::new(
            "E_PATTERN_BINDING",
            "two-note tremolo needs paired-note realization",
        ));
    }
    if kind == "tremolo"
        && (pattern.attribute("type") == Some("unmeasured")
            || pattern.text().unwrap_or("0").trim() == "0")
    {
        (note.articulation, note.descriptor) = rules::combine(
            note.articulation,
            note.descriptor,
            Articulation::named("repeat.tremolo")?,
            None,
        )?;
        return Ok(vec![note]);
    }
    if pattern.attribute("accelerate") == Some("yes")
        || pattern
            .attribute("two-note-turn")
            .is_some_and(|v| v != "none")
    {
        return Err(Diagnostic::new(
            "E_TIMING_POLICY",
            format!(
                "{}: explicit accelerated/terminal ornament requires a complete timing schedule",
                note.source
            ),
        ));
    }
    let span = note.end - note.onset;
    let (positions, pitches) = if kind == "tremolo" {
        let strokes = pattern
            .text()
            .unwrap_or("0")
            .trim()
            .parse::<u32>()
            .map_err(|_| err("invalid tremolo strokes"))?;
        if strokes > 8 {
            return Err(err("tremolo strokes exceed supported range"));
        }
        let flags = match text(element_node, "type").unwrap_or("quarter") {
            "eighth" => 1,
            "16th" => 2,
            "32nd" => 3,
            "64th" => 4,
            "128th" => 5,
            _ => 0,
        };
        let mut pulse = Rational::new(1, 1_i64 << (strokes + flags));
        if let Some(tm) = child(element_node, "time-modification") {
            pulse *=
                number(tm, "normal-notes", integer(1))? / number(tm, "actual-notes", integer(1))?;
        }
        let count = span / pulse;
        if !count.is_integer() || count < integer(1) || count > integer(65536) {
            return Err(Diagnostic::new(
                "E_PATTERN_COUNT",
                "measured tremolo does not fit exact span",
            ));
        }
        (
            (0..count.to_integer())
                .map(|i| pulse * integer(i) / span)
                .collect::<Vec<_>>(),
            vec![
                note.pitch;
                usize::try_from(count.to_integer())
                    .map_err(|_| err("ornament count out of range"))?
            ],
        )
    } else {
        let mordent = kind.contains("mordent") || kind == "shake";
        let count = attr_number(pattern, "beats", integer(if mordent { 3 } else { 4 }))?;
        if !count.is_integer() || count < integer(3) || count > integer(256) {
            return Err(Diagnostic::new(
                "E_PATTERN_COUNT",
                "ornament needs integral count 3..256 or an authored schedule",
            ));
        }
        let count =
            usize::try_from(count.to_integer()).map_err(|_| err("ornament count out of range"))?;
        let second = attr_number(
            pattern,
            "second-beat",
            integer(if mordent { 12 } else { 25 }),
        )? / integer(100);
        let last = attr_number(pattern, "last-beat", integer(if mordent { 24 } else { 75 }))?
            / integer(100);
        if second <= zero() || last <= second || last >= integer(1) {
            return Err(Diagnostic::new(
                "E_PATTERN_TIME",
                "invalid ornament landing percentages",
            ));
        }
        let mut positions = vec![zero(), second];
        for i in 2..count {
            positions.push(
                second
                    + (last - second)
                        * Rational::new(
                            i64::try_from(i - 1).expect("bounded ornament count"),
                            i64::try_from(count - 2).expect("bounded ornament count"),
                        ),
            );
        }
        let step = match pattern.attribute("trill-step").unwrap_or("whole") {
            "half" => integer(1),
            "whole" => integer(2),
            "unison" => zero(),
            _ => return Err(err("unknown trill-step")),
        };
        let mut upper = note.pitch + step;
        let mut lower = note.pitch - step;
        for accidental in element_node.descendants().filter(|n| {
            n.has_tag_name("accidental-mark")
                && n.parent().is_some_and(|p| p.has_tag_name("ornaments"))
        }) {
            let alteration = match accidental.text().unwrap_or("") {
                "natural" => zero(),
                "sharp" => integer(1),
                "flat" => integer(-1),
                "double-sharp" | "sharp-sharp" => integer(2),
                "flat-flat" => integer(-2),
                "quarter-sharp" => Rational::new(1, 2),
                "quarter-flat" => Rational::new(-1, 2),
                "three-quarters-sharp" => Rational::new(3, 2),
                "three-quarters-flat" => Rational::new(-3, 2),
                _ => {
                    return Err(Diagnostic::new(
                        "E_ORNAMENT_ACCIDENTAL",
                        "ornament accidental needs an exact pitch binding",
                    ));
                }
            };
            let pitch = child(source_node, "pitch")
                .ok_or_else(|| err("ornament without a pitched source"))?;
            let natural = read_pitch(pitch)? - number(pitch, "alter", zero())?;
            let transposition = note.pitch - read_pitch(pitch)?;
            let below = accidental.attribute("placement") == Some("below");
            let distance = if below {
                if matches!(text(pitch, "step"), Some("C" | "F")) {
                    1
                } else {
                    2
                }
            } else if matches!(text(pitch, "step"), Some("E" | "B")) {
                1
            } else {
                2
            };
            let target = natural
                + integer(if below { -distance } else { distance })
                + alteration
                + transposition;
            if below {
                lower = target;
            } else {
                upper = target;
            }
        }
        let pitches = if kind == "turn" || kind == "inverted-turn" {
            (0..count)
                .map(|i| {
                    if i % 2 == 1 {
                        note.pitch
                    } else if (i == 0) ^ (kind == "inverted-turn") {
                        upper
                    } else {
                        lower
                    }
                })
                .collect()
        } else {
            let start =
                pattern
                    .attribute("start-note")
                    .unwrap_or(if mordent { "main" } else { "upper" });
            let start_main = start == "main";
            let aux = if start == "below" || kind == "mordent" && start != "upper" {
                lower
            } else {
                upper
            };
            (0..count)
                .map(|i| {
                    if (i % 2 == 0) == start_main {
                        note.pitch
                    } else {
                        aux
                    }
                })
                .collect()
        };
        (positions, pitches)
    };
    let mut out = Vec::with_capacity(positions.len());
    for (i, (&position, &pitch)) in positions.iter().zip(&pitches).enumerate() {
        let mut n = note.clone();
        n.onset = note.onset + span * position;
        n.end = positions
            .get(i + 1)
            .map_or(note.end, |p| note.onset + span * p);
        n.pitch = pitch;
        n.key = u8::try_from(round(pitch).clamp(0, 127)).expect("clamped key");
        let mut descriptor = n.descriptor.take().unwrap_or_else(|| Descriptor {
            technique: rules::by_coordinate(n.articulation.at, n.articulation.subclass)
                .expect("validated coordinate")
                .id
                .into(),
            ..Descriptor::default()
        });
        rules::consume_pattern(&mut n.articulation, &mut descriptor);
        descriptor.qualifiers.insert("urn:midi2workbench:expanded-pattern".into(),serde_json::json!({"kind":kind,"index":i,"count":positions.len(),"logicalDuration":span.to_string()}));
        n.descriptor = Some(descriptor);
        out.push(n);
    }
    report.record(
        &note.source,
        "ornament realization",
        format!(
            "{kind}: {} explicit attacks; sampled pattern consumed",
            out.len()
        ),
    );
    Ok(out)
}

#[must_use]
pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Upgrade the root declaration while preserving written notation lexically.
/// Original archive bytes remain a separate source artifact.
/// # Errors
/// Returns a diagnostic for malformed archives/XML or playback semantics that cannot be resolved.
pub fn source_aware_xml(source: &str) -> Result<String> {
    let doc = document(source)?;
    let root = doc.root_element();
    let mut edits = Vec::new();
    if let Some(version) = root.attributes().find(|a| a.name() == "version") {
        edits.push((version.range_value(), "4.0".to_string()));
    } else {
        let at = root.range().start
            + source[root.range().start..]
                .find('>')
                .ok_or_else(|| err("unclosed score tag"))?;
        edits.push((at..at, " version=\"4.0\"".to_string()));
    }
    for n in root.descendants().filter(Node::is_element) {
        if (n.has_tag_name("forward") || n.has_tag_name("backup"))
            && number(n, "duration", integer(1))? == zero()
        {
            edits.push((n.range(), String::new()));
        }
        if n.has_tag_name("pitch") {
            if let (Some(alter), Some(octave)) = (child(n, "alter"), child(n, "octave")) {
                if alter.range().start > octave.range().start {
                    let step = child(n, "step").ok_or_else(|| err("pitch without step"))?;
                    edits.push((
                        n.range(),
                        format!(
                            "<pitch>{}{}{}</pitch>",
                            &source[step.range()],
                            &source[alter.range()],
                            &source[octave.range()]
                        ),
                    ));
                }
            }
        }
    }
    edits.sort_by_key(|(range, _)| range.start);
    let mut result = source.to_string();
    for (range, replacement) in edits.into_iter().rev() {
        result.replace_range(range, &replacement);
    }
    if let Some(start) = result.find("<!DOCTYPE") {
        if let Some(end) = result[start..].find('>') {
            result.replace_range(start..=(start + end), "");
        }
    }
    Ok(result)
}

/// Canonical score reconstruction is serialized by musicxml, not a second writer.
/// # Errors
/// Returns a diagnostic for malformed archives/XML or playback semantics that cannot be resolved.
#[allow(
    clippy::too_many_lines,
    reason = "Keep the stateful traversal together; semantic rules are delegated without duplicating transient models."
)]
pub fn write(score: &Performance) -> Result<String> {
    let ticks = crate::clip::tpq(score)?;
    let mut routes = score
        .notes
        .iter()
        .map(|n| n.route)
        .chain(score.piano_routes.iter().copied())
        .collect::<BTreeSet<_>>();
    if routes.is_empty() {
        routes.insert(Route::default());
    }
    let mut xml = String::from("<score-partwise version=\"4.0\">");
    let metadata = score
        .configuration
        .iter()
        .filter(|e| wire::instrument_from_text(&e.words).is_none())
        .filter_map(|e| crate::remarks::metadata(&e.words))
        .collect::<Vec<_>>();
    if !metadata.is_empty() {
        xml.push_str("<identification><miscellaneous>");
        for value in metadata {
            let _ = write!(
                &mut xml,
                "<miscellaneous-field name=\"MIDI metadata text\">{}</miscellaneous-field>",
                escape(&value)
            );
        }
        xml.push_str("</miscellaneous></identification>");
    }
    xml.push_str("<part-list>");
    for (i, route) in routes.iter().enumerate() {
        let name = score
            .configuration
            .iter()
            .filter_map(|e| wire::instrument_from_text(&e.words))
            .find(|(r, _)| r == route)
            .map_or_else(
                || {
                    if score.piano_routes.contains(route) {
                        "Piano".into()
                    } else {
                        format!("Part {}", i + 1)
                    }
                },
                |(_, name)| name,
            );
        let name = escape(&name);
        let sound = if score.piano_routes.contains(route) {
            "<instrument-sound>keyboard.piano</instrument-sound>"
        } else {
            ""
        };
        let _ = write!(
            &mut xml,
            "<score-part id=\"P{i}\"><part-name>{name}</part-name><score-instrument id=\"I{i}\"><instrument-name>{name}</instrument-name>{sound}</score-instrument><midi-device port=\"{}\"/><midi-instrument id=\"I{i}\"><midi-channel>{}</midi-channel></midi-instrument></score-part>",
            route.group + 1,
            route.channel + 1
        );
    }
    xml.push_str("</part-list>");
    for (i, route) in routes.iter().enumerate() {
        let _ = write!(
            &mut xml,
            "<part id=\"P{i}\"><measure number=\"1\"><attributes><divisions>{ticks}</divisions></attributes>"
        );
        let mut cursor = zero();
        {
            for (configuration, e) in score
                .configuration
                .iter()
                .map(|e| (true, e))
                .chain(score.events.iter().map(|e| (false, e)))
            {
                let performance_text = crate::remarks::performance(&e.words);
                let target = performance_text.as_ref().and_then(|(r, _)| *r);
                if target.map_or(i != 0, |r| r != *route) {
                    continue;
                }
                let offset = round(e.time * integer(i64::from(ticks)));
                let payload = serde_json::json!({"words":e.words,"configuration":configuration,"order":e.order});
                let direction_type = performance_text.map_or_else(
                    || "<other-direction print-object=\"no\"/>".to_string(),
                    |(_, value)| format!("<words>{}</words>", escape(&value)),
                );
                let context_fields = crate::remarks::scope(&e.words);
                let mut context = String::new();
                if let Some(fields) = context_fields {
                    for tag in ["voice", "staff"] {
                        if let Some(value) = fields[tag].as_str() {
                            let _ = write!(&mut context, "<{tag}>{}</{tag}>", escape(value));
                        }
                    }
                }
                let _ = write!(
                    &mut xml,
                    "<direction><direction-type>{direction_type}</direction-type><offset>{offset}</offset>{context}<sound><play><other-play type=\"urn:midi2workbench:ump:v1\">{}</other-play></play></sound></direction>",
                    escape(&payload.to_string())
                );
            }
        }
        for n in score.notes.iter().filter(|n| n.route == *route) {
            let delta = n.onset - cursor;
            if delta < zero() {
                let _ = write!(
                    &mut xml,
                    "<backup><duration>{}</duration></backup>",
                    round(-delta * integer(i64::from(ticks)))
                );
            } else if delta > zero() {
                let _ = write!(
                    &mut xml,
                    "<forward><duration>{}</duration></forward>",
                    round(delta * integer(i64::from(ticks)))
                );
            }
            let pitch = round(n.pitch).clamp(0, 127);
            let (step, alter) = match pitch % 12 {
                0 => ("C", 0),
                1 => ("C", 1),
                2 => ("D", 0),
                3 => ("D", 1),
                4 => ("E", 0),
                5 => ("F", 0),
                6 => ("F", 1),
                7 => ("G", 0),
                8 => ("G", 1),
                9 => ("A", 0),
                10 => ("A", 1),
                _ => ("B", 0),
            };
            let row = rules::by_coordinate(n.articulation.at, n.articulation.subclass).ok_or_else(
                || {
                    Diagnostic::new(
                        "E_CUSTOM_MAP",
                        "custom articulation requires inverse semantic map",
                    )
                },
            )?;
            let mut descriptor = n.descriptor.clone().unwrap_or_else(|| Descriptor {
                technique: row.id.into(),
                ..Descriptor::default()
            });
            descriptor.qualifiers.insert(
                "urn:midi2workbench:attack-velocity".into(),
                serde_json::json!(n.velocity),
            );
            let payload = serde_json::json!({"technique":descriptor.technique,"qualifiers":descriptor.qualifiers,"variation":n.articulation.variation,"direction":n.articulation.direction,"string":n.articulation.string,"resetRoundRobin":n.articulation.reset_round_robin,"release":(["none","soft","hard","rise","fall"][usize::from(n.articulation.release)]),"releaseTime":n.release_velocity,"onOrder":n.on_order,"offOrder":n.off_order});
            let notation = match rules::native_notation(n.articulation) {
                Some("") => String::new(),
                Some(fragment) => format!("<notations>{fragment}</notations>"),
                None => format!(
                    "<notations><technical><other-technical>{}</other-technical></technical></notations>",
                    escape(row.term)
                ),
            };
            let mut notation = notation;
            if score.piano_routes.contains(route) {
                if let Some(mark) = crate::piano::dynamic_label(n.velocity) {
                    let _ = write!(
                        &mut notation,
                        "<notations><dynamics><{mark}/></dynamics></notations>"
                    );
                }
            }
            if let Some(marks) = descriptor
                .qualifiers
                .get("urn:midi2workbench:source-articulation-marks")
                .and_then(serde_json::Value::as_array)
            {
                for mark in marks
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .filter(|v| ["tenuto", "stress", "unstress", "soft-accent"].contains(v))
                {
                    let _ = write!(
                        &mut notation,
                        "<notations><articulations><{mark}/></articulations></notations>"
                    );
                }
            }
            if let Some(values) = descriptor
                .qualifiers
                .get(crate::remarks::QUALIFIER)
                .and_then(serde_json::Value::as_array)
            {
                for value in values {
                    let Some(fragment) = value["xml"].as_str() else {
                        continue;
                    };
                    let parsed = document(fragment)?;
                    let node = parsed.root_element();
                    let kind = node.tag_name().name();
                    let parent = match kind {
                        "other-articulation" => Some("articulations"),
                        "other-ornament" => Some("ornaments"),
                        "other-dynamics" => Some("dynamics"),
                        "other-notation" => Some(""),
                        "other-technical" | "open-string" | "thumb-position" | "fingering"
                        | "pluck" | "double-tongue" | "triple-tongue" | "stopped" | "fret"
                        | "hammer-on" | "pull-off" | "bend" | "tap" | "heel" | "toe"
                        | "fingernails" | "hole" | "arrow" | "handbell" | "brass-bend" | "flip"
                        | "smear" | "open" | "half-muted" | "harmon-mute" | "golpe" => {
                            Some("technical")
                        }
                        _ => None,
                    };
                    if let Some(parent) = parent {
                        // Repeats/ornaments can realize one written instruction
                        // several times. Its source ID stays in the descriptor;
                        // reusing that ID on multiple native elements is invalid.
                        let mut native = fragment.to_string();
                        let ids = parsed
                            .descendants()
                            .filter(Node::is_element)
                            .flat_map(|node| node.attributes())
                            .filter(|a| a.name() == "id" && a.namespace().is_none())
                            .map(|a| a.range())
                            .collect::<Vec<_>>();
                        for range in ids.into_iter().rev() {
                            native.replace_range(range, "");
                        }
                        let fragment = &native;
                        if parent.is_empty() {
                            let _ = write!(&mut notation, "<notations>{fragment}</notations>");
                        } else {
                            let _ = write!(
                                &mut notation,
                                "<notations><{parent}>{fragment}</{parent}></notations>"
                            );
                        }
                    }
                }
            }
            let mut note_attributes = String::new();
            if (n.articulation.at, n.articulation.subclass) == (0x11, 10) {
                note_attributes.push_str(" pizzicato=\"yes\"");
            }
            if let Some(value) = descriptor
                .qualifiers
                .get("urn:midi2workbench:musicxml-end-dynamics")
                .and_then(serde_json::Value::as_str)
            {
                let _ = write!(&mut note_attributes, " end-dynamics=\"{}\"", escape(value));
            }
            let footnote = descriptor
                .qualifiers
                .get(crate::remarks::QUALIFIER)
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .find(|r| r["kind"] == "footnote")
                .and_then(|r| r["text"].as_str())
                .map_or_else(String::new, |value| {
                    format!("<footnote>{}</footnote>", escape(value))
                });
            let mut note_context = String::new();
            if let Some(fields) = descriptor
                .qualifiers
                .get(crate::remarks::CONTEXT)
                .or_else(|| {
                    descriptor
                        .qualifiers
                        .get(crate::remarks::QUALIFIER)
                        .and_then(serde_json::Value::as_array)
                        .and_then(|v| v.first())
                })
            {
                for tag in ["voice", "staff"] {
                    if let Some(value) = fields[tag].as_str() {
                        let _ = write!(&mut note_context, "<{tag}>{}</{tag}>", escape(value));
                    }
                }
            }
            let fraction = n.pitch - integer(pitch) + integer(alter);
            let decimal_alter =
                rust_decimal::Decimal::from_i128_with_scale(i128::from(*fraction.numer()), 0)
                    / rust_decimal::Decimal::from_i128_with_scale(i128::from(*fraction.denom()), 0);
            let _ = write!(
                &mut xml,
                "<note{note_attributes}><pitch><step>{step}</step><alter>{decimal_alter}</alter><octave>{}</octave></pitch><duration>{}</duration>{footnote}{note_context}{notation}<play><other-play type=\"{}\">{}</other-play></play></note>",
                pitch / 12 - 1,
                round((n.end - n.onset) * integer(i64::from(ticks))),
                crate::extension::XML_TYPE,
                escape(&payload.to_string())
            );
            cursor = n.end;
        }
        if score.duration > cursor {
            let _ = write!(
                &mut xml,
                "<forward><duration>{}</duration></forward>",
                round((score.duration - cursor) * integer(i64::from(ticks)))
            );
        }
        xml.push_str("</measure></part>");
    }
    xml.push_str("</score-partwise>");
    serialize_canonical(xml)
}

/// Shared typed musicxml serializer; the lexical adapter only repairs its
/// integer-backed pitch alteration field. Used by both reconstruction paths.
pub(crate) fn serialize_canonical(xml: String) -> Result<String> {
    // Retain the upstream writer while restoring the exact decimal alteration
    // its integer-backed Semitones type cannot represent.
    let doc = document(&xml)?;
    let alterations = doc
        .descendants()
        .filter(|n| n.has_tag_name("alter"))
        .map(|n| n.text().unwrap_or("0").to_string())
        .collect::<Vec<_>>();
    let repairs = doc
        .descendants()
        .filter(|n| n.has_tag_name("alter"))
        .filter_map(|n| n.first_child())
        .filter(|n| n.text().is_some_and(|t| t.parse::<i16>().is_err()))
        .map(|n| n.range())
        .collect::<Vec<_>>();
    let mut writer_input = xml;
    for range in repairs.into_iter().rev() {
        writer_input.replace_range(range, "0");
    }
    let typed = musicxml::read_score_data_partwise(writer_input.into_bytes()).map_err(err)?;
    let data = musicxml::write_partwise_score_data(&typed, false, false).map_err(err)?;
    let mut written = String::from_utf8(data).map_err(|e| err(e.to_string()))?;
    let doc = document(&written)?;
    let ranges = doc
        .descendants()
        .filter(|n| n.has_tag_name("alter"))
        .filter_map(|n| n.first_child())
        .map(|n| n.range())
        .collect::<Vec<_>>();
    if ranges.len() != alterations.len() {
        return Err(Diagnostic::new(
            "E_XML_WRITE",
            "upstream writer discarded pitched notes",
        ));
    }
    for (range, value) in ranges.into_iter().zip(alterations).rev() {
        written.replace_range(range, &value);
    }
    source_aware_xml(&written)
}
/// # Errors
/// Returns a diagnostic for malformed archives/XML or playback semantics that cannot be resolved.
pub fn compress(source: &str) -> Result<Vec<u8>> {
    // Reusing the source envelope avoids losing unknown/legacy notation in a reparse.
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("META-INF/container.xml", opts)
        .map_err(|e| err(e.to_string()))?;
    zip.write_all(b"<?xml version=\"1.0\"?><container><rootfiles><rootfile full-path=\"score.musicxml\" media-type=\"application/vnd.recordare.musicxml+xml\"/></rootfiles></container>").map_err(|e|err(e.to_string()))?;
    zip.start_file("score.musicxml", opts)
        .map_err(|e| err(e.to_string()))?;
    zip.write_all(source.as_bytes())
        .map_err(|e| err(e.to_string()))?;
    Ok(zip.finish().map_err(|e| err(e.to_string()))?.into_inner())
}
