//! Global OSC codec for TotalMix FX 2.1+.
//!
//! This is the absolute-channel sibling of [`crate::osc_rme`].  Global OSC
//! addresses each input, playback and output by its absolute 0-based
//! position, so it needs no bank or page state.  Values are native units
//! (dB, Hz, list indexes), not the normalized values of the classic
//! surface.
//!
//! The module is a pure codec: it decodes one message into
//! [`GlobalCommand`], and encodes a command (or feedback value) back into an
//! OSC packet.  The UDP worker in [`crate::osc`] owns the socket and the
//! app decides which commands are supported by the current device.

use rosc::{OscMessage, OscPacket, OscType};

use crate::osc_rme::CHANNELS;

/// The three absolute channel rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bus {
    Input,
    Playback,
    Output,
}

impl Bus {
    /// OSC path segment for the strip namespace.
    pub fn path(self) -> &'static str {
        match self {
            Bus::Input => "input",
            Bus::Playback => "playback",
            Bus::Output => "output",
        }
    }

    /// OSC path segment for the matrix namespace, if this row has one.
    pub fn mix_path(self) -> Option<&'static str> {
        match self {
            Bus::Input => Some("in"),
            Bus::Playback => Some("pb"),
            Bus::Output => None,
        }
    }
}

/// A decoded Global OSC control or feedback value.
#[derive(Debug, Clone, PartialEq)]
pub enum GlobalCommand {
    Channel {
        bus: Bus,
        channel: usize,
        control: ChannelControl,
    },
    Mix {
        source: Bus,
        channel: usize,
        submix: usize,
        control: MixControl,
    },
    ControlRoom(ControlRoomControl),
    Reverb(FxControl),
    Echo(FxControl),
    MuteGroup {
        group: usize,
        on: bool,
    },
    SoloGroup {
        group: usize,
        on: bool,
    },
    FaderGroup {
        group: usize,
        on: bool,
    },
    GlobalMute(bool),
    GlobalSolo(bool),
    Undo,
    Redo,
    SnapshotLoad {
        number: usize,
        state: f32,
    },
    SnapshotSave,
    LayoutLoad(usize),
    Durec(DurecControl),
    Send(SendControl),
    ShowWindow(bool),
    Level {
        bus: Bus,
        channel: usize,
        level_db: f32,
    },
    Status(StatusControl),
}

