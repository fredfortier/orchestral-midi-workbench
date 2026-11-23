//! The small checked orchestral attribute gap around upstream midi2 primitives.
use crate::{Diagnostic, Result, model::Route, rules::Articulation};
use midi2::{channel_voice2, prelude::*};

/// # Errors
/// Returns a diagnostic for invalid orchestral fields or malformed UMP messages.
pub fn note(route: Route, key: u8, velocity: u16, art: Articulation, on: bool) -> Result<Vec<u32>> {
    if route.group > 15 || route.channel > 15 || key > 127 {
        return Err(Diagnostic::new(
            "E_ADDRESS",
            "group/channel/key out of range",
        ));
    }
    art.validate()?;
    // Upstream builds the supported note fields; local words own the attribute.
    let mut message = channel_voice2::NoteOn::<[u32; 2]>::new();
    message.set_group(u4::new(route.group));
    message.set_channel(u4::new(route.channel));
    message.set_note_number(u7::new(key));
    message.set_velocity(velocity);
    let mut data = message.data().to_vec();
    data[0] |= u32::from(if on { art.at } else { 0x10 });
    if !on {
        data[0] = (data[0] & !0x00f0_0000) | 0x0080_0000;
    }
    data[1] |= u32::from(if on {
        art.data()?
    } else {
        u16::from(art.release) << 12 | u16::from(art.string)
    });
    Ok(data)
}

/// Standard polyphonic pressure using upstream's checked field layout.
#[must_use]
pub fn pressure(route: Route, key: u8, value: u32) -> Vec<u32> {
    let mut m = channel_voice2::KeyPressure::<[u32; 2]>::new();
    m.set_group(u4::new(route.group));
    m.set_channel(u4::new(route.channel));
    m.set_note_number(u7::new(key));
    m.set_key_pressure_data(value);
    m.data().to_vec()
}

#[must_use]
pub fn controller(route: Route, key: u8, index: u8, value: u32, registered: bool) -> Vec<u32> {
    if registered {
        let mut m = channel_voice2::RegisteredPerNoteController::<[u32; 2]>::new();
        m.set_group(u4::new(route.group));
        m.set_channel(u4::new(route.channel));
        m.set_note_number(u7::new(key));
        m.set_controller(match index {
            7 => channel_voice2::Controller::Volume(value),
            10 => channel_voice2::Controller::Pan(value),
            _ => channel_voice2::Controller::Pitch7_25(midi2::num::Fixed7_25::from_bits(value)),
        });
        let mut words = m.data().to_vec();
        words[0] = (words[0] & !255) | u32::from(index);
        words
    } else {
        let mut m = channel_voice2::AssignablePerNoteController::<[u32; 2]>::new();
        m.set_group(u4::new(route.group));
        m.set_channel(u4::new(route.channel));
        m.set_note_number(u7::new(key));
        m.set_index(index);
        m.set_controller_data(value);
        m.data().to_vec()
    }
}
#[must_use]
pub fn cc(route: Route, index: u8, value: u32) -> Vec<u32> {
    let mut m = channel_voice2::ControlChange::<[u32; 2]>::new();
    m.set_group(u4::new(route.group));
    m.set_channel(u4::new(route.channel));
    m.set_control(u7::new(index));
    m.set_control_change_data(value);
    m.data().to_vec()
}
#[must_use]
pub fn registered_channel(route: Route, bank: u8, index: u8, value: u32) -> Vec<u32> {
    let mut m = channel_voice2::RegisteredController::<[u32; 2]>::new();
    m.set_group(u4::new(route.group));
    m.set_channel(u4::new(route.channel));
    m.set_bank(u7::new(bank));
    m.set_index(u7::new(index));
    m.set_controller_data(value);
    m.data().to_vec()
}
pub fn program(route: Route, program: u8, bank: Option<u16>) -> Vec<u32> {
    let mut m = channel_voice2::ProgramChange::<[u32; 2]>::new();
    m.set_group(u4::new(route.group));
    m.set_channel(u4::new(route.channel));
    m.set_program(u7::new(program));
    m.set_bank(bank.map(midi2::ux::u14::new));
    let mut words = m.data().to_vec();
    if let Some(bank) = bank {
        // 0.11.1 uses SysEx LSB-first packing here; M2-104 Figure 64
        // instead places Bank MSB before Bank LSB in Channel Voice.
        words[1] = (words[1] & 0xffff_0000) | (u32::from(bank >> 7) << 8) | u32::from(bank & 127);
    }
    words
}
#[must_use]
pub fn detach_reset(route: Route, key: u8) -> Vec<u32> {
    let mut m = channel_voice2::PerNoteManagement::<[u32; 2]>::new();
    m.set_group(u4::new(route.group));
    m.set_channel(u4::new(route.channel));
    m.set_note_number(u7::new(key));
    m.set_detach(true);
    m.set_reset(true);
    m.data().to_vec()
}
#[must_use]
pub fn tempo(value: u32) -> Vec<u32> {
    let mut message = midi2::flex_data::SetTempo::<[u32; 4]>::new();
    message.set_number_of_10_nanosecond_units_per_quarter_note(value);
    message.data().to_vec()
}
#[must_use]
pub fn meter(beats: u8, exponent: u8) -> Vec<u32> {
    let mut message = midi2::flex_data::SetTimeSignature::<[u32; 4]>::new();
    message.set_numerator(beats);
    message.set_denominator(exponent);
    // M2-104 v1.1.2 §7.5.4: this is 1/32 notes per 24 MIDI Clocks,
    // not clocks per quarter. Score/clip positions here use quarter-note beats.
    message.set_number_of_32nd_notes(8);
    message.data().to_vec()
}
/// Convert exact quarter-note BPM at the Flex Data numeric boundary.
/// # Errors
/// Returns a diagnostic for nonpositive or unrepresentable tempos.
pub fn tempo_from_bpm(bpm: crate::model::Rational) -> Result<Vec<u32>> {
    if bpm <= crate::model::Rational::from_integer(0) {
        return Err(Diagnostic::new("E_TEMPO", "tempo must be positive"));
    }
    let wide = num_rational::Ratio::new(i128::from(*bpm.numer()), i128::from(*bpm.denom()));
    let units = (num_rational::Ratio::from_integer(6_000_000_000_i128) / wide)
        .round()
        .to_integer();
    let units = u32::try_from(units)
        .ok()
        .filter(|v| *v != 0)
        .ok_or_else(|| Diagnostic::new("E_TEMPO", "tempo does not fit Flex Data"))?;
    Ok(tempo(units))
}

