//! M2-101 §7 profile codecs and deterministic transaction state.
//!
//! Payloads here exclude F0/F7; upstream Sysex7 owns transport packetization.
use crate::{Diagnostic, Result, rules::PROFILE_ID};
use midi2::ci::Ci;
use midi2::{ci::DeviceId, prelude::*};
use std::collections::{BTreeMap, BTreeSet};

pub const BROADCAST: u32 = 0x0fff_ffff;
pub type ProfileId = [u8; 5];
/// M2-116 §6.2's file-only exception: both MUIDs are broadcast. This is a tag,
/// never an executable live CI transaction or evidence of negotiation.
/// # Errors
/// Returns a diagnostic for invalid fields, unsupported capabilities or failed protocol transactions.
pub fn file_profile_declaration(group: u8, channel: u8) -> Result<Vec<u32>> {
    file_profile(group, channel, PROFILE_ID)
}
/// M2-116 §6.2 file declaration using an actual standard Profile ID.
/// # Errors
/// Rejects invalid group/channel or malformed Profile IDs.
pub fn file_profile(group: u8, channel: u8, profile: ProfileId) -> Result<Vec<u32>> {
    if group > 15 {
        return Err(Diagnostic::new("E_CI_ADDRESS", "group outside 0..15"));
    }
    let message = Message::channel(
        channel,
        0,
        0,
        Body::Set {
            on: true,
            profile,
            channels: 1,
        },
    )?;
    let mut payload = message.encode()?;
    payload[5..13].fill(127);
    let mut sysex = midi2::sysex7::Sysex7::<Vec<u32>>::new();
    sysex.set_group(u4::new(group));
    sysex.set_payload(payload.into_iter().map(u7::new));
    Ok(sysex.data().to_vec())
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Body {
    Inquiry,
    Reply {
        enabled: Vec<ProfileId>,
        disabled: Vec<ProfileId>,
    },
    Set {
        on: bool,
        profile: ProfileId,
        channels: u16,
    },
    Report {
        on: bool,
        profile: ProfileId,
        channels: u16,
    },
    Added {
        profile: ProfileId,
    },
    Removed {
        profile: ProfileId,
    },
    Details {
        profile: ProfileId,
        target: u8,
    },
    DetailsReply {
        profile: ProfileId,
        target: u8,
        data: Vec<u8>,
    },
    Specific {
        profile: ProfileId,
        data: Vec<u8>,
    },
    Ack {
        negative: bool,
        original: u8,
        status: u8,
        status_data: u8,
        details: [u8; 5],
        data: Vec<u8>,
    },
    Invalidate {
        muid: u32,
    },
    PeCapabilities {
        reply: bool,
        simultaneous: u8,
        major: u8,
        minor: u8,
    },
    Property {
        reply: bool,
        request: u8,
        header: Vec<u8>,
        total: u16,
        current: u16,
        data: Vec<u8>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub address: DeviceId,
    pub version: u8,
    pub source: u28,
    pub destination: u28,
    pub body: Body,
}
fn address_byte(address: DeviceId) -> u8 {
    match address {
        DeviceId::Channel(c) => c.into(),
        DeviceId::Group => 0x7e,
        DeviceId::FunctionBlock => 0x7f,
    }
}
#[must_use]
pub fn pack(mut value: u32, count: usize) -> Vec<u8> {
    let mut out = Vec::new();
    for _ in 0..count {
        out.push((value & 127) as u8);
        value >>= 7;
    }
    out
}
fn packed_length(length: usize, width: usize) -> Result<Vec<u8>> {
    let value = u32::try_from(length)
        .map_err(|_| Diagnostic::new("E_CI_LENGTH", "length exceeds 32 bits"))?;
    if value >> (7 * width) != 0 {
        return Err(Diagnostic::new("E_CI_LENGTH", "length exceeds wire field"));
    }
    Ok(pack(value, width))
}
fn number(bytes: &[u8]) -> u32 {
    bytes.iter().rev().fold(0, |a, b| a << 7 | u32::from(*b))
}
struct Reader<'a> {
    data: &'a [u8],
    position: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self
            .position
            .checked_add(n)
            .ok_or_else(|| Diagnostic::new("E_CI_LENGTH", "overflow"))?;
        let out = self
            .data
            .get(self.position..end)
            .ok_or_else(|| Diagnostic::new("E_CI_LENGTH", "truncated profile message"))?;
        self.position = end;
        Ok(out)
    }
    fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn num(&mut self, n: usize) -> Result<u32> {
        Ok(number(self.take(n)?))
    }
    fn profile(&mut self) -> Result<ProfileId> {
        Ok(self.take(5)?.try_into().expect("five bytes"))
    }
    fn profiles(&mut self) -> Result<Vec<ProfileId>> {
        let count = self.num(2)?;
        (0..count).map(|_| self.profile()).collect()
    }
}
impl Message {
    /// # Errors
    /// Returns a diagnostic for invalid fields, unsupported capabilities or failed protocol transactions.
    pub fn channel(channel: u8, source: u32, destination: u32, body: Body) -> Result<Self> {
        if channel > 15 || source >= BROADCAST || destination > BROADCAST {
            return Err(Diagnostic::new("E_CI_ADDRESS", "invalid channel/MUID"));
        }
        Ok(Self {
            address: DeviceId::Channel(u4::new(channel)),
            version: 2,
            source: u28::new(source),
            destination: u28::new(destination),
            body,
        })
    }
    #[must_use]
    pub fn status(&self) -> u8 {
        match self.body {
            Body::Inquiry => 0x20,
            Body::Reply { .. } => 0x21,
            Body::Set { on: true, .. } => 0x22,
            Body::Set { on: false, .. } => 0x23,
            Body::Report { on: true, .. } => 0x24,
            Body::Report { on: false, .. } => 0x25,
            Body::Added { .. } => 0x26,
            Body::Removed { .. } => 0x27,
            Body::Details { .. } => 0x28,
            Body::DetailsReply { .. } => 0x29,
            Body::Specific { .. } => 0x2f,
            Body::Ack {
                negative: false, ..
            } => 0x7d,
            Body::Ack { negative: true, .. } => 0x7f,
            Body::Invalidate { .. } => 0x7e,
            Body::PeCapabilities { reply: false, .. } => 0x30,
            Body::PeCapabilities { reply: true, .. } => 0x31,
            Body::Property { reply: false, .. } => 0x34,
            Body::Property { reply: true, .. } => 0x35,
        }
    }
    /// # Errors
    /// Returns a diagnostic for invalid fields, unsupported capabilities or failed protocol transactions.
    #[allow(
        clippy::too_many_lines,
        reason = "The finite protocol-family match keeps field order adjacent to each message definition."
    )]
    pub fn encode(&self) -> Result<Vec<u8>> {
        let broadcast = matches!(
            self.body,
            Body::Report { .. }
                | Body::Added { .. }
                | Body::Removed { .. }
                | Body::Invalidate { .. }
        );
        if broadcast != (u32::from(self.destination) == BROADCAST) {
            return Err(Diagnostic::new(
                "E_CI_ADDRESS",
                "destination does not match this message's broadcast rule",
            ));
        }
        if matches!(self.body, Body::Invalidate { .. }) && self.address != DeviceId::FunctionBlock {
            return Err(Diagnostic::new(
                "E_CI_ADDRESS",
                "Invalidate MUID addresses the Function Block",
            ));
        }
        if u32::from(self.source) >= BROADCAST {
            return Err(Diagnostic::new(
                "E_CI_ADDRESS",
                "broadcast MUID cannot be a source",
            ));
        }
        if ![1, 2].contains(&self.version)
            || (self.version == 1
                && matches!(
                    self.body,
                    Body::Added { .. }
                        | Body::Removed { .. }
                        | Body::Details { .. }
                        | Body::DetailsReply { .. }
                ))
        {
            return Err(Diagnostic::new(
                "E_CI_VERSION",
                "message requires MIDI-CI version 2",
            ));
        }
        let mut out = vec![
            0x7e,
            address_byte(self.address),
            0x0d,
            self.status(),
            self.version,
        ];
        out.extend(pack(self.source.into(), 4));
        out.extend(pack(self.destination.into(), 4));
        match &self.body {
            Body::Inquiry => {}
            Body::Reply { enabled, disabled } => {
                if enabled.iter().any(|p| disabled.contains(p)) {
                    return Err(Diagnostic::new(
                        "E_PROFILE_STATE",
                        "profile appears in both enabled and disabled lists",
                    ));
                }
                for list in [enabled, disabled] {
                    if list.len() > 16383 {
                        return Err(Diagnostic::new("E_CI_LENGTH", "too many profiles"));
                    }
                    out.extend(packed_length(list.len(), 2)?);
                    for id in list {
                        out.extend(id);
                    }
                }
            }
            Body::Set {
                profile, channels, ..
            }
            | Body::Report {
                profile, channels, ..
            } => {
                if *channels > 16383 {
                    return Err(Diagnostic::new(
                        "E_CI_CHANNELS",
                        "14-bit channel count required",
                    ));
                }
                if matches!(self.body, Body::Set { on: false, .. }) && *channels != 0 {
                    return Err(Diagnostic::new(
                        "E_RESERVED",
                        "Set Profile Off channel field is reserved and must be zero",
                    ));
                }
                out.extend(profile);
                if self.version == 2 {
                    out.extend(pack(u32::from(*channels), 2));
                }
            }
            Body::Added { profile } | Body::Removed { profile } => out.extend(profile),
            Body::Details { profile, target } => {
                out.extend(profile);
                out.push(*target);
            }
            Body::DetailsReply {
                profile,
                target,
                data,
            } => {
                if data.len() > 16383 {
                    return Err(Diagnostic::new(
                        "E_CI_LENGTH",
                        "details length exceeds 14 bits",
                    ));
                }
                out.extend(profile);
                out.push(*target);
                out.extend(packed_length(data.len(), 2)?);
                out.extend(data);
            }
            Body::Specific { profile, data } => {
                out.extend(profile);
                out.extend(packed_length(data.len(), 4)?);
                out.extend(data);
            }
            Body::Ack {
                original,
                status,
                status_data,
                details,
                data,
                ..
            } => {
                if self.version == 2 {
                    out.extend([*original, *status, *status_data]);
                    out.extend(details);
                    out.extend(packed_length(data.len(), 2)?);
                    out.extend(data);
                }
            }
            Body::Invalidate { muid } => {
                if *muid > BROADCAST {
                    return Err(Diagnostic::new(
                        "E_CI_ADDRESS",
                        "invalidated MUID exceeds 28 bits",
                    ));
                }
                out.extend(pack(*muid, 4));
            }
            Body::PeCapabilities {
                simultaneous,
                major,
                minor,
                ..
            } => {
                out.push(*simultaneous);
                if self.version == 2 {
                    out.extend([*major, *minor]);
                }
            }
            Body::Property {
                request,
                header,
                total,
                current,
                data,
                reply,
            } => {
                if header.len() > 16383
                    || data.len() > 16383
                    || *total > 16383
                    || *current > 16383
                    || (!*reply && (*total != 1 || *current != 1 || !data.is_empty()))
                {
                    return Err(Diagnostic::new("E_PE_LENGTH", "invalid GET fields"));
                }
                out.push(*request);
                out.extend(packed_length(header.len(), 2)?);
                out.extend(header);
                out.extend(pack(u32::from(*total), 2));
                out.extend(pack(u32::from(*current), 2));
                out.extend(packed_length(data.len(), 2)?);
                out.extend(data);
            }
        }
        if out.iter().any(|v| *v > 127) {
            return Err(Diagnostic::new("E_CI_7BIT", "SysEx7 data exceeds 7 bits"));
        }
        Ok(out)
    }
    /// # Errors
    /// Returns a diagnostic for invalid fields, unsupported capabilities or failed protocol transactions.
    #[allow(
        clippy::too_many_lines,
        reason = "The finite protocol-family match keeps field order adjacent to each message definition."
    )]
    /// # Panics
    /// Panics only if validated wire-width invariants are inconsistent.
    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.iter().any(|v| *v > 127) {
            return Err(Diagnostic::new("E_CI_7BIT", "invalid byte"));
        }
        let mut r = Reader { data, position: 0 };
        if r.byte()? != 0x7e {
            return Err(Diagnostic::new(
                "E_CI_HEADER",
                "expected universal non-realtime",
            ));
        }
        let address = match r.byte()? {
            n @ 0..=15 => DeviceId::Channel(u4::new(n)),
            0x7e => DeviceId::Group,
            0x7f => DeviceId::FunctionBlock,
            _ => return Err(Diagnostic::new("E_CI_ADDRESS", "reserved destination")),
        };
        if r.byte()? != 0x0d {
            return Err(Diagnostic::new("E_CI_HEADER", "expected MIDI-CI"));
        }
        let status = r.byte()?;
        let version = r.byte()?;
        if ![1, 2].contains(&version) {
            return Err(Diagnostic::new("E_CI_VERSION", "unsupported version"));
        }
        let source = u28::new(r.num(4)?);
        let destination = u28::new(r.num(4)?);
        let body = match status {
            0x20 => Body::Inquiry,
            0x21 => Body::Reply {
                enabled: r.profiles()?,
                disabled: r.profiles()?,
            },
            0x22..=0x25 => {
                let profile = r.profile()?;
                let channels = if version == 2 {
                    u16::try_from(r.num(2)?).expect("14-bit field")
                } else {
                    u16::from(status != 0x23)
                };
                if status <= 0x23 {
                    Body::Set {
                        on: status == 0x22,
                        profile,
                        channels,
                    }
                } else {
                    Body::Report {
                        on: status == 0x24,
                        profile,
                        channels,
                    }
                }
            }
            0x26 => Body::Added {
                profile: r.profile()?,
            },
            0x27 => Body::Removed {
                profile: r.profile()?,
            },
            0x28 => Body::Details {
                profile: r.profile()?,
                target: r.byte()?,
            },
            0x29 => {
                let profile = r.profile()?;
                let target = r.byte()?;
                let size = r.num(2)?;
                Body::DetailsReply {
                    profile,
                    target,
                    data: r.take(size as usize)?.to_vec(),
                }
            }
            0x2f => {
                let profile = r.profile()?;
                let size = r.num(4)?;
                Body::Specific {
                    profile,
                    data: r.take(size as usize)?.to_vec(),
                }
            }
            0x7d | 0x7f => {
                if version == 1 {
                    Body::Ack {
                        negative: status == 0x7f,
                        original: 0,
                        status: 0,
                        status_data: 0,
                        details: [0; 5],
                        data: Vec::new(),
                    }
                } else {
                    let original = r.byte()?;
                    let code = r.byte()?;
                    let status_data = r.byte()?;
                    let details = r.profile()?;
                    let n = r.num(2)?;
                    Body::Ack {
                        negative: status == 0x7f,
                        original,
                        status: code,
                        status_data,
                        details,
                        data: r.take(n as usize)?.to_vec(),
                    }
                }
            }
            0x7e => Body::Invalidate { muid: r.num(4)? },
            0x30 | 0x31 => {
                let simultaneous = r.byte()?;
                let (major, minor) = if version == 2 {
                    (r.byte()?, r.byte()?)
                } else {
                    (0, 0)
                };
                Body::PeCapabilities {
                    reply: status == 0x31,
                    simultaneous,
                    major,
                    minor,
                }
            }
            0x34 | 0x35 => {
                let request = r.byte()?;
                let size = r.num(2)?;
                let header = r.take(size as usize)?.to_vec();
                let total = u16::try_from(r.num(2)?).expect("14-bit field");
                let current = u16::try_from(r.num(2)?).expect("14-bit field");
                let size = r.num(2)?;
                Body::Property {
                    reply: status == 0x35,
                    request,
                    header,
                    total,
                    current,
                    data: r.take(size as usize)?.to_vec(),
                }
            }
            _ => {
                return Err(Diagnostic::new(
                    "E_CI_MESSAGE",
                    format!("unsupported CI status {status:02x}"),
                ));
            }
        };
        if r.position != data.len() {
            return Err(Diagnostic::new("E_CI_LENGTH", "trailing bytes"));
        }
        let message = Self {
            address,
            version,
            source,
            destination,
            body,
        };
        message.encode()?;
        Ok(message)
    }
    /// # Errors
    /// Returns a diagnostic for invalid fields, unsupported capabilities or failed protocol transactions.
    pub fn ump(&self, group: u8) -> Result<Vec<u32>> {
        if group > 15 {
            return Err(Diagnostic::new("E_CI_ADDRESS", "invalid group"));
        }
        let mut m = midi2::sysex7::Sysex7::<Vec<u32>>::new();
        m.set_group(u4::new(group));
        m.set_payload(self.encode()?.into_iter().map(u7::new));
        Ok(m.data().to_vec())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Features {
    pub bits: u16,
    pub standard: [u8; 128],
    pub custom: [u8; 96],
}
impl Default for Features {
    fn default() -> Self {
        let mut standard = [0; 128];
        // Only the required default sound is assumed. The adapter supplies actual
        // optional sound counts; a vocabulary is not a receiver capability list.
        standard[0] = 1;
        Self {
            bits: 0,
            standard,
            custom: [0; 96],
        }
    }
}
impl Features {
    /// # Errors
    /// Returns a diagnostic for invalid fields, unsupported capabilities or failed protocol transactions.
    pub fn supports(&self, required: &Self) -> Result<()> {
        if self.bits & required.bits != required.bits
            || self
                .standard
                .iter()
                .zip(required.standard)
                .any(|(available, needed)| *available < needed)
            || self
                .custom
                .iter()
                .zip(required.custom)
                .any(|(available, needed)| *available < needed)
        {
            return Err(Diagnostic::new(
                "E_PROFILE_CAPABILITY",
                "receiver lacks required optional features or variations",
            ));
        }
        Ok(())
    }
    /// M2-123 §6.1; a substituted coordinate never proves exact playback.
    /// # Errors
    /// Returns a diagnostic for invalid fields, unsupported capabilities or failed protocol transactions.
    pub fn fallback(
        &self,
        requested: crate::rules::Articulation,
    ) -> Result<crate::rules::Articulation> {
        requested.validate()?;
        let count = |at: u8, sub: u8| {
            if at < 0x18 {
                self.standard[usize::from(at - 16) * 16 + usize::from(sub)]
            } else {
                self.custom[usize::from(at - 26) * 16 + usize::from(sub)]
            }
        };
        let mut result = requested;
        if count(result.at, result.subclass) > result.variation {
            return Ok(result);
        }
        result.variation = 0;
        if count(result.at, result.subclass) > 0 {
            return Ok(result);
        }
        result.subclass = 0;
        if count(result.at, 0) > 0 {
            return Ok(result);
        }
        result.at = 16;
        if count(16, 0) == 0 {
            return Err(Diagnostic::new(
                "E_PROFILE_BASELINE",
                "receiver has no default sound",
            ));
        }
        Ok(result)
    }
    /// # Errors
    /// Returns a diagnostic for invalid fields, unsupported capabilities or failed protocol transactions.
    pub fn payload(&self, target: u8) -> Result<Vec<u8>> {
        if self.bits & !31 != 0
            || self
                .standard
                .iter()
                .chain(self.custom.iter())
                .any(|v| *v > 16)
        {
            return Err(Diagnostic::new(
                "E_PROFILE_DETAILS",
                "reserved bits or count >16",
            ));
        }
        match target {
            1 => {
                let mut out = pack(u32::from(self.bits), 2);
                out.extend(self.standard);
                Ok(out)
            }
            0x40 => Ok(self.custom.to_vec()),
            _ => Err(Diagnostic::new("E_PROFILE_TARGET", "unsupported target")),
        }
    }
    /// # Errors
    /// Returns a diagnostic for invalid fields, unsupported capabilities or failed protocol transactions.
    /// # Panics
    /// Panics only if validated wire-width invariants are inconsistent.
    pub fn apply(&mut self, target: u8, data: &[u8]) -> Result<()> {
        let mut candidate = self.clone();
        match target {
            1 if data.len() == 130 => {
                candidate.bits = u16::try_from(number(&data[..2])).expect("two packed bytes");
                candidate.standard.copy_from_slice(&data[2..]);
            }
            0x40 if data.len() == 96 => candidate.custom.copy_from_slice(data),
            _ => return Err(Diagnostic::new("E_PROFILE_DETAILS", "wrong target length")),
        }
        candidate.payload(target)?;
        *self = candidate;
        Ok(())
    }
}

/// The local endpoint models one group. The caller owns transport and injected time.
type ChunkAssembly = (u16, u16, Vec<u8>, Vec<u8>);
pub struct Engine {
    group: u4,
    now_ms: u64,
    discovery_active: bool,
    discovery_seen: BTreeSet<u32>,
    pub muid: u28,
    pub enabled: BTreeSet<u8>,
    pub features: Features,
    profile: ProfileId,
    pub piano_details: BTreeMap<(u32, u8), Vec<u8>>,
    pub remote: BTreeMap<(u32, u8), bool>,
    pub pending: BTreeMap<(u32, u8), u64>,
    pub identity: midi2::ci::DiscoveryQuery<0x2, Vec<u8>>,
    peers: BTreeMap<u32, Vec<u8>>,
    pub extension_map: Option<crate::extension::Dictionary>,
    pe_ready: BTreeSet<u32>,
    pe_chunks: BTreeMap<(u32, u8), ChunkAssembly>,
    pe_pending: BTreeMap<(u32, u8), u64>,
    pub received_resources: BTreeMap<(u32, u8), serde_json::Value>,
    pub details: BTreeMap<(u32, u8), Features>,
    detail_targets: BTreeSet<(u32, u8, u8)>,
    details_pending: BTreeMap<(u32, u8, u8), u64>,
}
impl Engine {
    /// Create a standard Piano Profile sender service, sharing CI transactions.
    /// # Errors
    /// Rejects an invalid MUID.
    pub fn new_piano(muid: u32) -> Result<Self> {
        let mut engine = Self::new(muid)?;
        engine.profile = crate::piano::PROFILE_ID;
        Ok(engine)
    }
    /// # Errors
    /// Returns a diagnostic for invalid fields, unsupported capabilities or failed protocol transactions.
    pub fn new(muid: u32) -> Result<Self> {
        if muid >= BROADCAST {
            return Err(Diagnostic::new("E_CI_ADDRESS", "invalid local MUID"));
        }
        let mut identity = midi2::ci::DiscoveryQuery::<0x2, Vec<u8>>::new();
        identity.set_source(u28::new(muid));
        identity.set_profile_configuration_supported(true);
        identity.set_max_sysex_size(u28::new(1024));
        // Educational/non-commercial identity. A product supplies its own assigned ID.
        identity.set_device_manufacturer([u7::new(0x7d), u7::new(0), u7::new(0)]);
        Ok(Self {
            group: u4::new(0),
            now_ms: 0,
            discovery_active: false,
            discovery_seen: BTreeSet::new(),
            muid: u28::new(muid),
            enabled: BTreeSet::new(),
            features: Features::default(),
            profile: PROFILE_ID,
            piano_details: BTreeMap::new(),
            remote: BTreeMap::new(),
            pending: BTreeMap::new(),
            identity,
            peers: BTreeMap::new(),
            extension_map: None,
            pe_ready: BTreeSet::new(),
            pe_chunks: BTreeMap::new(),
            pe_pending: BTreeMap::new(),
            received_resources: BTreeMap::new(),
            details: BTreeMap::new(),
            detail_targets: BTreeSet::new(),
            details_pending: BTreeMap::new(),
        })
    }
    /// # Errors
    /// Returns a diagnostic for invalid fields, unsupported capabilities or failed protocol transactions.
    pub fn require_standard(
        &self,
        peer: u32,
        route: crate::model::Route,
        performance: &crate::model::Performance,
    ) -> Result<()> {
        if route.group != u8::from(self.group) {
            return Err(Diagnostic::new(
                "E_PROFILE_GROUP",
                "use the negotiation engine for the requested group",
            ));
        }
        if !self.peers.contains_key(&peer) {
            return Err(Diagnostic::new(
                "E_DISCOVERY_REQUIRED",
                "no current discovery identity",
            ));
        }
        if self.remote.get(&(peer, route.channel)) != Some(&true) {
            return Err(Diagnostic::new(
                "E_PROFILE_DISABLED",
                "profile enablement has not been confirmed",
            ));
        }
        let required = performance.required_features(Some(route))?;
        let needs_details = required.bits != 0
            || required
                .standard
                .iter()
                .enumerate()
                .any(|(i, count)| i != 0 && *count != 0)
            || required.standard[0] > 1;
        if needs_details && !self.detail_targets.contains(&(peer, route.channel, 1)) {
            return Err(Diagnostic::new(
                "E_PROFILE_DETAILS_REQUIRED",
                "optional standard support is unknown",
            ));
        }
        if required.custom.iter().any(|v| *v != 0)
            && !self.detail_targets.contains(&(peer, route.channel, 0x40))
        {
            return Err(Diagnostic::new(
                "E_PROFILE_DETAILS_REQUIRED",
                "custom sound support is unknown",
            ));
        }
        self.details
            .get(&(peer, route.channel))
            .unwrap_or(&Features::default())
            .supports(&required)
    }
    /// Construct an endpoint bound to one UMP group.
    /// # Errors
    /// Returns a diagnostic if the local MUID is reserved.
    pub fn new_in_group(muid: u32, group: u4) -> Result<Self> {
        let mut engine = Self::new(muid)?;
        engine.group = group;
        Ok(engine)
    }
    pub fn invalidate_program(&mut self, peer: u32, channel: u8) {
        self.details.remove(&(peer, channel));
        self.piano_details.remove(&(peer, channel));
        self.received_resources.retain(|(id, _), _| *id != peer);
        self.pe_pending.retain(|(id, _), _| *id != peer);
        self.pe_chunks.retain(|(id, _), _| *id != peer);
        self.detail_targets
            .retain(|(id, ch, _)| (*id, *ch) != (peer, channel));
        self.details_pending
            .retain(|(id, ch, _), _| (*id, *ch) != (peer, channel));
    }
    /// # Errors
    /// Returns a diagnostic for invalid fields, unsupported capabilities or failed protocol transactions.
    pub fn set_extension_map(&mut self, map: crate::extension::Dictionary) -> Result<()> {
        map.validate()?;
        self.extension_map = Some(map);
        self.identity.set_property_exchange_supported(true);
        Ok(())
    }
    #[must_use]
    pub fn discovery(&self) -> Vec<u8> {
        self.identity.data()[1..self.identity.data().len() - 1].to_vec()
    }
    /// Start a broadcast discovery transaction; repeated responders invalidate
    /// the duplicated MUID under M2-101 §5.9.2.
    pub fn begin_discovery(&mut self) -> Vec<u8> {
        self.discovery_active = true;
        self.discovery_seen.clear();
        self.discovery()
    }
    pub fn finish_discovery(&mut self) {
        self.discovery_active = false;
        self.discovery_seen.clear();
    }
    fn forget_peer(&mut self, muid: u32) {
        self.remote.retain(|(id, _), _| *id != muid);
        self.pending.retain(|(id, _), _| *id != muid);
        self.peers.remove(&muid);
        self.piano_details.retain(|(peer, _), _| *peer != muid);
        self.details.retain(|(id, _), _| *id != muid);
        self.detail_targets.retain(|(id, _, _)| *id != muid);
        self.details_pending.retain(|(id, _, _), _| *id != muid);
        self.pe_ready.remove(&muid);
        self.pe_chunks.retain(|(id, _), _| *id != muid);
        self.pe_pending.retain(|(id, _), _| *id != muid);
        self.received_resources.retain(|(id, _), _| *id != muid);
        self.discovery_seen.remove(&muid);
    }
    fn renew_muid<R: rand::Rng + ?Sized>(&mut self, rng: &mut R) {
        let old = u32::from(self.muid);
        let mut next = rng.random_range(0..BROADCAST);
        while next == old {
            next = rng.random_range(0..BROADCAST);
        }
        self.muid = u28::new(next);
        self.identity.set_source(self.muid);
        self.remote.clear();
        self.pending.clear();
        self.peers.clear();
        self.piano_details.clear();
        self.details.clear();
        self.detail_targets.clear();
        self.details_pending.clear();
        self.pe_ready.clear();
        self.pe_chunks.clear();
        self.pe_pending.clear();
        self.received_resources.clear();
        self.discovery_seen.clear();
    }
    /// Process a message at a caller-supplied monotonic millisecond timestamp.
    /// # Errors
    /// Returns a diagnostic if time moves backwards or the message is invalid.
    pub fn receive_at(&mut self, message: &Message, now_ms: u64) -> Result<Vec<Message>> {
        if now_ms < self.now_ms {
            return Err(Diagnostic::new(
                "E_CI_TIME",
                "monotonic clock moved backwards",
            ));
        }
        self.now_ms = now_ms;
        self.receive(message)
    }
    /// Process deframed SysEx7 at a caller-supplied monotonic timestamp.
    /// # Errors
    /// Returns a diagnostic if time moves backwards or the message is invalid.
    pub fn receive_payload_at(&mut self, payload: &[u8], now_ms: u64) -> Result<Vec<Vec<u8>>> {
        if now_ms < self.now_ms {
            return Err(Diagnostic::new(
                "E_CI_TIME",
                "monotonic clock moved backwards",
            ));
        }
        self.now_ms = now_ms;
        self.receive_payload(payload)
    }
    /// Accept deframed SysEx7. Published discovery types validate discovery bodies;
    /// local profile wrappers fill only upstream's absent transaction families.
    /// # Errors
    /// Returns a diagnostic for invalid fields, unsupported capabilities or failed protocol transactions.
    pub fn receive_payload(&mut self, payload: &[u8]) -> Result<Vec<Vec<u8>>> {
        self.receive_payload_with_rng(payload, &mut rand::rng())
    }
    /// Deterministic discovery/profile processing with caller-supplied entropy.
    /// # Errors
    /// Returns a diagnostic for invalid messages or failed transactions.
    pub fn receive_payload_with_rng<R: rand::Rng + ?Sized>(
        &mut self,
        payload: &[u8],
        rng: &mut R,
    ) -> Result<Vec<Vec<u8>>> {
        match self.receive_payload_checked(payload, rng) {
            Ok(messages) => Ok(messages),
            Err(error) => {
                // M2-101 §5.11: reply only when an intact header gives a valid return address.
                if payload.len() < 13
                    || payload.iter().any(|b| *b > 127)
                    || payload[0] != 0x7e
                    || payload[2] != 0x0d
                    || [0x7c, 0x7d].contains(&payload[3])
                {
                    return Err(error);
                }
                let source = number(&payload[5..9]);
                let dest = number(&payload[9..13]);
                if source == BROADCAST || dest != u32::from(self.muid) {
                    return Err(error);
                }
                let address = match payload[1] {
                    0..=15 => DeviceId::Channel(u4::new(payload[1])),
                    0x7e => DeviceId::Group,
                    0x7f => DeviceId::FunctionBlock,
                    _ => return Err(error),
                };
                if error.code == "E_PE_ABORTED" {
                    return Err(error);
                }
                let status = match error.code {
                    "E_PE_CHUNK" => 0x21,
                    "E_CI_VERSION" => 2,
                    "E_CI_MESSAGE" => 1,
                    _ => 0x41,
                };
                let reply = Message {
                    address,
                    version: 2,
                    source: self.muid,
                    destination: u28::new(source),
                    body: Body::Ack {
                        negative: true,
                        original: payload[3],
                        status,
                        status_data: 0,
                        details: [0; 5],
                        data: Vec::new(),
                    },
                };
                Ok(vec![reply.encode()?])
            }
        }
    }
    #[allow(
        clippy::too_many_lines,
        reason = "Keep the discovery and universal envelope dispatch together."
    )]
    fn receive_payload_checked<R: rand::Rng + ?Sized>(
        &mut self,
        payload: &[u8],
        rng: &mut R,
    ) -> Result<Vec<Vec<u8>>> {
        if payload.len() >= 13
            && payload[0] == 0x7e
            && payload[2] == 0x0d
            && [0x70, 0x71].contains(&payload[3])
        {
            let version = payload[4];
            let reply = payload[3] == 0x71;
            let destination = number(&payload[9..13]);
            if destination != BROADCAST && destination != u32::from(self.muid) {
                return Ok(vec![]);
            }
            let mut query = payload.to_vec();
            query[3] = 0x70;
            query[9..13].fill(127);
            if reply && version == 2 {
                query
                    .pop()
                    .ok_or_else(|| Diagnostic::new("E_DISCOVERY", "missing function block"))?;
            }
            let mut framed = vec![0xf0];
            framed.extend(&query);
            framed.push(0xf7);
            let source = match version {
                1 => u32::from(
                    midi2::ci::DiscoveryQuery::<1, _>::try_from(framed.as_slice())
                        .map_err(|e| Diagnostic::new("E_DISCOVERY", format!("{e:?}")))?
                        .source(),
                ),
                2 => u32::from(
                    midi2::ci::DiscoveryQuery::<2, _>::try_from(framed.as_slice())
                        .map_err(|e| Diagnostic::new("E_DISCOVERY", format!("{e:?}")))?
                        .source(),
                ),
                _ => {
                    return Err(Diagnostic::new(
                        "E_CI_VERSION",
                        "unsupported discovery version",
                    ));
                }
            };
            if source == BROADCAST {
                return Err(Diagnostic::new(
                    "E_CI_ADDRESS",
                    "broadcast cannot be a discovery source",
                ));
            }
            if source == u32::from(self.muid) {
                let old = u32::from(self.muid);
                let invalidation = Message {
                    address: DeviceId::FunctionBlock,
                    version: 2,
                    source: self.muid,
                    destination: u28::new(BROADCAST),
                    body: Body::Invalidate { muid: old },
                }
                .encode()?;
                self.renew_muid(rng);
                return Ok(vec![invalidation, self.discovery()]);
            }
            let size = number(&query[25..29]);
            if query[24] & 0x61 != 0 || size < 128 || (query[24] & 0x0c != 0 && size < 512) {
                return Err(Diagnostic::new(
                    "E_DISCOVERY",
                    "reserved categories or insufficient receive size",
                ));
            }
            let mut identity = query[13..29].to_vec();
            identity.push(version);
            if reply && self.discovery_active && !self.discovery_seen.insert(source) {
                let invalidation = Message {
                    address: DeviceId::FunctionBlock,
                    version: 2,
                    source: self.muid,
                    destination: u28::new(BROADCAST),
                    body: Body::Invalidate { muid: source },
                }
                .encode()?;
                self.forget_peer(source);
                return Ok(vec![invalidation, self.begin_discovery()]);
            }
            if self.peers.get(&source).is_some_and(|old| *old != identity) {
                self.forget_peer(source);
            }
            self.peers.insert(source, identity);
            if reply {
                return Ok(vec![]);
            }
            let mut response = self.discovery();
            response[3] = 0x71;
            response[4] = version;
            response[9..13].copy_from_slice(&pack(source, 4));
            if version == 2 {
                response[29] = payload[29];
                response.push(0x7f);
            } else {
                response.pop();
            }
            return Ok(vec![response]);
        }
        let message = Message::decode(payload)?;
        if matches!(message.body,Body::Invalidate{muid} if muid==u32::from(self.muid)) {
            self.renew_muid(rng);
            return Ok(vec![self.discovery()]);
        }
        self.receive(&message)
            .and_then(|messages| messages.iter().map(Message::encode).collect())
    }
    /// # Errors
    /// Returns a diagnostic for invalid fields, unsupported capabilities or failed protocol transactions.
    pub fn request_enable(&mut self, peer: u32, channel: u8, deadline: u64) -> Result<Message> {
        if self
            .peers
            .get(&peer)
            .is_some_and(|identity| identity[11] & 4 == 0)
        {
            return Err(Diagnostic::new(
                "E_PROFILE_UNSUPPORTED",
                "peer does not advertise Profile Configuration",
            ));
        }
        let mut m = Message::channel(
            channel,
            self.muid.into(),
            peer,
            Body::Set {
                on: true,
                profile: self.profile,
                channels: 1,
            },
        )?;
        m.version = self
            .peers
            .get(&peer)
            .and_then(|p| p.last())
            .copied()
            .unwrap_or(2);
        self.pending.insert((peer, channel), deadline);
        Ok(m)
    }
    /// # Errors
    /// Returns a diagnostic for invalid fields, unsupported capabilities or failed protocol transactions.
    pub fn request_property(
        &mut self,
        peer: u32,
        request: u8,
        resource: &str,
        res_id: Option<&str>,
        deadline: u64,
    ) -> Result<Message> {
        if !self.pe_ready.contains(&peer)
            || request > 127
            || self.pe_pending.keys().any(|(id, _)| *id == peer)
        {
            return Err(Diagnostic::new(
                "E_PE_TRANSACTION",
                "negotiate capabilities and finish the previous request first",
            ));
        }
        let mut header = serde_json::json!({"resource":resource});
        if let Some(id) = res_id {
            header["resId"] = id.into();
        }
        let message = Message {
            address: DeviceId::FunctionBlock,
            version: self
                .peers
                .get(&peer)
                .and_then(|p| p.last())
                .copied()
                .unwrap_or(2),
            source: self.muid,
            destination: u28::new(peer),
            body: Body::Property {
                reply: false,
                request,
                header: crate::extension::ascii_json(&header),
                total: 1,
                current: 1,
                data: vec![],
            },
        };
        message.encode()?;
        self.pe_pending.insert((peer, request), deadline);
        self.received_resources.remove(&(peer, request));
        Ok(message)
    }
    /// Request optional capabilities for the current receiver/program identity.
    /// # Errors
    /// Returns a diagnostic for unsupported targets, versions or addresses.
    pub fn request_details(
        &mut self,
        peer: u32,
        channel: u8,
        target: u8,
        deadline: u64,
    ) -> Result<Message> {
        if self.peers.get(&peer).and_then(|p| p.last()) == Some(&1)
            || !matches!(target, 1 | 0x40)
            || self.profile == crate::piano::PROFILE_ID && target != 0x40
        {
            return Err(Diagnostic::new(
                "E_PROFILE_TARGET",
                "details require CI version 2 and a defined orchestral target",
            ));
        }
        let message = Message::channel(
            channel,
            self.muid.into(),
            peer,
            Body::Details {
                profile: self.profile,
                target,
            },
        )?;
        self.details_pending
            .insert((peer, channel, target), deadline);
        Ok(message)
    }
    pub fn expire(&mut self, now: u64) -> Vec<(u32, u8)> {
        let now = now.max(self.now_ms);
        self.now_ms = now;
        self.details_pending.retain(|_, deadline| *deadline > now);
        self.pe_pending.retain(|key, deadline| {
            if *deadline <= now {
                self.pe_chunks.remove(key);
                false
            } else {
                true
            }
        });
        let expired = self
            .pending
            .iter()
            .filter_map(|(k, t)| (*t <= now).then_some(*k))
            .collect::<Vec<_>>();
        for k in &expired {
            self.pending.remove(k);
            self.remote.remove(k);
        }
        expired
    }
    /// # Panics
    /// Panics if a bundled schema/catalog or an internally validated invariant is inconsistent.
    /// # Errors
    /// Returns a diagnostic for invalid fields, unsupported capabilities or failed protocol transactions.
    #[allow(
        clippy::too_many_lines,
        reason = "The finite protocol-family match keeps field order adjacent to each message definition."
    )]
    pub fn receive(&mut self, m: &Message) -> Result<Vec<Message>> {
        m.encode()?;
        let peer: u32 = m.source.into();
        let dest: u32 = m.destination.into();
        if dest != BROADCAST && dest != u32::from(self.muid) {
            return Ok(Vec::new());
        }
        if let Body::Invalidate { muid } = m.body {
            if muid == u32::from(self.muid) {
                return Err(Diagnostic::new(
                    "E_CI_IDENTITY",
                    "use receive_payload for local MUID renewal and discovery",
                ));
            }
            self.forget_peer(muid);
            return Ok(vec![]);
        }
        let channel = match m.address {
            DeviceId::Channel(c) => u8::from(c),
            _ => 0x7f,
        };
        let response = |body: Body, broadcast: bool| Message {
            address: m.address,
            version: m.version,
            source: self.muid,
            destination: u28::new(if broadcast { BROADCAST } else { peer }),
            body,
        };
        let profile_id = self.profile;
        let details = match &m.body {
            Body::Set { profile, .. }
            | Body::Details { profile, .. }
            | Body::Specific { profile, .. } => *profile,
            _ => [0; 5],
        };
        let nak = |status| {
            response(
                Body::Ack {
                    negative: true,
                    original: m.status(),
                    status,
                    status_data: 0,
                    details,
                    data: Vec::new(),
                },
                false,
            )
        };
        match &m.body {
            Body::Inquiry => {
                let channels = if channel < 16 {
                    channel..=channel
                } else {
                    0..=15
                };
                Ok(channels
                    .map(|ch| {
                        let (enabled, disabled) = if self.enabled.contains(&ch) {
                            (vec![profile_id], vec![])
                        } else {
                            (vec![], vec![profile_id])
                        };
                        Message {
                            address: DeviceId::Channel(u4::new(ch)),
                            ..response(Body::Reply { enabled, disabled }, false)
                        }
                    })
                    .collect())
            }
            Body::Set {
                on,
                profile,
                channels,
            } => {
                if channel > 15 || *profile != profile_id || *channels > 1 {
                    return Ok(vec![nak(4)]);
                }
                if *on {
                    self.enabled.insert(channel);
                } else {
                    self.enabled.remove(&channel);
                }
                Ok(vec![response(
                    Body::Report {
                        on: *on,
                        profile: *profile,
                        channels: 1,
                    },
                    true,
                )])
            }
            Body::Details { profile, target } => {
                if *profile != profile_id || channel > 15 {
                    return Ok(vec![nak(4)]);
                }
                let payload = if profile_id == crate::piano::PROFILE_ID {
                    if *target == 0x40 {
                        Ok(crate::piano::sender_details())
                    } else {
                        Err(Diagnostic::new(
                            "E_PROFILE_TARGET",
                            "unsupported Piano target",
                        ))
                    }
                } else {
                    self.features.payload(*target)
                };
                match payload {
                    Ok(data) => Ok(vec![response(
                        Body::DetailsReply {
                            profile: *profile,
                            target: *target,
                            data,
                        },
                        false,
                    )]),
                    Err(_) => Ok(vec![nak(1)]),
                }
            }
            Body::DetailsReply {
                profile,
                target,
                data,
            } if *profile == profile_id && channel < 16 => {
                if !self.details_pending.contains_key(&(peer, channel, *target)) {
                    return Err(Diagnostic::new(
                        "E_CI_TRANSACTION",
                        "unsolicited Profile Details reply",
                    ));
                }
                if profile_id == crate::piano::PROFILE_ID {
                    crate::piano::validate_details(data)?;
                    self.piano_details.insert((peer, channel), data.clone());
                    self.details_pending.remove(&(peer, channel, *target));
                    self.detail_targets.insert((peer, channel, *target));
                    return Ok(vec![]);
                }
                self.details
                    .entry((peer, channel))
                    .or_default()
                    .apply(*target, data)?;
                self.details_pending.remove(&(peer, channel, *target));
                self.detail_targets.insert((peer, channel, *target));
                Ok(vec![])
            }
            Body::Report {
                on,
                profile,
                channels,
            } if *profile == profile_id => {
                if channel > 15 || *channels != 1 {
                    return Err(Diagnostic::new(
                        "E_PROFILE_CHANNELS",
                        "orchestral reports describe one MIDI channel",
                    ));
                }
                self.remote.insert((peer, channel), *on);
                if !*on {
                    self.invalidate_program(peer, channel);
                }
                self.pending.remove(&(peer, channel));
                Ok(vec![])
            }
            Body::Reply { enabled, disabled } => {
                if enabled.contains(&profile_id) {
                    self.remote.insert((peer, channel), true);
                } else if disabled.contains(&profile_id) {
                    self.remote.insert((peer, channel), false);
                }
                Ok(vec![])
            }
            Body::Added { profile } if *profile == profile_id => {
                self.invalidate_program(peer, channel);
                self.remote.entry((peer, channel)).or_insert(false);
                Ok(vec![])
            }
            Body::Removed { profile } if *profile == profile_id => {
                self.invalidate_program(peer, channel);
                self.remote.remove(&(peer, channel));
                self.pending.remove(&(peer, channel));
                Ok(vec![])
            }
            Body::Ack {
                negative: true,
                original,
                details,
                ..
            } if self.pending.contains_key(&(peer, channel))
                && ((*original == 0 && m.version == 1) || matches!(*original, 0x22 | 0x23))
                && (*details == profile_id || *details == [0; 5]) =>
            {
                self.pending.remove(&(peer, channel));
                self.remote.remove(&(peer, channel));
                Ok(vec![])
            }
            Body::Ack {
                negative: false,
                status: 0x10,
                status_data,
                original,
                details,
                ..
            } => {
                let until = self
                    .now_ms
                    .checked_add(u64::from(*status_data) * 100)
                    .ok_or_else(|| Diagnostic::new("E_CI_TIME", "timeout overflow"))?;
                if matches!(*original, 0x22 | 0x23) && *details == profile_id {
                    if let Some(deadline) = self.pending.get_mut(&(peer, channel)) {
                        *deadline = (*deadline).max(until);
                    }
                }
                if *original == 0x28 && *details == profile_id {
                    for ((id, ch, _), deadline) in &mut self.details_pending {
                        if *id == peer && *ch == channel {
                            *deadline = (*deadline).max(until);
                        }
                    }
                }
                if *original == 0x34 {
                    if let Some(deadline) = self.pe_pending.get_mut(&(peer, details[0])) {
                        *deadline = (*deadline).max(until);
                    }
                }
                Ok(vec![])
            }
            Body::Specific { profile, .. } => {
                Ok(vec![nak(if *profile == profile_id { 1 } else { 4 })])
            }
            Body::PeCapabilities {
                reply,
                simultaneous,
                major,
                minor,
            } => {
                if m.address != DeviceId::FunctionBlock
                    || *simultaneous == 0
                    || *major != 0
                    || *minor != 0
                {
                    return Ok(vec![nak(1)]);
                }
                self.pe_ready.insert(peer);
                if *reply {
                    Ok(vec![])
                } else if self.extension_map.is_some() {
                    Ok(vec![response(
                        Body::PeCapabilities {
                            reply: true,
                            simultaneous: 1,
                            major: 0,
                            minor: 0,
                        },
                        false,
                    )])
                } else {
                    Ok(vec![nak(1)])
                }
            }
            Body::Property {
                reply,
                request,
                header,
                total,
                current,
                data,
            } => {
                if m.address != DeviceId::FunctionBlock || !self.pe_ready.contains(&peer) {
                    return Ok(vec![nak(1)]);
                }
                if *reply {
                    if !self.pe_pending.contains_key(&(peer, *request)) {
                        return Err(Diagnostic::new(
                            "E_PE_TRANSACTION",
                            "unsolicited property reply",
                        ));
                    }
                    if *current == 0 {
                        self.pe_chunks.remove(&(peer, *request));
                        self.pe_pending.remove(&(peer, *request));
                        return Err(Diagnostic::new(
                            "E_PE_ABORTED",
                            "sender aborted the property transfer",
                        ));
                    }
                    if *total != 0 && *current > *total {
                        return Err(Diagnostic::new("E_PE_CHUNK", "chunk exceeds total"));
                    }
                    let entry = self.pe_chunks.entry((peer, *request)).or_insert((
                        1,
                        *total,
                        Vec::new(),
                        Vec::new(),
                    ));
                    // M2-101 §8.3 permits the final chunk to declare its actual total,
                    // including unknown-length and early-completed streams.
                    if (entry.1 != *total && *total != *current)
                        || entry.0 != *current
                        || (*current > 1 && !header.is_empty())
                    {
                        return Err(Diagnostic::new(
                            "E_PE_CHUNK",
                            "out-of-order chunk or repeated header",
                        ));
                    }
                    if *current == 1 {
                        entry.2.clone_from(header);
                    }
                    entry.3.extend(data);
                    entry.0 += 1;
                    if entry.3.len() > 16 * 1024 * 1024 {
                        return Err(Diagnostic::new("E_PE_LENGTH", "resource exceeds 16 MiB"));
                    }
                    if *current == *total {
                        let (_, _, header, body) = self
                            .pe_chunks
                            .remove(&(peer, *request))
                            .expect("active request");
                        self.pe_pending.remove(&(peer, *request));
                        let h: serde_json::Value = serde_json::from_slice(&header)
                            .map_err(|e| Diagnostic::new("E_PE_JSON", e.to_string()))?;
                        if h["status"] != 200 {
                            return Err(Diagnostic::new("E_PE_STATUS", h.to_string()));
                        }
                        let value = serde_json::from_slice(&body)
                            .map_err(|e| Diagnostic::new("E_PE_JSON", e.to_string()))?;
                        self.received_resources.insert((peer, *request), value);
                    }
                    return Ok(vec![]);
                }
                let h: serde_json::Value = serde_json::from_slice(header)
                    .map_err(|e| Diagnostic::new("E_PE_JSON", e.to_string()))?;
                if h.get("mutualEncoding")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|v| v != "ASCII")
                {
                    return Ok(vec![response(
                        Body::Property {
                            reply: true,
                            request: *request,
                            header: b"{\"status\":415}".to_vec(),
                            total: 1,
                            current: 1,
                            data: vec![],
                        },
                        false,
                    )]);
                }
                let resource = h
                    .get("resource")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                let value = match resource {
                    "ResourceList" => Some(
                        serde_json::json!([{"resource":"X-ArticulationMap","canGet":true,"canSet":"none","schema":{"$ref":"midi+jsonschema://articulationDictionaryV1"}},{"resource":"JSONSchema","canGet":true,"canSet":"none"}]),
                    ),
                    "X-ArticulationMap" => self
                        .extension_map
                        .as_ref()
                        .map(crate::extension::Dictionary::property_data),
                    "JSONSchema"
                        if h.get("resId").and_then(serde_json::Value::as_str)
                            == Some("articulationDictionaryV1") =>
                    {
                        Some(
                            serde_json::from_str(include_str!("../mapping/refinement.schema.json"))
                                .expect("checked schema"),
                        )
                    }
                    _ => None,
                };
                let Some(value) = value else {
                    return Ok(vec![response(
                        Body::Property {
                            reply: true,
                            request: *request,
                            header: b"{\"status\":404}".to_vec(),
                            total: 1,
                            current: 1,
                            data: vec![],
                        },
                        false,
                    )]);
                };
                let body = crate::extension::ascii_json(&value);
                let limit = self
                    .peers
                    .get(&peer)
                    .map_or(128, |identity| number(&identity[12..16]) as usize)
                    .clamp(128, 1024)
                    - 64;
                let total = body.len().div_ceil(limit);
                if total > 16383 {
                    return Err(Diagnostic::new("E_PE_LENGTH", "too many response chunks"));
                }
                Ok(body
                    .chunks(limit)
                    .enumerate()
                    .map(|(i, chunk)| {
                        response(
                            Body::Property {
                                reply: true,
                                request: *request,
                                header: if i == 0 {
                                    b"{\"status\":200}".to_vec()
                                } else {
                                    vec![]
                                },
                                total: u16::try_from(total).expect("validated chunk count"),
                                current: u16::try_from(i + 1).expect("validated chunk index"),
                                data: chunk.to_vec(),
                            },
                            false,
                        )
                    })
                    .collect())
            }
            _ => Ok(vec![]),
        }
    }
}