/// A channel-strip control.  The same set is used for inputs, playbacks and
/// outputs; the app decides which fields the current device exposes.
#[derive(Debug, Clone, PartialEq)]
pub enum ChannelControl {
    Mute(bool),
    Stereo(bool),
    Record(bool),
    Name(String),
    PlayChannel(usize),
    MsProc(bool),
    Phase(bool),
    Width(f32),
    Pfl(bool),
    Gain(f32),
    RefLevel(usize),
    Phantom(bool),
    Instrument(bool),
    Pad(bool),
    Autoset(bool),
    Volume(f32),
    VolumeLin(f32),
    Balance(f32),
    FxSend(f32),
    FxReturn(f32),
    Crossfeed(f32),
    Loopback(bool),
    TalkbackSel(bool),
    Delay(f32),
    Color(usize),
    LowCut(LowCutControl),
    Eq(EqControl),
    Dynamics(DynamicsControl),
    AutoLevel(AutoLevelControl),
    RoomEq(EqControl),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MixControl {
    Fader(f32),
    FaderLin(f32),
    Balance(f32),
    GroupFlags(u32),
    Solo(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LowCutControl {
    Enable(bool),
    Freq(f32),
    Slope(f32),
    LoadPreset,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EqControl {
    Enable(bool),
    BandGain { band: usize, db: f32 },
    BandFreq { band: usize, hz: f32 },
    BandQ { band: usize, q: f32 },
    BandType { band: usize, kind: usize },
    LoadPreset,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DynamicsControl {
    Enable(bool),
    Gain(f32),
    Attack(f32),
    Release(f32),
    CompThreshold(f32),
    CompRatio(f32),
    ExpThreshold(f32),
    ExpRatio(f32),
    LoadPreset,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AutoLevelControl {
    Enable(bool),
    MaxGain(f32),
    Headroom(f32),
    RiseTime(f32),
    LoadPreset,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FxControl {
    Enable(bool),
    Type(usize),
    Predelay(f32),
    LowCut(f32),
    RoomScale(f32),
    Attack(f32),
    Hold(f32),
    Release(f32),
    HighCut(f32),
    Time(f32),
    HighDamp(f32),
    Smooth(f32),
    Volume(f32),
    Width(f32),
    Delay(f32),
    Feedback(f32),
    LoadPreset,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlRoomControl {
    MainOut(usize),
    MainOutB(usize),
    Phones { slot: usize, channel: usize },
    MainMono(bool),
    DimReduction(f32),
    Dim(bool),
    RecallVolume(f32),
    Recall,
    TalkChannel(usize),
    Talkback(bool),
    CueChannel(Option<usize>),
    ExternalIn(bool),
    ExtInChannel(usize),
    ExtInGain(f32),
    LinkAB(bool),
    SpeakerB(bool),
    MuteFx(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DurecControl {
    Play,
    Pause,
    Stop,
    Record,
    Next,
    Previous,
    Time(String),
    State(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SendControl {
    SendAll(u8),
    SendSettings,
    SendChannel { bus: Bus, channel: usize },
    SendMix(u8),
    SendSubmix { submix: usize, mode: u8 },
    SendState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatusControl {
    Device(String),
    Connection(bool),
    Dsp(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    EmptyAddress,
    UnknownPath,
    MissingValue,
    WrongValueType,
    InvalidIndex,
    Ignored,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            DecodeError::EmptyAddress => "empty address",
            DecodeError::UnknownPath => "unknown path",
            DecodeError::MissingValue => "missing value",
            DecodeError::WrongValueType => "wrong value type",
            DecodeError::InvalidIndex => "invalid index",
            DecodeError::Ignored => "trigger below 0.5",
        })
    }
}

/// Parse one Global OSC message.  `Ignored` means a `(f)` trigger carried a
/// value below 0.5, which the reference says to ignore.
pub fn parse(addr: &str, args: &[OscType]) -> Result<GlobalCommand, DecodeError> {
    let parts: Vec<&str> = addr.split('/').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return Err(DecodeError::EmptyAddress);
    }

    match parts.as_slice() {
        ["mix", bus, channel, submix, field] => {
            let source = match *bus {
                "in" => Bus::Input,
                "pb" => Bus::Playback,
                _ => return Err(DecodeError::UnknownPath),
            };
            let channel = index(channel)?;
            let submix = index(submix)?;
            if channel >= CHANNELS || submix >= CHANNELS {
                return Err(DecodeError::InvalidIndex);
            }
            Ok(GlobalCommand::Mix {
                source,
                channel,
                submix,
                control: parse_mix(field, args)?,
            })
        }
        ["input", channel, rest @ ..] => parse_channel(Bus::Input, channel, rest, args),
        ["playback", channel, rest @ ..] => parse_channel(Bus::Playback, channel, rest, args),
        ["output", channel, rest @ ..] => parse_channel(Bus::Output, channel, rest, args),
        ["controlroom", field] => Ok(GlobalCommand::ControlRoom(parse_control_room(field, args)?)),
        ["reverb", field] => Ok(GlobalCommand::Reverb(parse_fx(field, args, false)?)),
        ["echo", field] => Ok(GlobalCommand::Echo(parse_fx(field, args, true)?)),
        ["mutegroup", group] => parse_group(group, args, GroupKind::Mute),
        ["sologroup", group] => parse_group(group, args, GroupKind::Solo),
        ["fadergroup", group] => parse_group(group, args, GroupKind::Fader),
        ["globalmute"] => Ok(GlobalCommand::GlobalMute(bool_arg(args)?)),
        ["globalsolo"] => Ok(GlobalCommand::GlobalSolo(bool_arg(args)?)),
        ["undo"] => trigger(args).map(|_| GlobalCommand::Undo),
        ["redo"] => trigger(args).map(|_| GlobalCommand::Redo),
        ["snapshot", "load", number] => {
            let number = positive_index(number)?;
            let state = optional_float(args).unwrap_or(1.0);
            Ok(GlobalCommand::SnapshotLoad { number, state })
        }
        ["snapshot", "save"] => trigger(args).map(|_| GlobalCommand::SnapshotSave),
        ["layout", "load", number] => Ok(GlobalCommand::LayoutLoad(positive_index(number)?)),
        ["durec", field] => Ok(GlobalCommand::Durec(parse_durec(field, args)?)),
        ["sendall"] => Ok(GlobalCommand::Send(SendControl::SendAll(send_mode(args)))),
        ["sendsettings"] => trigger(args).map(|_| GlobalCommand::Send(SendControl::SendSettings)),
        ["sendchan", bus, channel] => {
            let bus = bus_name(bus)?;
            let channel = index(channel)?;
            if channel >= CHANNELS {
                return Err(DecodeError::InvalidIndex);
            }
            Ok(GlobalCommand::Send(SendControl::SendChannel {
                bus,
                channel,
            }))
        }
        ["sendmix"] => Ok(GlobalCommand::Send(SendControl::SendMix(send_mode(args)))),
        ["sendsubmix", submix] => {
            let submix = index(submix)?;
            if submix >= CHANNELS {
                return Err(DecodeError::InvalidIndex);
            }
            Ok(GlobalCommand::Send(SendControl::SendSubmix {
                submix,
                mode: send_mode(args),
            }))
        }
        ["sendstate"] => trigger(args).map(|_| GlobalCommand::Send(SendControl::SendState)),
        ["showwindow"] => Ok(GlobalCommand::ShowWindow(bool_arg(args)?)),
        ["level", bus, channel] => {
            let bus = bus_name(bus)?;
            let channel = index(channel)?;
            if channel >= CHANNELS {
                return Err(DecodeError::InvalidIndex);
            }
            Ok(GlobalCommand::Level {
                bus,
                channel,
                level_db: float_arg(args)?,
            })
        }
        ["status", field] => Ok(GlobalCommand::Status(parse_status(field, args)?)),
        _ => Err(DecodeError::UnknownPath),
    }
}

enum GroupKind {
    Mute,
    Solo,
    Fader,
}

fn parse_group(
    group: &str,
    args: &[OscType],
    kind: GroupKind,
) -> Result<GlobalCommand, DecodeError> {
    let group = positive_index(group)?;
    if !(1..=4).contains(&group) {
        return Err(DecodeError::InvalidIndex);
    }
    let on = bool_arg(args)?;
    Ok(match kind {
        GroupKind::Mute => GlobalCommand::MuteGroup { group, on },
        GroupKind::Solo => GlobalCommand::SoloGroup { group, on },
        GroupKind::Fader => GlobalCommand::FaderGroup { group, on },
    })
}

fn parse_channel(
    bus: Bus,
    channel: &str,
    rest: &[&str],
    args: &[OscType],
) -> Result<GlobalCommand, DecodeError> {
    let channel = index(channel)?;
    if channel >= CHANNELS {
        return Err(DecodeError::InvalidIndex);
    }
    let control = match rest {
        ["mute"] => ChannelControl::Mute(bool_arg(args)?),
        ["stereo"] => ChannelControl::Stereo(bool_arg(args)?),
        ["record"] => ChannelControl::Record(bool_arg(args)?),
        ["name"] => ChannelControl::Name(string_arg(args)?),
        ["playchan"] => ChannelControl::PlayChannel(index_arg(args)?),
        ["msproc"] => ChannelControl::MsProc(bool_arg(args)?),
        ["phase"] => ChannelControl::Phase(bool_arg(args)?),
        ["width"] => ChannelControl::Width(float_arg(args)?),
        ["pfl"] => ChannelControl::Pfl(bool_arg(args)?),
        ["gain"] => ChannelControl::Gain(float_arg(args)?),
        ["reflevel"] => ChannelControl::RefLevel(index_arg(args)?),
        ["48v"] => ChannelControl::Phantom(bool_arg(args)?),
        ["instrument"] => ChannelControl::Instrument(bool_arg(args)?),
        ["pad"] => ChannelControl::Pad(bool_arg(args)?),
        ["autoset"] => ChannelControl::Autoset(bool_arg(args)?),
        ["volume"] => ChannelControl::Volume(float_arg(args)?),
        ["faderlin"] => ChannelControl::VolumeLin(float_arg(args)?),
        ["balpan"] => ChannelControl::Balance(float_arg(args)?),
        ["fxsend"] => ChannelControl::FxSend(float_arg(args)?),
        ["fxreturn"] => ChannelControl::FxReturn(float_arg(args)?),
        ["crossfeed"] => ChannelControl::Crossfeed(float_arg(args)?),
        ["loopback"] => ChannelControl::Loopback(bool_arg(args)?),
        ["talkbacksel"] => ChannelControl::TalkbackSel(bool_arg(args)?),
        ["delay"] => ChannelControl::Delay(float_arg(args)?),
        ["color"] => ChannelControl::Color(index_arg(args)?),
        ["lowcut", field] => ChannelControl::LowCut(parse_lowcut(field, args)?),
        ["eq", field] => ChannelControl::Eq(parse_eq(field, args, 3)?),
        ["dynamics", field] => ChannelControl::Dynamics(parse_dynamics(field, args)?),
        ["autolevel", field] => ChannelControl::AutoLevel(parse_autolevel(field, args)?),
        ["roomeq", field] => {
            if bus != Bus::Output {
                return Err(DecodeError::UnknownPath);
            }
            ChannelControl::RoomEq(parse_eq(field, args, 9)?)
        }
        _ => return Err(DecodeError::UnknownPath),
    };
    Ok(GlobalCommand::Channel {
        bus,
        channel,
        control,
    })
}

fn parse_mix(field: &str, args: &[OscType]) -> Result<MixControl, DecodeError> {
    Ok(match field {
        "fader" => MixControl::Fader(float_arg(args)?),
        "faderlin" => MixControl::FaderLin(float_arg(args)?),
        "balpan" => MixControl::Balance(float_arg(args)?),
        "groupflags" => MixControl::GroupFlags(index_arg(args)? as u32),
        "solo" => MixControl::Solo(bool_arg(args)?),
        _ => return Err(DecodeError::UnknownPath),
    })
}

fn parse_lowcut(field: &str, args: &[OscType]) -> Result<LowCutControl, DecodeError> {
    Ok(match field {
        "enable" => LowCutControl::Enable(bool_arg(args)?),
        "freq" => LowCutControl::Freq(float_arg(args)?),
        "slope" => LowCutControl::Slope(float_arg(args)?),
        "loadpreset" => {
            trigger(args)?;
            LowCutControl::LoadPreset
        }
        _ => return Err(DecodeError::UnknownPath),
    })
}

fn parse_eq(field: &str, args: &[OscType], max_bands: usize) -> Result<EqControl, DecodeError> {
    match field {
        "enable" => return Ok(EqControl::Enable(bool_arg(args)?)),
        "loadpreset" => {
            trigger(args)?;
            return Ok(EqControl::LoadPreset);
        }
        _ => {}
    }
    let Some((band, kind)) = band_field(field) else {
        return Err(DecodeError::UnknownPath);
    };
    if band == 0 || band > max_bands {
        return Err(DecodeError::InvalidIndex);
    }
    Ok(match kind {
        "gain" => EqControl::BandGain {
            band,
            db: float_arg(args)?,
        },
        "freq" => EqControl::BandFreq {
            band,
            hz: float_arg(args)?,
        },
        "q" => EqControl::BandQ {
            band,
            q: float_arg(args)?,
        },
        "type" => EqControl::BandType {
            band,
            kind: index_arg(args)?,
        },
        _ => return Err(DecodeError::UnknownPath),
    })
}

fn parse_dynamics(field: &str, args: &[OscType]) -> Result<DynamicsControl, DecodeError> {
    Ok(match field {
        "enable" => DynamicsControl::Enable(bool_arg(args)?),
        "gain" => DynamicsControl::Gain(float_arg(args)?),
        "attack" => DynamicsControl::Attack(float_arg(args)?),
        "release" => DynamicsControl::Release(float_arg(args)?),
        "compthres" => DynamicsControl::CompThreshold(float_arg(args)?),
        "compratio" => DynamicsControl::CompRatio(float_arg(args)?),
        "expthres" => DynamicsControl::ExpThreshold(float_arg(args)?),
        "expratio" => DynamicsControl::ExpRatio(float_arg(args)?),
        "loadpreset" => {
            trigger(args)?;
            DynamicsControl::LoadPreset
        }
        _ => return Err(DecodeError::UnknownPath),
    })
}

fn parse_autolevel(field: &str, args: &[OscType]) -> Result<AutoLevelControl, DecodeError> {
    Ok(match field {
        "enable" => AutoLevelControl::Enable(bool_arg(args)?),
        "maxgain" => AutoLevelControl::MaxGain(float_arg(args)?),
        "headroom" => AutoLevelControl::Headroom(float_arg(args)?),
        "risetime" => AutoLevelControl::RiseTime(float_arg(args)?),
        "loadpreset" => {
            trigger(args)?;
            AutoLevelControl::LoadPreset
        }
        _ => return Err(DecodeError::UnknownPath),
    })
}

fn parse_fx(field: &str, args: &[OscType], echo: bool) -> Result<FxControl, DecodeError> {
    Ok(match field {
        "enable" => FxControl::Enable(bool_arg(args)?),
        "type" => FxControl::Type(index_arg(args)?),
        "predelay" if !echo => FxControl::Predelay(float_arg(args)?),
        "lowcut" if !echo => FxControl::LowCut(float_arg(args)?),
        "roomscale" if !echo => FxControl::RoomScale(float_arg(args)?),
        "attack" if !echo => FxControl::Attack(float_arg(args)?),
        "hold" if !echo => FxControl::Hold(float_arg(args)?),
        "release" if !echo => FxControl::Release(float_arg(args)?),
        "highcut" => FxControl::HighCut(float_arg(args)?),
        "time" if !echo => FxControl::Time(float_arg(args)?),
        "highdamp" if !echo => FxControl::HighDamp(float_arg(args)?),
        "smooth" if !echo => FxControl::Smooth(float_arg(args)?),
        "volume" => FxControl::Volume(float_arg(args)?),
        "width" => FxControl::Width(float_arg(args)?),
        "delay" if echo => FxControl::Delay(float_arg(args)?),
        "feedback" if echo => FxControl::Feedback(float_arg(args)?),
        "loadpreset" => {
            trigger(args)?;
            FxControl::LoadPreset
        }
        _ => return Err(DecodeError::UnknownPath),
    })
}

fn parse_control_room(field: &str, args: &[OscType]) -> Result<ControlRoomControl, DecodeError> {
    Ok(match field {
        "mainout" => ControlRoomControl::MainOut(channel_index_arg(args)?),
        "mainoutb" => ControlRoomControl::MainOutB(channel_index_arg(args)?),
        "phones1" => ControlRoomControl::Phones {
            slot: 1,
            channel: channel_index_arg(args)?,
        },
        "phones2" => ControlRoomControl::Phones {
            slot: 2,
            channel: channel_index_arg(args)?,
        },
        "phones3" => ControlRoomControl::Phones {
            slot: 3,
            channel: channel_index_arg(args)?,
        },
        "phones4" => ControlRoomControl::Phones {
            slot: 4,
            channel: channel_index_arg(args)?,
        },
        "mainmono" => ControlRoomControl::MainMono(bool_arg(args)?),
        "dimreduction" => ControlRoomControl::DimReduction(float_arg(args)?),
        "dim" => ControlRoomControl::Dim(bool_arg(args)?),
        "recallvolume" => ControlRoomControl::RecallVolume(float_arg(args)?),
        "recall" => {
            trigger(args)?;
            ControlRoomControl::Recall
        }
        "talkchannel" => ControlRoomControl::TalkChannel(channel_index_arg(args)?),
        "talkback" => ControlRoomControl::Talkback(bool_arg(args)?),
        "cuechan" => ControlRoomControl::CueChannel(cue_channel_arg(args)?),
        "externalin" => ControlRoomControl::ExternalIn(bool_arg(args)?),
        "extinchannel" => ControlRoomControl::ExtInChannel(channel_index_arg(args)?),
        "extingain" => ControlRoomControl::ExtInGain(float_arg(args)?),
        "linkab" => ControlRoomControl::LinkAB(bool_arg(args)?),
        "speakerb" => ControlRoomControl::SpeakerB(bool_arg(args)?),
        "mutefx" => ControlRoomControl::MuteFx(bool_arg(args)?),
        _ => return Err(DecodeError::UnknownPath),
    })
}

fn parse_durec(field: &str, args: &[OscType]) -> Result<DurecControl, DecodeError> {
    Ok(match field {
        "play" => {
            trigger(args)?;
            DurecControl::Play
        }
        "pause" => {
            trigger(args)?;
            DurecControl::Pause
        }
        "stop" => {
            trigger(args)?;
            DurecControl::Stop
        }
        "record" => {
            trigger(args)?;
            DurecControl::Record
        }
        "next" => {
            trigger(args)?;
            DurecControl::Next
        }
        "previous" => {
            trigger(args)?;
            DurecControl::Previous
        }
        "time" => DurecControl::Time(string_arg(args)?),
        "state" => DurecControl::State(string_arg(args)?),
        _ => return Err(DecodeError::UnknownPath),
    })
}

fn parse_status(field: &str, args: &[OscType]) -> Result<StatusControl, DecodeError> {
    Ok(match field {
        "device" => StatusControl::Device(string_arg(args)?),
        "connection" => StatusControl::Connection(bool_arg(args)?),
        "dsp" => StatusControl::Dsp(float_arg(args)?),
        _ => return Err(DecodeError::UnknownPath),
    })
}

/// `band<N><kind>` -> `(N, kind)`.
fn band_field(field: &str) -> Option<(usize, &str)> {
    let rest = field.strip_prefix("band")?;
    let end = rest.find(|c: char| !c.is_ascii_digit())?;
    let band: usize = rest[..end].parse().ok()?;
    Some((band, &rest[end..]))
}

fn index(value: &str) -> Result<usize, DecodeError> {
    value.parse().map_err(|_| DecodeError::InvalidIndex)
}

fn positive_index(value: &str) -> Result<usize, DecodeError> {
    let value = index(value)?;
    if value == 0 {
        Err(DecodeError::InvalidIndex)
    } else {
        Ok(value)
    }
}

fn bus_name(value: &str) -> Result<Bus, DecodeError> {
    match value {
        "input" | "in" => Ok(Bus::Input),
        "playback" | "pb" => Ok(Bus::Playback),
        "output" | "out" => Ok(Bus::Output),
        _ => Err(DecodeError::UnknownPath),
    }
}

fn float_arg(args: &[OscType]) -> Result<f32, DecodeError> {
    optional_float(args).ok_or(DecodeError::WrongValueType)
}

fn optional_float(args: &[OscType]) -> Option<f32> {
    match args.first()? {
        OscType::Float(v) => Some(*v),
        OscType::Double(v) => Some(*v as f32),
        OscType::Int(v) => Some(*v as f32),
        OscType::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
        _ => None,
    }
}

fn bool_arg(args: &[OscType]) -> Result<bool, DecodeError> {
    match args.first() {
        None => Err(DecodeError::MissingValue),
        Some(OscType::Bool(v)) => Ok(*v),
        Some(OscType::Int(v)) => Ok(*v != 0),
        Some(OscType::Float(v)) => Ok(*v >= 0.5),
        Some(OscType::Double(v)) => Ok(*v >= 0.5),
        Some(_) => Err(DecodeError::WrongValueType),
    }
}

fn string_arg(args: &[OscType]) -> Result<String, DecodeError> {
    match args.first() {
        None => Err(DecodeError::MissingValue),
        Some(OscType::String(v)) => Ok(v.clone()),
        Some(_) => Err(DecodeError::WrongValueType),
    }
}

fn index_arg(args: &[OscType]) -> Result<usize, DecodeError> {
    let v = float_arg(args)?;
    if !v.is_finite() || v < 0.0 {
        return Err(DecodeError::InvalidIndex);
    }
    Ok(v.round() as usize)
}

fn channel_index_arg(args: &[OscType]) -> Result<usize, DecodeError> {
    let index = index_arg(args)?;
    if index >= CHANNELS {
        Err(DecodeError::InvalidIndex)
    } else {
        Ok(index)
    }
}

fn cue_channel_arg(args: &[OscType]) -> Result<Option<usize>, DecodeError> {
    let v = float_arg(args)?;
    if !v.is_finite() {
        return Err(DecodeError::InvalidIndex);
    }
    if v < 0.0 {
        Ok(None)
    } else {
        let index = v.round() as usize;
        if index >= CHANNELS {
            Err(DecodeError::InvalidIndex)
        } else {
            Ok(Some(index))
        }
    }
}

fn trigger(args: &[OscType]) -> Result<(), DecodeError> {
    match args.first() {
        None => Ok(()),
        Some(OscType::Bool(true)) => Ok(()),
        Some(OscType::Int(v)) if *v != 0 => Ok(()),
        Some(OscType::Float(v)) if *v >= 0.5 => Ok(()),
        Some(OscType::Double(v)) if *v >= 0.5 => Ok(()),
        Some(_) => Err(DecodeError::Ignored),
    }
}

fn send_mode(args: &[OscType]) -> u8 {
    optional_float(args)
        .map(|v| v.round().clamp(1.0, 2.0) as u8)
        .unwrap_or(1)
}

/// Encode one command or feedback value as an OSC message.  Trigger-only
/// messages carry `1.0`; the reference accepts a bare message too, but a
/// value keeps the packet self-describing and works with both receivers.
pub fn encode(command: &GlobalCommand) -> OscPacket {
    let (addr, args) = match command {
        GlobalCommand::Channel {
            bus,
            channel,
            control,
        } => {
            let (suffix, args) = channel_wire(control);
            (format!("/{}/{}/{}", bus.path(), channel, suffix), args)
        }
        GlobalCommand::Mix {
            source,
            channel,
            submix,
            control,
        } => {
            let (suffix, args) = mix_wire(control);
            let path = source.mix_path().unwrap_or("in");
            (format!("/mix/{path}/{channel}/{submix}/{suffix}"), args)
        }
        GlobalCommand::ControlRoom(control) => control_room_wire(control),
        GlobalCommand::Reverb(control) => {
            let (field, args) = fx_wire(control);
            (format!("/reverb/{field}"), args)
        }
        GlobalCommand::Echo(control) => {
            let (field, args) = fx_wire(control);
            (format!("/echo/{field}"), args)
        }
        GlobalCommand::MuteGroup { group, on } => {
            (format!("/mutegroup/{group}"), vec![osc_bool(*on)])
        }
        GlobalCommand::SoloGroup { group, on } => {
            (format!("/sologroup/{group}"), vec![osc_bool(*on)])
        }
        GlobalCommand::FaderGroup { group, on } => {
            (format!("/fadergroup/{group}"), vec![osc_bool(*on)])
        }
        GlobalCommand::GlobalMute(on) => ("/globalmute".into(), vec![osc_bool(*on)]),
        GlobalCommand::GlobalSolo(on) => ("/globalsolo".into(), vec![osc_bool(*on)]),
        GlobalCommand::Undo => ("/undo".into(), vec![OscType::Float(1.0)]),
        GlobalCommand::Redo => ("/redo".into(), vec![OscType::Float(1.0)]),
        GlobalCommand::SnapshotLoad { number, state } => (
            format!("/snapshot/load/{number}"),
            vec![OscType::Float(*state)],
        ),
        GlobalCommand::SnapshotSave => ("/snapshot/save".into(), vec![OscType::Float(1.0)]),
        GlobalCommand::LayoutLoad(number) => {
            (format!("/layout/load/{number}"), vec![OscType::Float(1.0)])
        }
        GlobalCommand::Durec(control) => durec_wire(control),
        GlobalCommand::Send(control) => send_wire(control),
        GlobalCommand::ShowWindow(on) => ("/showwindow".into(), vec![osc_bool(*on)]),
        GlobalCommand::Level {
            bus,
            channel,
            level_db,
        } => (
            format!("/level/{}/{channel}", level_path(*bus)),
            vec![OscType::Float(*level_db)],
        ),
        GlobalCommand::Status(control) => status_wire(control),
    };
    OscPacket::Message(OscMessage { addr, args })
}

fn channel_wire(control: &ChannelControl) -> (String, Vec<OscType>) {
    match control {
        ChannelControl::Mute(v) => ("mute".into(), vec![osc_bool(*v)]),
        ChannelControl::Stereo(v) => ("stereo".into(), vec![osc_bool(*v)]),
        ChannelControl::Record(v) => ("record".into(), vec![osc_bool(*v)]),
        ChannelControl::Name(v) => ("name".into(), vec![OscType::String(v.clone())]),
        ChannelControl::PlayChannel(v) => ("playchan".into(), vec![OscType::Float(*v as f32)]),
        ChannelControl::MsProc(v) => ("msproc".into(), vec![osc_bool(*v)]),
        ChannelControl::Phase(v) => ("phase".into(), vec![osc_bool(*v)]),
        ChannelControl::Width(v) => ("width".into(), vec![OscType::Float(*v)]),
        ChannelControl::Pfl(v) => ("pfl".into(), vec![osc_bool(*v)]),
        ChannelControl::Gain(v) => ("gain".into(), vec![OscType::Float(*v)]),
        ChannelControl::RefLevel(v) => ("reflevel".into(), vec![OscType::Float(*v as f32)]),
        ChannelControl::Phantom(v) => ("48v".into(), vec![osc_bool(*v)]),
        ChannelControl::Instrument(v) => ("instrument".into(), vec![osc_bool(*v)]),
        ChannelControl::Pad(v) => ("pad".into(), vec![osc_bool(*v)]),
        ChannelControl::Autoset(v) => ("autoset".into(), vec![osc_bool(*v)]),
        ChannelControl::Volume(v) => ("volume".into(), vec![OscType::Float(*v)]),
        ChannelControl::VolumeLin(v) => ("faderlin".into(), vec![OscType::Float(*v)]),
        ChannelControl::Balance(v) => ("balpan".into(), vec![OscType::Float(*v)]),
        ChannelControl::FxSend(v) => ("fxsend".into(), vec![OscType::Float(*v)]),
        ChannelControl::FxReturn(v) => ("fxreturn".into(), vec![OscType::Float(*v)]),
        ChannelControl::Crossfeed(v) => ("crossfeed".into(), vec![OscType::Float(*v)]),
        ChannelControl::Loopback(v) => ("loopback".into(), vec![osc_bool(*v)]),
        ChannelControl::TalkbackSel(v) => ("talkbacksel".into(), vec![osc_bool(*v)]),
        ChannelControl::Delay(v) => ("delay".into(), vec![OscType::Float(*v)]),
        ChannelControl::Color(v) => ("color".into(), vec![OscType::Float(*v as f32)]),
        ChannelControl::LowCut(v) => {
            let (field, args) = lowcut_wire(v);
            (format!("lowcut/{field}"), args)
        }
        ChannelControl::Eq(v) => {
            let (field, args) = eq_wire(v);
            (format!("eq/{field}"), args)
        }
        ChannelControl::Dynamics(v) => {
            let (field, args) = dynamics_wire(v);
            (format!("dynamics/{field}"), args)
        }
        ChannelControl::AutoLevel(v) => {
            let (field, args) = autolevel_wire(v);
            (format!("autolevel/{field}"), args)
        }
        ChannelControl::RoomEq(v) => {
            let (field, args) = eq_wire(v);
            (format!("roomeq/{field}"), args)
        }
    }
}

fn mix_wire(control: &MixControl) -> (String, Vec<OscType>) {
    match control {
        MixControl::Fader(v) => ("fader".into(), vec![OscType::Float(*v)]),
        MixControl::FaderLin(v) => ("faderlin".into(), vec![OscType::Float(*v)]),
        MixControl::Balance(v) => ("balpan".into(), vec![OscType::Float(*v)]),
        MixControl::GroupFlags(v) => ("groupflags".into(), vec![OscType::Float(*v as f32)]),
        MixControl::Solo(v) => ("solo".into(), vec![osc_bool(*v)]),
    }
}

fn lowcut_wire(control: &LowCutControl) -> (String, Vec<OscType>) {
    match control {
        LowCutControl::Enable(v) => ("enable".into(), vec![osc_bool(*v)]),
        LowCutControl::Freq(v) => ("freq".into(), vec![OscType::Float(*v)]),
        LowCutControl::Slope(v) => ("slope".into(), vec![OscType::Float(*v)]),
        LowCutControl::LoadPreset => ("loadpreset".into(), vec![OscType::Float(1.0)]),
    }
}

fn eq_wire(control: &EqControl) -> (String, Vec<OscType>) {
    match control {
        EqControl::Enable(v) => ("enable".into(), vec![osc_bool(*v)]),
        EqControl::BandGain { band, db } => (format!("band{band}gain"), vec![OscType::Float(*db)]),
        EqControl::BandFreq { band, hz } => (format!("band{band}freq"), vec![OscType::Float(*hz)]),
        EqControl::BandQ { band, q } => (format!("band{band}q"), vec![OscType::Float(*q)]),
        EqControl::BandType { band, kind } => (
            format!("band{band}type"),
            vec![OscType::Float(*kind as f32)],
        ),
        EqControl::LoadPreset => ("loadpreset".into(), vec![OscType::Float(1.0)]),
    }
}

fn dynamics_wire(control: &DynamicsControl) -> (String, Vec<OscType>) {
    match control {
        DynamicsControl::Enable(v) => ("enable".into(), vec![osc_bool(*v)]),
        DynamicsControl::Gain(v) => ("gain".into(), vec![OscType::Float(*v)]),
        DynamicsControl::Attack(v) => ("attack".into(), vec![OscType::Float(*v)]),
        DynamicsControl::Release(v) => ("release".into(), vec![OscType::Float(*v)]),
        DynamicsControl::CompThreshold(v) => ("compthres".into(), vec![OscType::Float(*v)]),
        DynamicsControl::CompRatio(v) => ("compratio".into(), vec![OscType::Float(*v)]),
        DynamicsControl::ExpThreshold(v) => ("expthres".into(), vec![OscType::Float(*v)]),
        DynamicsControl::ExpRatio(v) => ("expratio".into(), vec![OscType::Float(*v)]),
        DynamicsControl::LoadPreset => ("loadpreset".into(), vec![OscType::Float(1.0)]),
    }
}

fn autolevel_wire(control: &AutoLevelControl) -> (String, Vec<OscType>) {
    match control {
        AutoLevelControl::Enable(v) => ("enable".into(), vec![osc_bool(*v)]),
        AutoLevelControl::MaxGain(v) => ("maxgain".into(), vec![OscType::Float(*v)]),
        AutoLevelControl::Headroom(v) => ("headroom".into(), vec![OscType::Float(*v)]),
        AutoLevelControl::RiseTime(v) => ("risetime".into(), vec![OscType::Float(*v)]),
        AutoLevelControl::LoadPreset => ("loadpreset".into(), vec![OscType::Float(1.0)]),
    }
}

fn fx_wire(control: &FxControl) -> (String, Vec<OscType>) {
    match control {
        FxControl::Enable(v) => ("enable".into(), vec![osc_bool(*v)]),
        FxControl::Type(v) => ("type".into(), vec![OscType::Float(*v as f32)]),
        FxControl::Predelay(v) => ("predelay".into(), vec![OscType::Float(*v)]),
        FxControl::LowCut(v) => ("lowcut".into(), vec![OscType::Float(*v)]),
        FxControl::RoomScale(v) => ("roomscale".into(), vec![OscType::Float(*v)]),
        FxControl::Attack(v) => ("attack".into(), vec![OscType::Float(*v)]),
        FxControl::Hold(v) => ("hold".into(), vec![OscType::Float(*v)]),
        FxControl::Release(v) => ("release".into(), vec![OscType::Float(*v)]),
        FxControl::HighCut(v) => ("highcut".into(), vec![OscType::Float(*v)]),
        FxControl::Time(v) => ("time".into(), vec![OscType::Float(*v)]),
        FxControl::HighDamp(v) => ("highdamp".into(), vec![OscType::Float(*v)]),
        FxControl::Smooth(v) => ("smooth".into(), vec![OscType::Float(*v)]),
        FxControl::Volume(v) => ("volume".into(), vec![OscType::Float(*v)]),
        FxControl::Width(v) => ("width".into(), vec![OscType::Float(*v)]),
        FxControl::Delay(v) => ("delay".into(), vec![OscType::Float(*v)]),
        FxControl::Feedback(v) => ("feedback".into(), vec![OscType::Float(*v)]),
        FxControl::LoadPreset => ("loadpreset".into(), vec![OscType::Float(1.0)]),
    }
}

fn control_room_wire(control: &ControlRoomControl) -> (String, Vec<OscType>) {
    match control {
        ControlRoomControl::MainOut(v) => (
            "/controlroom/mainout".into(),
            vec![OscType::Float(*v as f32)],
        ),
        ControlRoomControl::MainOutB(v) => (
            "/controlroom/mainoutb".into(),
            vec![OscType::Float(*v as f32)],
        ),
        ControlRoomControl::Phones { slot, channel } => (
            format!("/controlroom/phones{slot}"),
            vec![OscType::Float(*channel as f32)],
        ),
        ControlRoomControl::MainMono(v) => ("/controlroom/mainmono".into(), vec![osc_bool(*v)]),
        ControlRoomControl::DimReduction(v) => {
            ("/controlroom/dimreduction".into(), vec![OscType::Float(*v)])
        }
        ControlRoomControl::Dim(v) => ("/controlroom/dim".into(), vec![osc_bool(*v)]),
        ControlRoomControl::RecallVolume(v) => {
            ("/controlroom/recallvolume".into(), vec![OscType::Float(*v)])
        }
        ControlRoomControl::Recall => ("/controlroom/recall".into(), vec![OscType::Float(1.0)]),
        ControlRoomControl::TalkChannel(v) => (
            "/controlroom/talkchannel".into(),
            vec![OscType::Float(*v as f32)],
        ),
        ControlRoomControl::Talkback(v) => ("/controlroom/talkback".into(), vec![osc_bool(*v)]),
        ControlRoomControl::CueChannel(v) => (
            "/controlroom/cuechan".into(),
            vec![OscType::Float(match v {
                Some(v) => *v as f32,
                None => -1.0,
            })],
        ),
        ControlRoomControl::ExternalIn(v) => ("/controlroom/externalin".into(), vec![osc_bool(*v)]),
        ControlRoomControl::ExtInChannel(v) => (
            "/controlroom/extinchannel".into(),
            vec![OscType::Float(*v as f32)],
        ),
        ControlRoomControl::ExtInGain(v) => {
            ("/controlroom/extingain".into(), vec![OscType::Float(*v)])
        }
        ControlRoomControl::LinkAB(v) => ("/controlroom/linkab".into(), vec![osc_bool(*v)]),
        ControlRoomControl::SpeakerB(v) => ("/controlroom/speakerb".into(), vec![osc_bool(*v)]),
        ControlRoomControl::MuteFx(v) => ("/controlroom/mutefx".into(), vec![osc_bool(*v)]),
    }
}

fn durec_wire(control: &DurecControl) -> (String, Vec<OscType>) {
    match control {
        DurecControl::Play => ("/durec/play".into(), vec![OscType::Float(1.0)]),
        DurecControl::Pause => ("/durec/pause".into(), vec![OscType::Float(1.0)]),
        DurecControl::Stop => ("/durec/stop".into(), vec![OscType::Float(1.0)]),
        DurecControl::Record => ("/durec/record".into(), vec![OscType::Float(1.0)]),
        DurecControl::Next => ("/durec/next".into(), vec![OscType::Float(1.0)]),
        DurecControl::Previous => ("/durec/previous".into(), vec![OscType::Float(1.0)]),
        DurecControl::Time(v) => ("/durec/time".into(), vec![OscType::String(v.clone())]),
        DurecControl::State(v) => ("/durec/state".into(), vec![OscType::String(v.clone())]),
    }
}

fn send_wire(control: &SendControl) -> (String, Vec<OscType>) {
    match control {
        SendControl::SendAll(mode) => ("/sendall".into(), vec![OscType::Float(*mode as f32)]),
        SendControl::SendSettings => ("/sendsettings".into(), vec![OscType::Float(1.0)]),
        SendControl::SendChannel { bus, channel } => (
            format!("/sendchan/{}/{channel}", bus.path()),
            vec![OscType::Float(1.0)],
        ),
        SendControl::SendMix(mode) => ("/sendmix".into(), vec![OscType::Float(*mode as f32)]),
        SendControl::SendSubmix { submix, mode } => (
            format!("/sendsubmix/{submix}"),
            vec![OscType::Float(*mode as f32)],
        ),
        SendControl::SendState => ("/sendstate".into(), vec![OscType::Float(1.0)]),
    }
}

fn status_wire(control: &StatusControl) -> (String, Vec<OscType>) {
    match control {
        StatusControl::Device(v) => ("/status/device".into(), vec![OscType::String(v.clone())]),
        StatusControl::Connection(v) => ("/status/connection".into(), vec![osc_bool(*v)]),
        StatusControl::Dsp(v) => ("/status/dsp".into(), vec![OscType::Float(*v)]),
    }
}

fn level_path(bus: Bus) -> &'static str {
    match bus {
        Bus::Input => "in",
        Bus::Playback => "pb",
        Bus::Output => "out",
    }
}

fn osc_bool(value: bool) -> OscType {
    OscType::Float(if value { 1.0 } else { 0.0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(value: f32) -> Vec<OscType> {
        vec![OscType::Float(value)]
    }

    fn s(value: &str) -> Vec<OscType> {
        vec![OscType::String(value.into())]
    }

    #[test]
    fn parses_a_channel_mute() {
        assert_eq!(
            parse("/input/3/mute", &f(1.0)),
            Ok(GlobalCommand::Channel {
                bus: Bus::Input,
                channel: 3,
                control: ChannelControl::Mute(true),
            })
        );
    }

    #[test]
    fn parses_all_three_strips_absolutely() {
        for (bus, path) in [
            (Bus::Input, "/input/17/name"),
            (Bus::Playback, "/playback/17/name"),
            (Bus::Output, "/output/17/name"),
        ] {
            assert_eq!(
                parse(path, &s("Phones")),
                Ok(GlobalCommand::Channel {
                    bus,
                    channel: 17,
                    control: ChannelControl::Name("Phones".into()),
                })
            );
        }
    }

    #[test]
    fn parses_the_mix_matrix() {
        assert_eq!(
            parse("/mix/in/3/8/fader", &f(-37.0)),
            Ok(GlobalCommand::Mix {
                source: Bus::Input,
                channel: 3,
                submix: 8,
                control: MixControl::Fader(-37.0),
            })
        );
        assert_eq!(
            parse("/mix/pb/0/0/balpan", &f(-1.0)),
            Ok(GlobalCommand::Mix {
                source: Bus::Playback,
                channel: 0,
                submix: 0,
                control: MixControl::Balance(-1.0),
            })
        );
    }

    #[test]
    fn parses_nested_channel_controls() {
        assert_eq!(
            parse("/output/4/roomeq/band9q", &f(1.5)),
            Ok(GlobalCommand::Channel {
                bus: Bus::Output,
                channel: 4,
                control: ChannelControl::RoomEq(EqControl::BandQ { band: 9, q: 1.5 }),
            })
        );
        assert_eq!(
            parse("/input/0/eq/band3type", &f(2.0)),
            Ok(GlobalCommand::Channel {
                bus: Bus::Input,
                channel: 0,
                control: ChannelControl::Eq(EqControl::BandType { band: 3, kind: 2 }),
            })
        );
        assert_eq!(
            parse("/input/1/dynamics/compthres", &f(-12.0)),
            Ok(GlobalCommand::Channel {
                bus: Bus::Input,
                channel: 1,
                control: ChannelControl::Dynamics(DynamicsControl::CompThreshold(-12.0)),
            })
        );
    }

    #[test]
    fn rejects_room_eq_on_a_input() {
        assert_eq!(
            parse("/input/0/roomeq/enable", &f(1.0)),
            Err(DecodeError::UnknownPath)
        );
    }

    #[test]
    fn parses_control_room() {
        assert_eq!(
            parse("/controlroom/mainout", &f(2.0)),
            Ok(GlobalCommand::ControlRoom(ControlRoomControl::MainOut(2)))
        );
        assert_eq!(
            parse("/controlroom/phones3", &f(4.0)),
            Ok(GlobalCommand::ControlRoom(ControlRoomControl::Phones {
                slot: 3,
                channel: 4,
            }))
        );
        assert_eq!(
            parse("/controlroom/recall", &[]),
            Ok(GlobalCommand::ControlRoom(ControlRoomControl::Recall))
        );
        assert_eq!(
            parse("/controlroom/cuechan", &f(4.0)),
            Ok(GlobalCommand::ControlRoom(ControlRoomControl::CueChannel(
                Some(4)
            )))
        );
        assert_eq!(
            parse("/controlroom/cuechan", &f(-1.0)),
            Ok(GlobalCommand::ControlRoom(ControlRoomControl::CueChannel(
                None
            )))
        );
    }

    #[test]
    fn parses_fx_and_global_controls() {
        assert_eq!(
            parse("/reverb/predelay", &f(20.0)),
            Ok(GlobalCommand::Reverb(FxControl::Predelay(20.0)))
        );
        assert_eq!(
            parse("/echo/feedback", &f(0.5)),
            Ok(GlobalCommand::Echo(FxControl::Feedback(0.5)))
        );
        assert_eq!(
            parse("/globalmute", &f(1.0)),
            Ok(GlobalCommand::GlobalMute(true))
        );
        assert_eq!(parse("/undo", &[]), Ok(GlobalCommand::Undo));
        assert_eq!(
            parse("/mutegroup/2", &f(0.0)),
            Ok(GlobalCommand::MuteGroup {
                group: 2,
                on: false
            })
        );
    }

    #[test]
    fn parses_snapshots_and_sends() {
        assert_eq!(
            parse("/snapshot/load/3", &f(1.0)),
            Ok(GlobalCommand::SnapshotLoad {
                number: 3,
                state: 1.0,
            })
        );
        assert_eq!(
            parse("/snapshot/load/3", &f(2.0)),
            Ok(GlobalCommand::SnapshotLoad {
                number: 3,
                state: 2.0,
            })
        );
        assert_eq!(
            parse("/sendchan/playback/5", &f(1.0)),
            Ok(GlobalCommand::Send(SendControl::SendChannel {
                bus: Bus::Playback,
                channel: 5,
            }))
        );
        assert_eq!(
            parse("/sendsubmix/8", &f(2.0)),
            Ok(GlobalCommand::Send(SendControl::SendSubmix {
                submix: 8,
                mode: 2,
            }))
        );
    }

    #[test]
    fn parses_feedback_only_paths() {
        assert_eq!(
            parse("/level/out/4", &f(-12.0)),
            Ok(GlobalCommand::Level {
                bus: Bus::Output,
                channel: 4,
                level_db: -12.0,
            })
        );
        assert_eq!(
            parse("/status/device", &s("Fireface UCX")),
            Ok(GlobalCommand::Status(StatusControl::Device(
                "Fireface UCX".into()
            )))
        );
        assert_eq!(
            parse("/durec/state", &s("Play")),
            Ok(GlobalCommand::Durec(DurecControl::State("Play".into())))
        );
    }

    #[test]
    fn rejects_bad_indexes_and_values() {
        assert_eq!(
            parse("/input/18/mute", &f(1.0)),
            Err(DecodeError::InvalidIndex)
        );
        assert_eq!(
            parse("/mix/in/0/18/fader", &f(-3.0)),
            Err(DecodeError::InvalidIndex)
        );
        assert_eq!(
            parse("/snapshot/load/0", &f(1.0)),
            Err(DecodeError::InvalidIndex)
        );
        assert_eq!(
            parse("/input/0/mute", &s("on")),
            Err(DecodeError::WrongValueType)
        );
        assert_eq!(parse("/not/a/path", &f(1.0)), Err(DecodeError::UnknownPath));
        assert_eq!(
            parse("/controlroom/mainout", &f(18.0)),
            Err(DecodeError::InvalidIndex)
        );
    }

    #[test]
    fn ignores_a_false_trigger() {
        assert_eq!(parse("/undo", &f(0.0)), Err(DecodeError::Ignored));
        assert_eq!(parse("/durec/play", &f(0.0)), Err(DecodeError::Ignored));
        assert_eq!(
            parse("/snapshot/save", &[]),
            Ok(GlobalCommand::SnapshotSave)
        );
    }

    #[test]
    fn encodes_back_to_the_wire_path() {
        let packet = encode(&GlobalCommand::Mix {
            source: Bus::Input,
            channel: 3,
            submix: 8,
            control: MixControl::Fader(-37.0),
        });
        let OscPacket::Message(msg) = packet else {
            panic!("expected a message");
        };
        assert_eq!(msg.addr, "/mix/in/3/8/fader");
        assert_eq!(msg.args, f(-37.0));
    }

    #[test]
    fn round_trips_a_nested_control() {
        let command = GlobalCommand::Channel {
            bus: Bus::Output,
            channel: 4,
            control: ChannelControl::Eq(EqControl::BandType { band: 3, kind: 2 }),
        };
        let packet = encode(&command);
        let OscPacket::Message(msg) = packet else {
            panic!("expected a message");
        };
        assert_eq!(parse(&msg.addr, &msg.args), Ok(command));
    }

    #[test]
    fn round_trips_the_feedback_paths() {
        for command in [
            GlobalCommand::SnapshotLoad {
                number: 3,
                state: 2.0,
            },
            GlobalCommand::Send(SendControl::SendChannel {
                bus: Bus::Input,
                channel: 7,
            }),
            GlobalCommand::Level {
                bus: Bus::Output,
                channel: 4,
                level_db: -12.0,
            },
            GlobalCommand::Status(StatusControl::Device("Fireface UCX".into())),
        ] {
            let packet = encode(&command);
            let OscPacket::Message(msg) = packet else {
                panic!("expected a message");
            };
            assert_eq!(parse(&msg.addr, &msg.args), Ok(command));
        }
    }
}
