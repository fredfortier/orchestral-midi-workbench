//! Conservative measured notation, reusing the exact writer's musical note payloads.
use crate::{
    Diagnostic, Result, arrangement,
    model::{Performance, Rational, Route},
    remarks, wire, xml,
};
use std::{collections::BTreeMap, fmt::Write};
fn zero() -> Rational {
    Rational::from_integer(0)
}
fn err(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("E_NOTATION", message)
}
fn snap(time: Rational) -> Rational {
    for division in [1, 2, 3, 4, 6, 8, 12, 16, 24, 32] {
        let grid = Rational::from_integer(division);
        let candidate = Rational::new((time * grid).round().to_integer(), division);
        if candidate - time <= Rational::new(1, 96) && time - candidate <= Rational::new(1, 96) {
            return candidate;
        }
    }
    time
}
fn rhythm(duration: Rational) -> String {
    for (name, base) in [
        ("whole", Rational::from_integer(4)),
        ("half", Rational::from_integer(2)),
        ("quarter", Rational::from_integer(1)),
        ("eighth", Rational::new(1, 2)),
        ("16th", Rational::new(1, 4)),
        ("32nd", Rational::new(1, 8)),
        ("64th", Rational::new(1, 16)),
        ("128th", Rational::new(1, 32)),
    ] {
        if duration == base {
            return format!("<type>{name}</type>");
        }
        if duration == base * Rational::new(3, 2) {
            return format!("<type>{name}</type><dot/>");
        }
        if duration == base * Rational::new(2, 3) {
            return format!(
                "<type>{name}</type><time-modification><actual-notes>3</actual-notes><normal-notes>2</normal-notes></time-modification>"
            );
        }
    }
    String::new()
}
fn raw<'a>(node: roxmltree::Node<'a, 'a>) -> &'a str {
    &node.document().input_text()[node.range()]
}
#[derive(Clone)]
struct Written<'a> {
    onset: Rational,
    end: Rational,
    note: roxmltree::Node<'a, 'a>,
    voice: usize,
    chord: bool,
    lyric: Option<String>,
}
/// # Errors
/// Returns invalid part ownership, unresolved clocks or malformed exact-writer output.
pub fn write(score: &Performance) -> Result<String> {
    let Some(arrangement) = arrangement::read(score)? else {
        return xml::write(score);
    };
    let exact = xml::write(score)?;
    let document = roxmltree::Document::parse(&exact).map_err(|e| err(e.to_string()))?;
    let routes: Vec<_> = score
        .notes
        .iter()
        .map(|n| n.route)
        .chain(score.piano_routes.iter().copied())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut fragments = BTreeMap::<Route, Vec<_>>::new();
    for (route, part) in routes.iter().zip(
        document
            .root_element()
            .children()
            .filter(|n| n.has_tag_name("part")),
    ) {
        fragments.insert(
            *route,
            part.descendants()
                .filter(|n| n.has_tag_name("note"))
                .collect::<Vec<_>>(),
        );
    }
    let mut notation = score.clone();
    for n in &mut notation.notes {
        let onset = snap(n.onset);
        let end = snap(n.end);
        if end > onset {
            n.onset = onset;
            n.end = end;
        }
    }
    // The notation grid is independent of the performed clip grid.
    let ticks = crate::clip::tpq(&notation)?;
    let bars = measure_grid(score)?;
    let mut out = String::from(
        "<score-partwise version=\"4.0\"><work><work-title>Reconstructed performance</work-title></work><part-list>",
    );
    for (i, p) in arrangement.parts.iter().enumerate() {
        let _ = write!(
            out,
            "<score-part id=\"P{i}\"><part-name>{}</part-name></score-part>",
            xml::escape(&p.name)
        );
    }
    out.push_str("</part-list>");
    for (part_index, part) in arrangement.parts.iter().enumerate() {
        let _ = write!(out, "<part id=\"P{part_index}\">");
        let (written, voice_count) = part_notes(score, &notation, part, &fragments)?;
        for (bar_index, &(from, to, numerator, denominator)) in bars.iter().enumerate() {
            let _ = write!(out, "<measure number=\"{}\">", bar_index + 1);
            let changed = bar_index == 0
                || bars[bar_index - 1].2 != numerator
                || bars[bar_index - 1].3 != denominator;
            if changed {
                write_attributes(
                    &mut out,
                    score,
                    part,
                    ticks,
                    numerator,
                    denominator,
                    bar_index == 0,
                );
            }
            write_directions(&mut out, score, part, part_index, from, to, ticks);
            write_voices(&mut out, &written, voice_count, from, to, ticks);
            out.push_str("</measure>");
        }
        out.push_str("</part>");
    }
    out.push_str("</score-partwise>");
    xml::serialize_canonical(out)
}
fn rest(out: &mut String, duration: i64, voice: usize, rhythm: &str) {
    let _ = write!(
        out,
        "<note><rest/><duration>{duration}</duration><voice>{voice}</voice>{rhythm}</note>"
    );
}
fn render_note(
    out: &mut String,
    n: &Written<'_>,
    duration: i64,
    voice: usize,
    stop: bool,
    start: bool,
    rhythm: &str,
) {
    out.push_str("<note");
    for a in n.note.attributes() {
        let _ = write!(out, " {}=\"{}\"", a.name(), xml::escape(a.value()));
    }
    out.push('>');
    if n.chord {
        out.push_str("<chord/>");
    }
    for c in n
        .note
        .children()
        .filter(|c| c.has_tag_name("pitch") || c.has_tag_name("unpitched"))
    {
        out.push_str(raw(c));
    }
    let _ = write!(out, "<duration>{duration}</duration>");
    if stop {
        out.push_str("<tie type=\"stop\"/>");
    }
    if start {
        out.push_str("<tie type=\"start\"/>");
    }
    for c in n
        .note
        .children()
        .filter(|c| c.has_tag_name("footnote") || c.has_tag_name("level"))
    {
        out.push_str(raw(c));
    }
    let _ = write!(out, "<voice>{voice}</voice>{rhythm}");
    for c in n.note.children().filter(|c| c.has_tag_name("notations")) {
        out.push_str(raw(c));
    }
    if stop || start {
        out.push_str("<notations>");
        if stop {
            out.push_str("<tied type=\"stop\"/>");
        }
        if start {
            out.push_str("<tied type=\"start\"/>");
        }
        out.push_str("</notations>");
    }
    if !stop {
        if let Some(lyric) = &n.lyric {
            out.push_str(lyric);
        }
    }
    for c in n.note.children().filter(|c| c.has_tag_name("play")) {
        out.push_str(raw(c));
    }
    out.push_str("</note>");
}