#[must_use]
pub fn packet_len(word: u32) -> usize {
    // M2-104 Table 1; reserved types have a defined size but no inferred meaning.
    [1, 1, 1, 2, 2, 4, 1, 1, 2, 2, 2, 3, 3, 4, 4, 4][(word >> 28) as usize]
}
/// # Errors
/// Returns a diagnostic for invalid orchestral fields or malformed UMP messages.
pub fn validate_event(words: &[u32], configuration: bool) -> Result<()> {
    validate(words)?;
    let head = words[0];
    let kind = head >> 28;
    let status = (head >> 20) & 15;
    if (kind == 0 && [3, 4].contains(&status))
        || (kind == 15 && [0x20, 0x21].contains(&((head >> 16) & 0x3ff)))
        || (!configuration && kind == 4 && [8, 9].contains(&status))
    {
        return Err(Diagnostic::new(
            "E_RAW_EVENT",
            "notes and clip framing must use the score/timeline model",
        ));
    }
    Ok(())
}
/// # Errors
/// Returns a diagnostic for invalid orchestral fields or malformed UMP messages.
pub fn validate(words: &[u32]) -> Result<()> {
    let Some(&head) = words.first() else {
        return Err(Diagnostic::new("E_UMP", "empty message"));
    };
    if head >> 28 == 0xd && head & 0xffff == 0x010d {
        return instrument_from_text(words)
            .map(|_| ())
            .ok_or_else(|| Diagnostic::new("E_FLEX", "invalid Piano Description text"));
    }
    if words.len() == 2
        && head >> 28 == 4
        && (head >> 20) & 15 == 9
        && head.to_le_bytes()[0] == 4
        && head & 0x8000 == 0
    {
        return Ok(());
    }
    if head >> 28 == 0xd && head & 0x0000_ff00 != 0 {
        let checked = crate::remarks::validation_words(words)
            .ok_or_else(|| Diagnostic::new("E_FLEX", "Invalid text framing or UTF-8"))?;
        return midi2::UmpMessage::try_from(checked.as_ref())
            .map(|_| ())
            .map_err(|e| Diagnostic::new("E_FLEX", format!("{e:?}")));
    }
    if words.len() != packet_len(head) {
        return Err(Diagnostic::new("E_UMP", "truncated or oversized packet"));
    }
    if head >> 28 == 3 {
        let count = ((head >> 16) & 15) as usize;
        let form = (head >> 20) & 15;
        let bytes = words
            .iter()
            .flat_map(|w| w.to_be_bytes())
            .collect::<Vec<_>>();
        if count > 6
            || form > 3
            || bytes[2..2 + count].iter().any(|b| *b > 127)
            || bytes[2 + count..].iter().any(|b| *b != 0)
        {
            return Err(Diagnostic::new("E_SYSEX", "invalid SysEx7 packet"));
        }
        return Ok(());
    }
    let status = (head >> 20) & 15;
    if head >> 28 == 4
        && (status == 9 || status == 8)
        && (0x10..=0x1f).contains(&(head.to_le_bytes()[0]))
    {
        if head & 0x8000 != 0 {
            return Err(Diagnostic::new("E_RESERVED", "reserved note-number bit"));
        }
        if status == 9 {
            Articulation::from_data(
                head.to_le_bytes()[0],
                u16::from_le_bytes([words[1].to_le_bytes()[0], words[1].to_le_bytes()[1]]),
            )?;
        } else if head.to_le_bytes()[0] != 0x10
            || words[1] & 0x0ff8 != 0
            || ((words[1] >> 12) & 15) > 4
        {
            return Err(Diagnostic::new(
                "E_RESERVED",
                "invalid Note Off articulation",
            ));
        }
        return Ok(());
    }
    // M2-123 §8.2 defines position after the baseline crate's controller enum.
    if head >> 28 == 4
        && status == 0
        && (12..=15).contains(&head.to_le_bytes()[0])
        && head & 0x8000 == 0
    {
        return Ok(());
    }
    midi2::UmpMessage::try_from(words)
        .map(|_| ())
        .map_err(|e| Diagnostic::new("E_UMP", format!("{e:?}")))
}

