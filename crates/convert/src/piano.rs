//! M2-126-UM v1.0 (published 2026-07-22): acoustic Piano Profile rules.
//! Profile identity, dynamic centers and capabilities below are normative facts.
//! Instrument selection is a semantic deduction from MusicXML instrument identity,
//! never from a piano GM program alone (a program number without a declared sound set does not identify a timbre).
use crate::{Diagnostic, Result, model::Route};
pub const PROFILE_ID: [u8; 5] = [0x7e, 0x20, 0, 1, 1];
/// M2-126 §4.1.1 Table 6, MIDI-1 center values; M2-115 supplies 16-bit scaling.
#[must_use]
pub fn dynamic(mark: &str) -> Option<i64> {
    Some(match mark {
        "pppp" => 10,
        "ppp" => 25,
        "pp" => 40,
        "p" => 53,
        "mp" => 66,
        "mf" => 79,
        "f" => 92,
        "ff" => 104,
        "fff" => 117,
        "ffff" => 125,
        _ => return None,
    })
}
/// Explicit MusicXML sound IDs outrank human labels. Labels are needed for older
/// exports without instrument-sound. A concrete non-piano part name outranks an
/// inconsistent generic piano score-instrument label. M2-126 §2.1 excludes electric pianos.
#[must_use]
pub fn identifies(part: &str, instrument: &str, sound: &str) -> bool {
    if !sound.is_empty() {
        return sound == "keyboard.piano"
            || sound.starts_with("keyboard.piano.")
            || sound == "keyboard.fortepiano";
    }
    let part = crate::rules::normalize(part);
    let instrument = crate::rules::normalize(instrument);
    let piano = |name: &str| {
        matches!(
            name,
            "piano"
                | "grand piano"
                | "acoustic grand piano"
                | "acoustic piano"
                | "fortepiano"
                | "pianoforte"
        )
    };
    piano(&part) || matches!(part.as_str(), "" | "musicxml part" | "part") && piano(&instrument)
}
/// M2-126 §10 Table 23: validate the 31-byte capability layout, including reserved bits.
/// # Errors
/// Rejects incorrect length, non-7-bit bytes, reserved flags and contradictory ranges.
pub fn validate_details(data: &[u8]) -> Result<()> {
    if data.len() != 31 || data.iter().any(|v| *v > 127) {
        return Err(Diagnostic::new(
            "E_PIANO_DETAILS",
            "expected 31 seven-bit bytes",
        ));
    }
    for (index, mask) in [
        (0, 0x78),
        (2, 0x40),
        (4, 0x70),
        (7, 0x78),
        (8, 0x7e),
        (14, 0x40),
        (16, 0x70),
        (19, 0x78),
        (20, 0x7e),
    ] {
        if data[index] & mask != 0 {
            return Err(Diagnostic::new(
                "E_PIANO_DETAILS",
                "reserved capability bits",
            ));
        }
    }
    if data[0] & 6 != 0 && data[25] > data[26] {
        return Err(Diagnostic::new(
            "E_PIANO_DETAILS",
            "lowest playable note exceeds highest",
        ));
    }
    Ok(())
}
/// M2-126 §10: this file converter is a Sender, not an acoustic reproducing unit.
#[must_use]
pub fn sender_details() -> Vec<u8> {
    let mut data = vec![0; 31];
    data[0] = 1;
    data[1] = 0x7f;
    data
}
/// M2-126 §§5.1.3, 8.1: Attribute 0x04 holds key velocity, not an articulation.
/// Reuse the upstream note builder and adapt only its missing attribute kind.
/// # Errors
/// Returns the upstream/common note-field diagnostic for an invalid address.
pub fn note(
    route: Route,
    key: u8,
    hammer: u16,
    key_velocity: Option<u16>,
    on: bool,
) -> Result<Vec<u32>> {
    let mut words = crate::wire::note(
        route,
        key,
        hammer,
        crate::rules::Articulation::default(),
        on,
    )?;
    words[0] &= !255;
    words[1] &= 0xffff_0000;
    if on {
        if let Some(value) = key_velocity {
            words[0] |= 4;
            words[1] |= u32::from(value);
        }
    }
    Ok(words)
}

/// M2-126 §4.1.1 Table 6 inclusive MIDI 2.0 ranges, not nearest-center guesses.
/// Silent attacks remain real notes; their key/damper actions can matter.
#[must_use]
pub fn dynamic_label(velocity: u16) -> Option<&'static str> {
    Some(match velocity {
        0..=0x03ff => return None,
        0x0400..=0x23ff => "pppp",
        0x2400..=0x41ff => "ppp",
        0x4200..=0x5dff => "pp",
        0x5e00..=0x77ff => "p",
        0x7800..=0x9248 => "mp",
        0x9249..=0xacb1 => "mf",
        0xacb2..=0xc71b => "f",
        0xc71c..=0xe185 => "ff",
        0xe186..=0xf7de => "fff",
        0xf7df..=0xffff => "ffff",
    })
}