type Bar = (Rational, Rational, u8, u16);
fn measure_grid(score: &Performance) -> Result<Vec<Bar>> {
    let mut meters = vec![(zero(), 4_u8, 4_u16)];
    for e in &score.events {
        if e.words[0] >> 28 == 0xd && e.words[0] & 0xffff == 1 {
            let numerator =
                u8::try_from((e.words[1] >> 24) & 255).map_err(|_| err("meter numerator"))?;
            let power = (e.words[1] >> 16) & 255;
            let denominator = 1_u16
                .checked_shl(power)
                .ok_or_else(|| err("meter denominator"))?;
            if numerator == 0 {
                return Err(err("zero meter numerator"));
            }
            meters.push((e.time, numerator, denominator));
        }
    }
    meters.sort_by_key(|m| m.0);
    let mut bars = Vec::new();
    let mut start = zero();
    while start < score.duration {
        if bars.len() > 100_000 {
            return Err(err("too many measures"));
        }
        let (_, numerator, denominator) = *meters
            .iter()
            .rev()
            .find(|m| m.0 <= start)
            .ok_or_else(|| err("missing meter"))?;
        let nominal = start + Rational::new(i64::from(numerator) * 4, i64::from(denominator));
        let end = meters
            .iter()
            .find(|m| m.0 > start && m.0 < nominal)
            .map_or(nominal, |m| m.0)
            .min(score.duration);
        bars.push((start, end, numerator, denominator));
        start = end;
    }
    Ok(bars)
}
fn part_notes<'a>(
    score: &Performance,
    notation: &Performance,
    part: &arrangement::Part,
    fragments: &BTreeMap<Route, Vec<roxmltree::Node<'a, 'a>>>,
) -> Result<(Vec<Written<'a>>, usize)> {
    let mut written = Vec::new();
    for route in &part.routes {
        let source: Vec<_> = score.notes.iter().filter(|n| n.route == *route).collect();
        let rendered: Vec<_> = notation
            .notes
            .iter()
            .filter(|n| n.route == *route)
            .collect();
        let payload = fragments.get(route).map_or(&[][..], Vec::as_slice);
        if source.len() != payload.len() {
            return Err(err("exact writer note ownership differs"));
        }
        for (n, fragment) in rendered.into_iter().zip(payload) {
            written.push(Written {
                onset: n.onset,
                end: n.end,
                note: *fragment,
                voice: 0,
                chord: false,
                lyric: None,
            });
        }
    }
    written.sort_by_key(|n| (n.onset, n.end));
    let mut voices: Vec<(Rational, Rational)> = Vec::new();
    for n in &mut written {
        if let Some(i) = voices
            .iter()
            .position(|(on, end)| *on == n.onset && *end == n.end)
        {
            n.voice = i;
            n.chord = true;
        } else {
            let i = voices
                .iter()
                .position(|(_, end)| *end <= n.onset)
                .unwrap_or(voices.len());
            n.voice = i;
            if i == voices.len() {
                voices.push((n.onset, n.end));
            } else {
                voices[i] = (n.onset, n.end);
            }
        }
    }
    let lyrics: Vec<_> = score
        .events
        .iter()
        .filter(|e| e.words[0] >> 28 == 0xd && e.words[0] & 0xffff == 0x0201)
        .collect();
    let mut previous_word_end = true;
    for n in written.iter_mut().filter(|n| !n.chord) {
        let lyric = lyrics.iter().find(|e| {
            snap(e.time) == n.onset
                && e.words[0] >> 28 == 0xd
                && e.words[0] & 0xffff == 0x0201
                && text_route(&e.words).is_none_or(|r| part.routes.contains(&r))
        });
        if let Some(e) = lyric {
            if let Some(text) = remarks::flex_text(&e.words) {
                if text.is_empty() {
                    n.lyric = Some("<lyric><extend type=\"continue\"/></lyric>".into());
                } else {
                    let word_end = text.ends_with([' ', '\r', '\n']);
                    let syllabic = match (previous_word_end, word_end) {
                        (true, true) => "single",
                        (true, false) => "begin",
                        (false, true) => "end",
                        (false, false) => "middle",
                    };
                    n.lyric = Some(format!(
                        "<lyric><syllabic>{syllabic}</syllabic><text>{}</text></lyric>",
                        xml::escape(text.trim_end())
                    ));
                    previous_word_end = word_end;
                }
            }
        }
    }
    Ok((written, voices.len()))
}
fn write_directions(
    out: &mut String,
    score: &Performance,
    part: &arrangement::Part,
    part_index: usize,
    from: Rational,
    to: Rational,
    ticks: u16,
) {
    let tick = |t: Rational| {
        (t * Rational::from_integer(i64::from(ticks)))
            .round()
            .to_integer()
    };
    for e in &score.events {
        if e.time < from || e.time >= to {
            continue;
        }
        let target = wire::route(&e.words);
        if target.is_some_and(|r| !part.routes.contains(&r))
            || (target.is_none() && part_index != 0)
        {
            continue;
        }
        if e.words[0] >> 28 == 0xd && matches!(e.words[0] & 0xffff, 0x0201 | 0x0202) {
            continue;
        }
        let offset = tick(e.time - from);
        if e.words[0] >> 28 == 0xd && e.words[0].trailing_zeros() >= 16 {
            let bpm = 6_000_000_000.0 / f64::from(e.words[1]);
            let mark = if e.time == zero() {
                format!(
                    "<metronome><beat-unit>quarter</beat-unit><per-minute>{bpm:.2}</per-minute></metronome>"
                )
            } else {
                "<other-direction print-object=\"no\"/>".into()
            };
            let _ = write!(
                out,
                "<direction><direction-type>{mark}</direction-type><offset>{offset}</offset><sound tempo=\"{bpm:.8}\"/></direction>"
            );
        } else if let Some((_, text)) = remarks::performance(&e.words) {
            if !text.starts_with(arrangement::COMMAND) {
                let _ = write!(
                    out,
                    "<direction><direction-type><words>{}</words></direction-type><offset>{offset}</offset></direction>",
                    xml::escape(&text)
                );
            }
        } else if e.words[0] & 0xffff != 1 || e.words[0] >> 28 != 0xd {
            let payload =
                serde_json::json!({"words":e.words,"configuration":false,"order":e.order});
            let _ = write!(
                out,
                "<direction><direction-type><other-direction print-object=\"no\"/></direction-type><offset>{offset}</offset><sound><play><other-play type=\"urn:midi2workbench:ump:v1\">{}</other-play></play></sound></direction>",
                xml::escape(&payload.to_string())
            );
        }
    }
}