pub(crate) fn route(words: &[u32]) -> Option<Route> {
    let head = *words.first()?;
    if head >> 28 == 4 {
        Some(Route {
            group: head.to_be_bytes()[0] & 15,
            channel: head.to_be_bytes()[1] & 15,
        })
    } else {
        instrument_from_text(words)
            .map(|(route, _)| route)
            .or_else(|| crate::remarks::performance(words).and_then(|(route, _)| route))
    }
}

pub(crate) fn channel_cc(words: &[u32]) -> Option<(Route, u8)> {
    let [head, _] = words else { return None };
    if head >> 28 != 4 || (head >> 20) & 15 != 0xb {
        return None;
    }
    Some((route(words)?, ((head >> 8) & 127) as u8))
}

pub(crate) fn reroute(words: &[u32], route: Route) -> Result<Vec<u32>> {
    if let Ok(message) = midi2::flex_data::UnknownPerformanceText::try_from(words) {
        return Ok(crate::remarks::text(Some(route), &message.text(), false));
    }
    if let Some((_, name)) = instrument_from_text(words) {
        return Ok(instrument_text(route, &name, words[0] & 0xffff == 0x010d));
    }
    let parsed = midi2::channel_voice2::ChannelVoice2::try_from(words)
        .map_err(|e| Diagnostic::new("E_ROUTE_MESSAGE", format!("{e:?}")))?;
    let mut message: midi2::channel_voice2::ChannelVoice2<Vec<u32>> = parsed.rebuffer_into();
    message.set_group(u4::new(route.group));
    message.set_channel(u4::new(route.channel));
    Ok(message.data().to_vec())
}

/// M2-126 §8.5 defines Piano Description (bank 1/status 0x0D).
/// Other instruments use M2-104 generic metadata with a declared identity convention;
/// the Piano-specific status is never reassigned to orchestral instruments.
#[must_use]
/// # Panics
/// Panics only if serializing a Rust string fails, which `serde_json` cannot do.
pub fn instrument_text(route: Route, name: &str, piano: bool) -> Vec<u32> {
    let mut message = midi2::flex_data::UnknownMetadataText::<Vec<u32>>::new();
    message.set_group(u4::new(route.group));
    message.set_optional_channel(Some(u4::new(route.channel)));
    message.set_text(&if piano {
        name.to_string()
    } else {
        format!(
            "urn:midi2workbench:instrument:v1:{}",
            serde_json::to_string(name).expect("string JSON")
        )
    });
    let mut words = message.data().to_vec();
    if piano {
        for packet in words.chunks_exact_mut(4) {
            packet[0] |= 0x0d;
        }
    }
    words
}
#[must_use]
pub fn instrument_from_text(words: &[u32]) -> Option<(Route, String)> {
    let head = *words.first()?;
    if head >> 28 != 0xd || !matches!(head & 0xffff, 0x0100 | 0x010d) || head & 0x0030_0000 != 0 {
        return None;
    }
    let mut common = words.to_vec();
    for packet in common.chunks_exact_mut(4) {
        packet[0] &= !255;
    }
    let message = midi2::flex_data::UnknownMetadataText::try_from(common.as_slice()).ok()?;
    Some((
        Route {
            group: head.to_be_bytes()[0] & 15,
            channel: head.to_be_bytes()[1] & 15,
        },
        if head & 0xffff == 0x010d {
            message.text()
        } else {
            serde_json::from_str(
                message
                    .text()
                    .strip_prefix("urn:midi2workbench:instrument:v1:")?,
            )
            .ok()?
        },
    ))
}