fn text_route(words: &[u32]) -> Option<Route> {
    let h = *words.first()?;
    if h >> 28 == 0xd && h & 0x0030_0000 == 0 {
        Some(Route {
            group: u8::try_from((h >> 24) & 15).ok()?,
            channel: u8::try_from((h >> 16) & 15).ok()?,
        })
    } else {
        None
    }
}
/// Conservative editorial decisions, separate from the actual performed clip values.
#[must_use]
pub fn report(score: &Performance) -> crate::model::Report {
    let mut report = score.report.clone();
    for (i, n) in score.notes.iter().enumerate() {
        let onset = snap(n.onset);
        let end = snap(n.end);
        if end <= onset {
            report.record(
                format!("note[{i}]"),
                "exceptional short duration",
                format!(
                    "Kept exact {}..{} beats; snapping would erase the note",
                    n.onset, n.end
                ),
            );
        }
        if end > onset && (onset != n.onset || end != n.end) {
            report.record(
                format!("note[{i}]"),
                "bounded rhythm inference",
                format!(
                    "{}..{} -> {onset}..{end} quarter-note beats",
                    n.onset, n.end
                ),
            );
        }
    }
    report.record("score","editorial notation","Deterministic voices and spelling; no authored slurs, spelling or lyric alignment inferred from receiver names");
    report
}

fn write_voices(
    out: &mut String,
    written: &[Written<'_>],
    voice_count: usize,
    from: Rational,
    to: Rational,
    ticks: u16,
) {
    let tick = |t: Rational| {
        (t * Rational::from_integer(i64::from(ticks)))
            .round()
            .to_integer()
    };
    // Like music21's per-measure makeVoices step, only material present in this
    // measure needs a voice. Empty global voices must not add colliding bar rests.
    let mut active: Vec<_> = (0..voice_count.max(1))
        .filter(|voice| {
            written
                .iter()
                .any(|n| n.voice == *voice && n.onset < to && n.end > from)
        })
        .collect();
    if active.is_empty() {
        active.push(0);
    }
    for (index, voice) in active.into_iter().enumerate() {
        if index > 0 {
            let _ = write!(
                out,
                "<backup><duration>{}</duration></backup>",
                tick(to - from)
            );
        }
        let mut cursor = from;
        for n in written
            .iter()
            .filter(|n| n.voice == voice && n.onset < to && n.end > from)
        {
            let onset = n.onset.max(from);
            let end = n.end.min(to);
            if !n.chord && onset > cursor {
                rest(
                    out,
                    tick(onset - cursor),
                    voice + 1,
                    &rhythm(onset - cursor),
                );
            }
            render_note(
                out,
                n,
                tick(end - onset),
                voice + 1,
                n.onset < from,
                n.end > to,
                &rhythm(end - onset),
            );
            if !n.chord {
                cursor = end;
            }
        }
        if cursor < to {
            rest(out, tick(to - cursor), voice + 1, &rhythm(to - cursor));
        }
    }
}

fn write_attributes(
    out: &mut String,
    score: &Performance,
    part: &arrangement::Part,
    ticks: u16,
    numerator: u8,
    denominator: u16,
    first: bool,
) {
    out.push_str("<attributes>");
    if first {
        let _ = write!(out, "<divisions>{ticks}</divisions>");
    }
    let _ = write!(
        out,
        "<time><beats>{numerator}</beats><beat-type>{denominator}</beat-type></time>"
    );
    if first {
        let (sum, count) = score
            .notes
            .iter()
            .filter(|n| part.routes.contains(&n.route))
            .fold((0_u64, 0_u64), |(s, c), n| (s + u64::from(n.key), c + 1));
        let (sign, line) = if count > 0 && sum < count * 60 {
            ("F", 4)
        } else {
            ("G", 2)
        };
        let _ = write!(out, "<clef><sign>{sign}</sign><line>{line}</line></clef>");
    }
    out.push_str("</attributes>");
}
