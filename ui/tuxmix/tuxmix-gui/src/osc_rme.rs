//! TotalMix FX OSC front-end.
//!
//! Implements the address space in `docs/spec/OSC-CONTROL-SPEC.md` so a
//! controller written for TotalMix FX drives TotalControl unchanged.  It
//! decodes navigation, pages 1-3 and the monitor controls; page 4 paths are
//! accepted and ignored.  The same module translates the GUI's internal
//! `OscOutbound` state into feedback for the page the controller currently
//! addresses.
//!
//! The RME surface is banked: page 1 addresses `bank + n`, page 2 addresses
//! one selected channel, and `/setSubmix` selects the output whose mix is
//! being edited.

use rosc::OscType;

use tuxmix_core::channel::{Dynamics, Echo, EqBandType, InputEq, InputLevel, Reverb};
use tuxmix_core::ChannelId;

use crate::app::Message;
use crate::osc::OscOutbound;

/// Which row the banked page-1 commands address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Bus {
    #[default]
    Input,
    Playback,
    Output,
}

/// OSC-side navigation state.  Not part of the mixer scene.
#[derive(Debug, Clone, Copy, Default)]
pub struct RmeState {
    pub bus: Bus,
    /// First channel of the page-1 bank.
    pub bank: usize,
    /// Channel selected for page 2.
    pub channel: usize,
    /// Output whose submix is being edited.
    pub submix: usize,
}

const CHANNELS: usize = 18;

/// The documented TotalMix fader curve: normalized position to dB.
fn fader_db(p: f32) -> f32 {
    let p = p.clamp(0.0, 1.0);
    let f = p * 1023.0;
    if f >= 649.0 {
        f * 0.0320855615 - 26.8235294118
    } else {
        -(f * f) / 11033.0 + f * 0.1497326203 - 65.0
    }
}

fn fader_gain(v: &OscType) -> Option<f32> {
    let p = arg_f32(v)?;
    let db = fader_db(p);
    if db <= -65.0 {
        Some(0.0)
    } else {
        Some(10f32.powf(db / 20.0).clamp(0.0, 2.0))
    }
}

fn pan_value(v: &OscType) -> Option<i8> {
    let p = arg_f32(v)?.clamp(0.0, 1.0);
    Some((p * 200.0 - 100.0).round() as i8)
}

/// The documented frequency curve: normalized position to Hz.
fn freq_hz(p: f32, lo: f32, hi: f32) -> u16 {
    let p = p.clamp(0.0, 1.0);
    (lo * (hi / lo).powf(p)).round() as u16
}

fn eq_type(v: &OscType) -> Option<EqBandType> {
    if let OscType::String(s) = v {
        return match s.to_ascii_lowercase().as_str() {
            "off" => Some(EqBandType::Off),
            "bell" | "peak" => Some(EqBandType::Bell),
            "lowshelf" | "low" => Some(EqBandType::LowShelf),
            "highshelf" | "high" => Some(EqBandType::HighShelf),
            _ => None,
        };
    }
    let p = arg_f32(v)?.clamp(0.0, 1.0);
    Some(match (p * 3.0).round() as u8 {
        0 => EqBandType::Off,
        1 => EqBandType::Bell,
        2 => EqBandType::LowShelf,
        _ => EqBandType::HighShelf,
    })
}

/// Page 2 names the bands `eqType1` … `eqType3`; the mixer stores them at
/// indices 0 … 2.
fn eq_band(path: &str, prefix: &str) -> Option<usize> {
    let band: usize = path.trim_start_matches(prefix).parse().ok()?;
    band.checked_sub(1).filter(|b| *b < 3)
}

fn arg_f32(v: &OscType) -> Option<f32> {
    match v {
        OscType::Float(f) => Some(*f),
        OscType::Double(d) => Some(*d as f32),
        OscType::Int(i) => Some(*i as f32),
        _ => None,
    }
}

fn arg_on(v: &OscType) -> Option<bool> {
    match v {
        OscType::Bool(b) => Some(*b),
        OscType::Int(i) => Some(*i != 0),
        OscType::Float(f) => Some(*f >= 0.5),
        _ => None,
    }
}

fn channel(bus: Bus, idx: usize) -> Option<ChannelId> {
    (idx < CHANNELS).then_some(match bus {
        Bus::Input => ChannelId::Input(idx),
        Bus::Playback => ChannelId::Playback(idx),
        Bus::Output => ChannelId::Output(idx),
    })
}

/// Bank-relative channel number from a page-1 index (1-based path suffix).
fn banked(st: &RmeState, n: usize) -> Option<usize> {
    let idx = st.bank + n.checked_sub(1)?;
    (idx < CHANNELS).then_some(idx)
}

fn reverb_field(path: &str) -> Option<&'static str> {
    Some(match path {
        "reverbEnable" => "on",
        "reverbType" => "type",
        "reverbPredelay" => "predelay",
        "reverbLowcut" => "lowcut",
        "reverbHighcut" => "highcut",
        "reverbRoomscale" => "roomscale",
        "reverbAttack" => "attack",
        "reverbHold" => "hold",
        "reverbRelease" => "release",
        "reverbTime" => "time",
        "reverbHighdamp" => "highdamp",
        "reverbSmooth" => "smooth",
        "reverbWidth" => "width",
        "reverbVolume" => "volume",
        _ => return None,
    })
}

fn echo_field(path: &str) -> Option<&'static str> {
    Some(match path {
        "echoEnable" => "on",
        "echoType" => "type",
        "echoDelaytime" => "delay",
        "echoFeedback" => "feedback",
        "echoWidth" => "width",
        "echoVolume" => "volume",
        _ => return None,
    })
}

/// Parse one message.  Returns the messages to dispatch, or `None` when the
/// path is outside this slice.
pub fn parse(addr: &str, args: &[OscType], st: &mut RmeState) -> Option<Vec<Message>> {
    let parts: Vec<&str> = addr.split('/').filter(|s| !s.is_empty()).collect();
    let first = args.first()?;
    match parts.as_slice() {
        // ---- none-paged navigation -------------------------------------
        ["setSubmix"] => {
            let idx = arg_f32(first)? as usize;
            if idx >= CHANNELS {
                return None;
            }
            st.submix = idx;
            Some(vec![Message::SelectOutput(idx)])
        }
        ["setBankStart"] => {
            st.channel = (arg_f32(first)? as usize).min(CHANNELS - 1);
            st.bank = st.channel;
            Some(Vec::new())
        }
        ["setOffsetInBank"] => {
            let off = arg_f32(first)? as usize;
            st.channel = (st.bank + off).min(CHANNELS - 1);
            Some(Vec::new())
        }

        // ---- page 1 navigation -----------------------------------------
        ["1", "busInput"] => {
            st.bus = Bus::Input;
            Some(Vec::new())
        }
        ["1", "busPlayback"] => {
            st.bus = Bus::Playback;
            Some(Vec::new())
        }
        ["1", "busOutput"] => {
            st.bus = Bus::Output;
            Some(Vec::new())
        }
        ["1", "track+"] => {
            st.bank = (st.bank + 1).min(CHANNELS - 1);
            Some(Vec::new())
        }
        ["1", "track-"] => {
            st.bank = st.bank.saturating_sub(1);
            Some(Vec::new())
        }
        ["1", "bank+"] => {
            st.bank = (st.bank + 8).min(CHANNELS - 1);
            Some(Vec::new())
        }
        ["1", "bank-"] => {
            st.bank = st.bank.saturating_sub(8);
            Some(Vec::new())
        }

        // ---- page 1 channel commands -----------------------------------
        ["1", vol] if vol.starts_with("volume") => {
            let n: usize = vol.trim_start_matches("volume").parse().ok()?;
            let idx = banked(st, n)?;
            let cid = channel(st.bus, idx)?;
            let gain = fader_gain(first)?;
            let out = if st.bus == Bus::Output {
                idx
            } else {
                st.submix
            };
            Some(vec![Message::VolumeChanged(cid, out, gain)])
        }
        ["1", pan] if pan.starts_with("pan") => {
            let n: usize = pan.trim_start_matches("pan").parse().ok()?;
            let idx = banked(st, n)?;
            let cid = channel(st.bus, idx)?;
            let value = pan_value(first)?;
            let out = if st.bus == Bus::Output {
                idx
            } else {
                st.submix
            };
            Some(vec![Message::PanChanged(cid, out, value)])
        }
        ["1", "mute", _, n] => {
            let idx = banked(st, n.parse().ok()?)?;
            let cid = channel(st.bus, idx)?;
            Some(vec![Message::Mute(cid, arg_on(first)?)])
        }
        ["1", "solo", _, n] => {
            let idx = banked(st, n.parse().ok()?)?;
            let cid = channel(st.bus, idx)?;
            Some(vec![Message::Solo(cid, arg_on(first)?)])
        }
        ["1", "cue", _, n] => {
            let idx = banked(st, n.parse().ok()?)?;
            Some(vec![Message::CueChanged(
                ChannelId::Output(idx),
                arg_on(first)?,
            )])
        }
        ["1", gain] if gain.starts_with("micgain") => {
            let n: usize = gain.trim_start_matches("micgain").parse().ok()?;
            let idx = banked(st, n)?;
            let db = (arg_f32(first)? * 65.0).round().max(0.0) as u32;
            Some(vec![Message::Gain(idx, db)])
        }
        ["1", "phantom", _, n] => {
            let idx = banked(st, n.parse().ok()?)?;
            Some(vec![Message::Phantom(idx, arg_on(first)?)])
        }

        // ---- page 2: selected channel ----------------------------------
        ["2", "volume"] => {
            let cid = channel(st.bus, st.channel)?;
            let out = if st.bus == Bus::Output {
                st.channel
            } else {
                st.submix
            };
            Some(vec![Message::VolumeChanged(cid, out, fader_gain(first)?)])
        }
        ["2", "pan"] => {
            let cid = channel(st.bus, st.channel)?;
            let out = if st.bus == Bus::Output {
                st.channel
            } else {
                st.submix
            };
            Some(vec![Message::PanChanged(cid, out, pan_value(first)?)])
        }
        ["2", "mute"] => {
            let cid = channel(st.bus, st.channel)?;
            Some(vec![Message::Mute(cid, arg_on(first)?)])
        }
        ["2", "solo"] => {
            let cid = channel(st.bus, st.channel)?;
            Some(vec![Message::Solo(cid, arg_on(first)?)])
        }
        ["2", "stereo"] => {
            let cid = channel(st.bus, st.channel)?;
            Some(vec![Message::StereoLinkChanged(cid, arg_on(first)?)])
        }
        ["2", "phase"] => {
            let cid = channel(st.bus, st.channel)?;
            Some(vec![Message::PhaseChanged(cid, arg_on(first)?)])
        }
        ["2", "phantom"] if st.bus == Bus::Input => {
            Some(vec![Message::Phantom(st.channel, arg_on(first)?)])
        }
        ["2", "instrument"] if st.bus == Bus::Input => {
            Some(vec![Message::Instrument(st.channel, arg_on(first)?)])
        }
        ["2", "pad"] if st.bus == Bus::Input => {
            Some(vec![Message::Pad(st.channel, arg_on(first)?)])
        }
        ["2", "autoset"] if st.bus == Bus::Input => {
            Some(vec![Message::Autoset(st.channel, arg_on(first)?)])
        }
        ["2", "refLevel"] if st.bus == Bus::Input => {
            let code = arg_f32(first)?.round().clamp(0.0, 2.0) as u16;
            Some(vec![Message::LevelChanged(
                st.channel,
                InputLevel::from_code(code).unwrap_or(InputLevel::Plus4dBu),
            )])
        }
        ["2", "gain"] if st.bus == Bus::Input => {
            let db = (arg_f32(first)? * 65.0).round().max(0.0) as u32;
            Some(vec![Message::Gain(st.channel, db)])
        }
        ["2", "msProc"] => match st.bus {
            Bus::Input => Some(vec![Message::InputMsProcChanged(
                st.channel,
                arg_on(first)?,
            )]),
            Bus::Playback => Some(vec![Message::PlaybackMsProcChanged(
                st.channel,
                arg_on(first)?,
            )]),
            Bus::Output => None,
        },
        ["2", "loopback"] => Some(vec![Message::LoopbackChanged(st.channel, arg_on(first)?)]),
        ["2", "cue"] if st.bus == Bus::Output => Some(vec![Message::CueChanged(
            ChannelId::Output(st.channel),
            arg_on(first)?,
        )]),
        ["2", "talkbackSel"] if st.bus == Bus::Output => {
            Some(vec![Message::TalkbackReceiveChanged(
                st.channel,
                arg_on(first)?,
            )])
        }
        ["2", "noTrim"] if st.bus == Bus::Output => {
            Some(vec![Message::NoTrimChanged(st.channel, arg_on(first)?)])
        }
        ["2", "width"] => {
            let cid = channel(st.bus, st.channel)?;
            Some(vec![Message::ChannelWidthChanged(
                cid,
                arg_f32(first)?.clamp(0.0, 1.0),
            )])
        }
        ["2", "reverbSend"] => {
            let cid = channel(st.bus, st.channel)?;
            let db = -65.0 * (1.0 - arg_f32(first)?.clamp(0.0, 1.0));
            Some(vec![Message::FxSendChanged(cid, db)])
        }
        ["2", "reverbReturn"] if st.bus == Bus::Output => {
            let db = -65.0 * (1.0 - arg_f32(first)?.clamp(0.0, 1.0));
            Some(vec![Message::FxReturnChanged(st.channel, db)])
        }
        ["2", "lowcutEnable"] => {
            let cid = channel(st.bus, st.channel)?;
            Some(vec![Message::EqLowCutEnabled(cid, arg_on(first)?)])
        }
        ["2", "lowcutFreq"] => {
            let cid = channel(st.bus, st.channel)?;
            Some(vec![Message::EqLowCutFreq(
                cid,
                freq_hz(arg_f32(first)?, 20.0, 500.0),
            )])
        }
        ["2", "lowcutGrade"] => {
            let cid = channel(st.bus, st.channel)?;
            let p = arg_f32(first)?.clamp(0.0, 1.0);
            Some(vec![Message::EqLowCutSlope(
                cid,
                6 + (p * 3.0).round() as u8 * 6,
            )])
        }
        ["2", "eqEnable"] => {
            let cid = channel(st.bus, st.channel)?;
            Some(vec![Message::EqEnabled(cid, arg_on(first)?)])
        }
        ["2", path] if path.starts_with("eqType") => {
            let band = eq_band(path, "eqType")?;
            let cid = channel(st.bus, st.channel)?;
            Some(vec![Message::EqBandType(cid, band, eq_type(first)?)])
        }
        ["2", path] if path.starts_with("eqGain") => {
            let band = eq_band(path, "eqGain")?;
            let cid = channel(st.bus, st.channel)?;
            let db = arg_f32(first)? * 48.0 - 24.0;
            Some(vec![Message::EqBandGain(cid, band, db.clamp(-24.0, 24.0))])
        }
        ["2", path] if path.starts_with("eqFreq") => {
            let band = eq_band(path, "eqFreq")?;
            let cid = channel(st.bus, st.channel)?;
            Some(vec![Message::EqBandFreq(
                cid,
                band,
                freq_hz(arg_f32(first)?, 20.0, 20_000.0),
            )])
        }
        ["2", path] if path.starts_with("eqQ") => {
            let band = eq_band(path, "eqQ")?;
            let cid = channel(st.bus, st.channel)?;
            let q = 10f32.powf(-1.0 + arg_f32(first)?.clamp(0.0, 1.0) * 2.0);
            Some(vec![Message::EqBandQ(cid, band, q.clamp(0.05, 10.0))])
        }

        // ---- page 3: reverb and echo -----------------------------------
        ["3", path] if path.starts_with("reverb") => {
            let field = reverb_field(path)?;
            Some(vec![Message::OscRmeFx {
                echo: false,
                field,
                value: arg_f32(first)?,
            }])
        }
        ["3", path] if path.starts_with("echo") => {
            let field = echo_field(path)?;
            Some(vec![Message::OscRmeFx {
                echo: true,
                field,
                value: arg_f32(first)?,
            }])
        }

        // ---- page 3: snapshots, undo -----------------------------------
        ["3", "snapshots", n, _] => {
            let n: u8 = n.parse().ok()?;
            (1..=8)
                .contains(&n)
                .then_some(vec![Message::SnapshotClicked(n)])
        }
        ["3", "undo"] => Some(vec![Message::Undo]),
        ["3", "redo"] => Some(vec![Message::Redo]),
        ["3", "muteGroups", ..] | ["3", "soloGroups", ..] | ["3", "faderGroups", ..] => {
            Some(Vec::new())
        }
        ["3", rec] if rec.starts_with("record") => Some(Vec::new()),
        ["4", ..] => Some(Vec::new()),

        // ---- monitor controls ------------------------------------------
        ["1", "mainDim"] => Some(vec![Message::Dim(arg_on(first)?)]),
        ["1", "mainRecall"] => Some(vec![Message::ControlRoomRecall]),
        ["1", "mainSpeakerB"] => Some(vec![Message::SpeakerB(arg_on(first)?)]),
        ["1", "mainMuteFx"] => Some(vec![Message::ControlRoomMuteFx(arg_on(first)?)]),
        ["1", "mainMono"] => Some(vec![Message::ControlRoomMono(arg_on(first)?)]),
        ["1", "mainTalkback"] => Some(vec![Message::TalkbackChanged(arg_on(first)?)]),
        ["1", "globalMute"] => arg_on(first).map(|on| {
            if on {
                vec![Message::GlobalMuteToggle]
            } else {
                Vec::new()
            }
        }),
        ["1", "globalSolo"] => arg_on(first).map(|on| {
            if on {
                vec![Message::GlobalSoloToggle]
            } else {
                Vec::new()
            }
        }),

        _ => None,
    }
}

/// A translated feedback message: the TotalMix path and its wire arguments.
pub type Feedback = (String, Vec<OscType>);

/// Page 1 shows this many channels starting at the bank start.
const BANK_SIZE: usize = 8;

/// Latest mixer values, updated from `OscOutbound` so a page selection can
/// resend everything the controller now addresses.
#[derive(Debug, Clone, Default)]
struct ChannelMirror {
    /// Send level per submix for inputs/playbacks; unused for outputs.
    volumes: Vec<Option<f32>>,
    /// Send pan per submix for inputs/playbacks; output balance at index 0.
    pans: Vec<Option<i8>>,
    /// Output master fader.
    master_volume: Option<f32>,
    mute: Option<bool>,
    solo: Option<bool>,
    cue: Option<bool>,
    gain: Option<u32>,
    phantom: Option<bool>,
    pad: Option<bool>,
    instrument: Option<bool>,
    autoset: Option<bool>,
    sensitivity: Option<bool>,
    ref_level: Option<u16>,
    phase: Option<bool>,
    eq: Option<InputEq>,
    stereo: Option<bool>,
    loopback: Option<bool>,
    fx_send_db: Option<f32>,
    fx_return_db: Option<f32>,
    ms_proc: Option<bool>,
    no_trim: Option<bool>,
    talkback: Option<bool>,
    width: Option<f32>,
    dynamics: Option<Dynamics>,
}

/// Feedback-side mirror of the mixer and the current page selection.
#[derive(Debug, Clone)]
pub struct RmeFeedback {
    inputs: Vec<ChannelMirror>,
    playbacks: Vec<ChannelMirror>,
    outputs: Vec<ChannelMirror>,
    reverb: Option<Reverb>,
    echo: Option<Echo>,
    dim: Option<bool>,
    speaker_b: Option<bool>,
    mono: Option<bool>,
    mute_fx: Option<bool>,
    talkback: Option<bool>,
}

impl Default for RmeFeedback {
    fn default() -> Self {
        Self {
            inputs: vec![ChannelMirror::default(); CHANNELS],
            playbacks: vec![ChannelMirror::default(); CHANNELS],
            outputs: vec![ChannelMirror::default(); CHANNELS],
            reverb: None,
            echo: None,
            dim: None,
            speaker_b: None,
            mono: None,
            mute_fx: None,
            talkback: None,
        }
    }
}

fn split_channel(cid: ChannelId) -> (Bus, usize) {
    match cid {
        ChannelId::Input(i) => (Bus::Input, i),
        ChannelId::Playback(i) => (Bus::Playback, i),
        ChannelId::Output(i) => (Bus::Output, i),
    }
}

fn parse_channel(kind: &str, id: &str) -> Option<ChannelId> {
    let idx: usize = id.parse().ok()?;
    if idx >= CHANNELS {
        return None;
    }
    Some(match kind {
        "input" => ChannelId::Input(idx),
        "playback" => ChannelId::Playback(idx),
        "output" => ChannelId::Output(idx),
        _ => return None,
    })
}

/// One-based page-1 fader number, if the channel is in the bank the
/// controller currently shows.
fn page1_slot(st: &RmeState, cid: ChannelId) -> Option<usize> {
    let (bus, idx) = split_channel(cid);
    if bus != st.bus || idx < st.bank || idx >= st.bank.saturating_add(BANK_SIZE) {
        return None;
    }
    Some(idx - st.bank + 1)
}

fn selected(st: &RmeState, cid: ChannelId) -> bool {
    let (bus, idx) = split_channel(cid);
    bus == st.bus && idx == st.channel
}

fn set_at<T: Copy>(values: &mut Vec<Option<T>>, idx: usize, value: T) {
    if values.len() <= idx {
        values.resize(idx + 1, None);
    }
    values[idx] = Some(value);
}

fn at<T: Copy>(values: &[Option<T>], idx: usize) -> Option<T> {
    values.get(idx).copied().flatten()
}

/// Linear gain to the documented TotalMix fader position.
fn fader_position(gain: f32) -> f32 {
    let db = if gain <= 0.0 {
        -65.0
    } else {
        (20.0 * gain.log10()).clamp(-65.0, 6.0)
    };
    if db <= -65.0 {
        0.0
    } else if db >= -6.0 {
        ((db + 26.823_529_411_8) / 0.032_085_561_5) / 1023.0
    } else {
        (826.0 - (-34869.0 - 11033.0 * db).sqrt()) / 1023.0
    }
}

fn gain_db(gain: f32) -> f32 {
    if gain <= 0.0 {
        -65.0
    } else {
        (20.0 * gain.log10()).clamp(-65.0, 6.0)
    }
}

fn db_label(db: f32) -> String {
    if db <= -65.0 {
        "-oo".into()
    } else {
        format!("{db:.1} dB")
    }
}

fn pan_position(pan: i8) -> f32 {
    (f32::from(pan) + 100.0) / 200.0
}

fn pan_label(pan: i8) -> String {
    match pan {
        0 => "C".into(),
        p if p < 0 => format!("L{}", -p),
        p => format!("R{p}"),
    }
}

fn freq_position(hz: f32, lo: f32, hi: f32) -> f32 {
    if hz <= 0.0 {
        0.0
    } else {
        ((hz / lo).ln() / (hi / lo).ln()).clamp(0.0, 1.0)
    }
}

fn eq_type_position(t: EqBandType) -> Option<f32> {
    Some(match t {
        EqBandType::Off => 0.0,
        EqBandType::Bell => 1.0 / 3.0,
        EqBandType::LowShelf => 2.0 / 3.0,
        EqBandType::HighShelf => 1.0,
        EqBandType::LowPass => return None,
    })
}

/// The internal EQ type code used by `OscOutbound::Raw`, not the Lin01
/// value the controller sends.
fn eq_type_code(v: &OscType) -> EqBandType {
    match arg_f32(v).unwrap_or(0.0).round() as i32 {
        1 => EqBandType::Bell,
        2 => EqBandType::LowShelf,
        3 => EqBandType::HighShelf,
        4 => EqBandType::LowPass,
        _ => EqBandType::Off,
    }
}

fn eq_type_name(t: EqBandType) -> &'static str {
    match t {
        EqBandType::Off => "Off",
        EqBandType::Bell => "Bell",
        EqBandType::LowShelf => "Low Shelf",
        EqBandType::HighShelf => "High Shelf",
        EqBandType::LowPass => "Low Pass",
    }
}

fn lowcut_grade_position(slope: u8) -> f32 {
    match slope {
        6 => 0.0,
        12 => 1.0 / 3.0,
        18 => 2.0 / 3.0,
        _ => 1.0,
    }
}

fn lowcut_grade_name(slope: u8) -> String {
    format!("{slope} dB/oct")
}

fn fx_db_position(db: f32) -> f32 {
    ((db + 65.0) / 65.0).clamp(0.0, 1.0)
}

fn on_value(on: bool) -> OscType {
    OscType::Float(if on { 1.0 } else { 0.0 })
}

fn push_fader(msgs: &mut Vec<Feedback>, addr: impl Into<String>, gain: f32) {
    let addr = addr.into();
    msgs.push((addr.clone(), vec![OscType::Float(fader_position(gain))]));
    msgs.push((
        format!("{addr}Val"),
        vec![OscType::String(db_label(gain_db(gain)))],
    ));
}

fn push_pan(msgs: &mut Vec<Feedback>, addr: impl Into<String>, pan: i8) {
    let addr = addr.into();
    msgs.push((addr.clone(), vec![OscType::Float(pan_position(pan))]));
    msgs.push((format!("{addr}Val"), vec![OscType::String(pan_label(pan))]));
}

fn push_bool(msgs: &mut Vec<Feedback>, addr: impl Into<String>, on: bool) {
    msgs.push((addr.into(), vec![on_value(on)]));
}

impl RmeFeedback {
    fn channel(&self, cid: ChannelId) -> &ChannelMirror {
        match cid {
            ChannelId::Input(i) => &self.inputs[i],
            ChannelId::Playback(i) => &self.playbacks[i],
            ChannelId::Output(i) => &self.outputs[i],
        }
    }

    fn channel_mut(&mut self, cid: ChannelId) -> &mut ChannelMirror {
        match cid {
            ChannelId::Input(i) => &mut self.inputs[i],
            ChannelId::Playback(i) => &mut self.playbacks[i],
            ChannelId::Output(i) => &mut self.outputs[i],
        }
    }

    fn page1_volume(&self, st: &RmeState, cid: ChannelId, out: usize) -> Option<f32> {
        let mirror = self.channel(cid);
        match cid {
            ChannelId::Output(_) => mirror.master_volume,
            _ if out == st.submix => at(&mirror.volumes, out),
            _ => None,
        }
    }

    fn page1_pan(&self, st: &RmeState, cid: ChannelId, out: usize) -> Option<i8> {
        let mirror = self.channel(cid);
        match cid {
            ChannelId::Output(_) => at(&mirror.pans, 0),
            _ if out == st.submix => at(&mirror.pans, out),
            _ => None,
        }
    }

    fn emit_volume(&self, st: &RmeState, cid: ChannelId, out: usize, gain: f32) -> Vec<Feedback> {
        let mut msgs = Vec::new();
        let visible = matches!(cid, ChannelId::Output(_)) || out == st.submix;
        if let Some(n) = page1_slot(st, cid) {
            if visible && self.page1_volume(st, cid, out).is_some() {
                push_fader(&mut msgs, format!("/1/volume{n}"), gain);
            }
        }
        if visible && selected(st, cid) {
            push_fader(&mut msgs, "/2/volume", gain);
        }
        msgs
    }

    fn emit_pan(&self, st: &RmeState, cid: ChannelId, out: usize, pan: i8) -> Vec<Feedback> {
        let mut msgs = Vec::new();
        let visible = matches!(cid, ChannelId::Output(_)) || out == st.submix;
        if let Some(n) = page1_slot(st, cid) {
            if visible && self.page1_pan(st, cid, out).is_some() {
                push_pan(&mut msgs, format!("/1/pan{n}"), pan);
            }
        }
        if visible && selected(st, cid) {
            push_pan(&mut msgs, "/2/pan", pan);
        }
        msgs
    }

    fn emit_mute(&self, st: &RmeState, cid: ChannelId, on: bool) -> Vec<Feedback> {
        self.emit_onoff(st, cid, "mute", Some("/2/mute"), on)
    }

    fn emit_solo(&self, st: &RmeState, cid: ChannelId, on: bool) -> Vec<Feedback> {
        self.emit_onoff(st, cid, "solo", Some("/2/solo"), on)
    }

    fn emit_onoff(
        &self,
        st: &RmeState,
        cid: ChannelId,
        family: &str,
        page2: Option<&str>,
        on: bool,
    ) -> Vec<Feedback> {
        let mut msgs = Vec::new();
        if let Some(n) = page1_slot(st, cid) {
            push_bool(&mut msgs, format!("/1/{family}/1/{n}"), on);
        }
        if selected(st, cid) {
            if let Some(addr) = page2 {
                push_bool(&mut msgs, addr, on);
            }
        }
        msgs
    }

    fn emit_selected_onoff(
        &self,
        st: &RmeState,
        cid: ChannelId,
        addr: &str,
        on: bool,
    ) -> Vec<Feedback> {
        let mut msgs = Vec::new();
        if selected(st, cid) {
            push_bool(&mut msgs, addr, on);
        }
        msgs
    }

    fn emit_selected_fader(
        &self,
        st: &RmeState,
        cid: ChannelId,
        addr: &str,
        value: f32,
    ) -> Vec<Feedback> {
        let mut msgs = Vec::new();
        if selected(st, cid) {
            msgs.push((addr.into(), vec![OscType::Float(value)]));
        }
        msgs
    }

    fn emit_selected_value(
        &self,
        st: &RmeState,
        cid: ChannelId,
        addr: &str,
        value: f32,
    ) -> Vec<Feedback> {
        self.emit_selected_fader(st, cid, addr, value)
    }

    /// Translate one GUI state change into the TotalMix feedback paths the
    /// controller can currently address.
    pub fn translate(&mut self, out: OscOutbound, st: &RmeState) -> Vec<Feedback> {
        match out {
            OscOutbound::Volume(cid, out, gain) => {
                match cid {
                    ChannelId::Output(_) => self.channel_mut(cid).master_volume = Some(gain),
                    _ => set_at(&mut self.channel_mut(cid).volumes, out, gain),
                }
                self.emit_volume(st, cid, out, gain)
            }
            OscOutbound::OutputVolume(id, gain) => {
                let cid = ChannelId::Output(id);
                self.channel_mut(cid).master_volume = Some(gain);
                self.emit_volume(st, cid, id, gain)
            }
            OscOutbound::Pan(cid, out, pan) => {
                match cid {
                    ChannelId::Output(_) => set_at(&mut self.channel_mut(cid).pans, 0, pan),
                    _ => set_at(&mut self.channel_mut(cid).pans, out, pan),
                }
                self.emit_pan(st, cid, out, pan)
            }
            OscOutbound::Mute(cid, on) => {
                self.channel_mut(cid).mute = Some(on);
                self.emit_mute(st, cid, on)
            }
            OscOutbound::Solo(cid, on) => {
                self.channel_mut(cid).solo = Some(on);
                self.emit_solo(st, cid, on)
            }
            OscOutbound::Raw { addr, args } => self.translate_raw(&addr, &args, st),
        }
    }

    /// Resend the state of the page the controller now addresses.
    pub fn page_dump(&self, st: &RmeState) -> Vec<Feedback> {
        let mut msgs = Vec::new();
        self.dump_page1(st, &mut msgs);
        self.dump_page2(st, &mut msgs);
        if let Some(on) = self.dim {
            push_bool(&mut msgs, "/1/mainDim", on);
        }
        if let Some(on) = self.speaker_b {
            push_bool(&mut msgs, "/1/mainSpeakerB", on);
        }
        if let Some(on) = self.mono {
            push_bool(&mut msgs, "/1/mainMono", on);
        }
        if let Some(on) = self.mute_fx {
            push_bool(&mut msgs, "/1/mainMuteFx", on);
        }
        if let Some(on) = self.talkback {
            push_bool(&mut msgs, "/1/mainTalkback", on);
        }
        msgs
    }

    fn dump_page1(&self, st: &RmeState, msgs: &mut Vec<Feedback>) {
        for n in 1..=BANK_SIZE {
            let Some(idx) = st.bank.checked_add(n - 1).filter(|idx| *idx < CHANNELS) else {
                break;
            };
            let cid = match st.bus {
                Bus::Input => ChannelId::Input(idx),
                Bus::Playback => ChannelId::Playback(idx),
                Bus::Output => ChannelId::Output(idx),
            };
            let mirror = self.channel(cid);
            if let Some(gain) = self.page1_volume(st, cid, st.submix) {
                push_fader(msgs, format!("/1/volume{n}"), gain);
            }
            if let Some(pan) = self.page1_pan(st, cid, st.submix) {
                push_pan(msgs, format!("/1/pan{n}"), pan);
            }
            if let Some(on) = mirror.mute {
                push_bool(msgs, format!("/1/mute/1/{n}"), on);
            }
            if let Some(on) = mirror.solo {
                push_bool(msgs, format!("/1/solo/1/{n}"), on);
            }
            match st.bus {
                Bus::Input => {
                    if let Some(gain) = mirror.gain {
                        msgs.push((
                            format!("/1/micgain{n}"),
                            vec![OscType::Float(gain as f32 / 65.0)],
                        ));
                    }
                    if let Some(on) = mirror.phantom {
                        push_bool(msgs, format!("/1/phantom/1/{n}"), on);
                    }
                }
                Bus::Output => {
                    if let Some(on) = mirror.cue {
                        push_bool(msgs, format!("/1/cue/1/{n}"), on);
                    }
                }
                Bus::Playback => {}
            }
        }
    }

    fn dump_page2(&self, st: &RmeState, msgs: &mut Vec<Feedback>) {
        let Some(cid) = channel(st.bus, st.channel) else {
            return;
        };
        let mirror = self.channel(cid);
        let out = match cid {
            ChannelId::Output(i) => i,
            _ => st.submix,
        };
        if let Some(gain) = self.page1_volume(st, cid, out) {
            push_fader(msgs, "/2/volume", gain);
        }
        if let Some(pan) = self.page1_pan(st, cid, out) {
            push_pan(msgs, "/2/pan", pan);
        }
        if let Some(on) = mirror.mute {
            push_bool(msgs, "/2/mute", on);
        }
        if let Some(on) = mirror.solo {
            push_bool(msgs, "/2/solo", on);
        }
        if let Some(on) = mirror.stereo {
            push_bool(msgs, "/2/stereo", on);
        }
        if let Some(on) = mirror.phase {
            push_bool(msgs, "/2/phase", on);
        }
        match st.bus {
            Bus::Input => {
                if let Some(gain) = mirror.gain {
                    msgs.push(("/2/gain".into(), vec![OscType::Float(gain as f32 / 65.0)]));
                }
                if let Some(on) = mirror.phantom {
                    push_bool(msgs, "/2/phantom", on);
                }
                if let Some(on) = mirror.instrument {
                    push_bool(msgs, "/2/instrument", on);
                }
                if let Some(on) = mirror.pad {
                    push_bool(msgs, "/2/pad", on);
                }
                if let Some(on) = mirror.autoset {
                    push_bool(msgs, "/2/autoset", on);
                }
                if let Some(code) = mirror.ref_level {
                    msgs.push(("/2/refLevel".into(), vec![OscType::Float(f32::from(code))]));
                }
            }
            Bus::Playback => {}
            Bus::Output => {
                if let Some(on) = mirror.loopback {
                    push_bool(msgs, "/2/loopback", on);
                }
                if let Some(on) = mirror.cue {
                    push_bool(msgs, "/2/cue", on);
                }
                if let Some(on) = mirror.talkback {
                    push_bool(msgs, "/2/talkbackSel", on);
                }
                if let Some(on) = mirror.no_trim {
                    push_bool(msgs, "/2/noTrim", on);
                }
                if let Some(db) = mirror.fx_return_db {
                    msgs.push((
                        "/2/reverbReturn".into(),
                        vec![OscType::Float(fx_db_position(db))],
                    ));
                }
            }
        }
        if !matches!(cid, ChannelId::Output(_)) {
            if let Some(on) = mirror.ms_proc {
                push_bool(msgs, "/2/msProc", on);
            }
            if let Some(width) = mirror.width {
                msgs.push((
                    "/2/width".into(),
                    vec![OscType::Float(width.clamp(0.0, 1.0))],
                ));
            }
            if let Some(db) = mirror.fx_send_db {
                msgs.push((
                    "/2/reverbSend".into(),
                    vec![OscType::Float(fx_db_position(db))],
                ));
            }
        }
        if matches!(cid, ChannelId::Input(_) | ChannelId::Output(_)) {
            dump_eq(msgs, mirror);
            dump_dynamics(msgs, mirror);
        }
    }

    fn translate_raw(&mut self, addr: &str, args: &[OscType], st: &RmeState) -> Vec<Feedback> {
        let parts: Vec<&str> = addr.split('/').filter(|s| !s.is_empty()).collect();
        let Some(first) = args.first() else {
            return Vec::new();
        };

        match parts.as_slice() {
            ["meter", kind, id] => {
                let Some(cid) = parse_channel(kind, id) else {
                    return Vec::new();
                };
                let Some(value) = arg_f32(first) else {
                    return Vec::new();
                };
                let mut msgs = Vec::new();
                if let Some(n) = page1_slot(st, cid) {
                    msgs.push((format!("/1/level{n}Left"), vec![OscType::Float(value)]));
                    msgs.push((format!("/1/level{n}Right"), vec![OscType::Float(value)]));
                }
                if selected(st, cid) {
                    msgs.push(("/2/levelLeft".into(), vec![OscType::Float(value)]));
                    msgs.push(("/2/levelRight".into(), vec![OscType::Float(value)]));
                }
                return msgs;
            }
            ["fx", "reverb", field] => return self.translate_reverb(field, first),
            ["fx", "echo", field] => return self.translate_echo(field, first),
            ["dim"] => {
                let on = arg_on(first).unwrap_or(false);
                self.dim = Some(on);
                let mut msgs = Vec::new();
                push_bool(&mut msgs, "/1/mainDim", on);
                return msgs;
            }
            ["speaker_b"] => {
                let on = arg_on(first).unwrap_or(false);
                self.speaker_b = Some(on);
                let mut msgs = Vec::new();
                push_bool(&mut msgs, "/1/mainSpeakerB", on);
                return msgs;
            }
            ["mono"] => {
                let on = arg_on(first).unwrap_or(false);
                self.mono = Some(on);
                let mut msgs = Vec::new();
                push_bool(&mut msgs, "/1/mainMono", on);
                return msgs;
            }
            ["mute_fx"] => {
                let on = arg_on(first).unwrap_or(false);
                self.mute_fx = Some(on);
                let mut msgs = Vec::new();
                push_bool(&mut msgs, "/1/mainMuteFx", on);
                return msgs;
            }
            ["talkback"] => {
                let on = arg_on(first).unwrap_or(false);
                self.talkback = Some(on);
                let mut msgs = Vec::new();
                push_bool(&mut msgs, "/1/mainTalkback", on);
                return msgs;
            }
            _ => {}
        }

        let (cid, rest) = match parts.as_slice() {
            [kind, id, rest @ ..] if matches!(*kind, "input" | "playback" | "output") => {
                match parse_channel(kind, id) {
                    Some(cid) => (cid, rest),
                    None => return Vec::new(),
                }
            }
            _ => return Vec::new(),
        };

        match rest {
            ["volume", out] => {
                let out: usize = out.parse().unwrap_or(0);
                let Some(gain) = arg_f32(first) else {
                    return Vec::new();
                };
                set_at(&mut self.channel_mut(cid).volumes, out, gain);
                self.emit_volume(st, cid, out, gain)
            }
            ["mute"] => {
                let on = arg_on(first).unwrap_or(false);
                self.channel_mut(cid).mute = Some(on);
                self.emit_mute(st, cid, on)
            }
            ["solo"] => {
                let on = arg_on(first).unwrap_or(false);
                self.channel_mut(cid).solo = Some(on);
                self.emit_solo(st, cid, on)
            }
            ["gain"] => {
                let gain = arg_f32(first).unwrap_or(0.0).max(0.0).round() as u32;
                self.channel_mut(cid).gain = Some(gain);
                let mut msgs = Vec::new();
                if let Some(n) = page1_slot(st, cid) {
                    msgs.push((
                        format!("/1/micgain{n}"),
                        vec![OscType::Float(gain as f32 / 65.0)],
                    ));
                }
                if selected(st, cid) {
                    msgs.push(("/2/gain".into(), vec![OscType::Float(gain as f32 / 65.0)]));
                }
                msgs
            }
            ["phantom"] => {
                let on = arg_on(first).unwrap_or(false);
                self.channel_mut(cid).phantom = Some(on);
                self.emit_onoff(st, cid, "phantom", Some("/2/phantom"), on)
            }
            ["pad"] => {
                let on = arg_on(first).unwrap_or(false);
                self.channel_mut(cid).pad = Some(on);
                self.emit_selected_onoff(st, cid, "/2/pad", on)
            }
            ["instrument"] => {
                let on = arg_on(first).unwrap_or(false);
                self.channel_mut(cid).instrument = Some(on);
                self.emit_selected_onoff(st, cid, "/2/instrument", on)
            }
            ["autoset"] => {
                let on = arg_on(first).unwrap_or(false);
                self.channel_mut(cid).autoset = Some(on);
                self.emit_selected_onoff(st, cid, "/2/autoset", on)
            }
            ["sensitivity"] => {
                let on = arg_on(first).unwrap_or(false);
                let code = if on { 1 } else { 0 };
                let mirror = self.channel_mut(cid);
                mirror.sensitivity = Some(on);
                mirror.ref_level = Some(code);
                self.emit_selected_value(st, cid, "/2/refLevel", f32::from(code))
            }
            ["level"] => {
                let code = arg_f32(first).unwrap_or(0.0).round().clamp(0.0, 2.0) as u16;
                self.channel_mut(cid).ref_level = Some(code);
                self.emit_selected_value(st, cid, "/2/refLevel", f32::from(code))
            }
            ["phase"] => {
                let on = arg_on(first).unwrap_or(false);
                self.channel_mut(cid).phase = Some(on);
                self.emit_selected_onoff(st, cid, "/2/phase", on)
            }
            ["trim"] => Vec::new(),
            ["eq"] => {
                let on = arg_on(first).unwrap_or(false);
                self.channel_mut(cid)
                    .eq
                    .get_or_insert_with(InputEq::default)
                    .enabled = on;
                self.emit_selected_onoff(st, cid, "/2/eqEnable", on)
            }
            ["eq", band, "type"] => {
                let Some(band) = band.parse::<usize>().ok().filter(|b| *b < 3) else {
                    return Vec::new();
                };
                let t = eq_type_code(first);
                self.channel_mut(cid)
                    .eq
                    .get_or_insert_with(InputEq::default)
                    .bands[band]
                    .band_type = t;
                let Some(position) = eq_type_position(t) else {
                    return Vec::new();
                };
                let mut msgs = Vec::new();
                if selected(st, cid) {
                    msgs.push((
                        format!("/2/eqType{}", band + 1),
                        vec![OscType::Float(position)],
                    ));
                    msgs.push((
                        format!("/2/eqType{}Val", band + 1),
                        vec![OscType::String(eq_type_name(t).into())],
                    ));
                }
                msgs
            }
            ["eq", band, "freq"] => {
                let Some(band) = band.parse::<usize>().ok().filter(|b| *b < 3) else {
                    return Vec::new();
                };
                let hz = arg_f32(first).unwrap_or(20.0).round().clamp(20.0, 20_000.0) as u16;
                self.channel_mut(cid)
                    .eq
                    .get_or_insert_with(InputEq::default)
                    .bands[band]
                    .freq_hz = hz;
                self.emit_selected_fader(
                    st,
                    cid,
                    &format!("/2/eqFreq{}", band + 1),
                    freq_position(f32::from(hz), 20.0, 20_000.0),
                )
            }
            ["eq", band, "q"] => {
                let Some(band) = band.parse::<usize>().ok().filter(|b| *b < 3) else {
                    return Vec::new();
                };
                let q = arg_f32(first).unwrap_or(0.7).clamp(0.05, 10.0);
                self.channel_mut(cid)
                    .eq
                    .get_or_insert_with(InputEq::default)
                    .bands[band]
                    .q = q;
                self.emit_selected_fader(
                    st,
                    cid,
                    &format!("/2/eqQ{}", band + 1),
                    ((q.log10() + 1.0) / 2.0).clamp(0.0, 1.0),
                )
            }
            ["eq", band, "gain"] => {
                let Some(band) = band.parse::<usize>().ok().filter(|b| *b < 3) else {
                    return Vec::new();
                };
                let gain = arg_f32(first).unwrap_or(0.0).clamp(-24.0, 24.0);
                self.channel_mut(cid)
                    .eq
                    .get_or_insert_with(InputEq::default)
                    .bands[band]
                    .gain_db = gain;
                self.emit_selected_fader(
                    st,
                    cid,
                    &format!("/2/eqGain{}", band + 1),
                    (gain + 24.0) / 48.0,
                )
            }
            ["lowcut"] => {
                let on = arg_on(first).unwrap_or(false);
                self.channel_mut(cid)
                    .eq
                    .get_or_insert_with(InputEq::default)
                    .low_cut_enabled = on;
                self.emit_selected_onoff(st, cid, "/2/lowcutEnable", on)
            }
            ["lowcut", "freq"] => {
                let hz = arg_f32(first).unwrap_or(20.0).round().clamp(20.0, 500.0) as u16;
                self.channel_mut(cid)
                    .eq
                    .get_or_insert_with(InputEq::default)
                    .low_cut_freq_hz = hz;
                self.emit_selected_fader(
                    st,
                    cid,
                    "/2/lowcutFreq",
                    freq_position(f32::from(hz), 20.0, 500.0),
                )
            }
            ["lowcut", "slope"] => {
                let slope = arg_f32(first).unwrap_or(6.0).round() as u8;
                self.channel_mut(cid)
                    .eq
                    .get_or_insert_with(InputEq::default)
                    .low_cut_slope_db_oct = slope;
                let mut msgs = Vec::new();
                if selected(st, cid) {
                    msgs.push((
                        "/2/lowcutGrade".into(),
                        vec![OscType::Float(lowcut_grade_position(slope))],
                    ));
                    msgs.push((
                        "/2/lowcutGradeVal".into(),
                        vec![OscType::String(lowcut_grade_name(slope))],
                    ));
                }
                msgs
            }
            ["stereo"] => {
                let on = arg_on(first).unwrap_or(false);
                self.channel_mut(cid).stereo = Some(on);
                self.emit_selected_onoff(st, cid, "/2/stereo", on)
            }
            ["loopback"] => {
                let on = arg_on(first).unwrap_or(false);
                self.channel_mut(cid).loopback = Some(on);
                self.emit_selected_onoff(st, cid, "/2/loopback", on)
            }
            ["cue"] => {
                let on = arg_on(first).unwrap_or(false);
                self.channel_mut(cid).cue = Some(on);
                self.emit_onoff(st, cid, "cue", Some("/2/cue"), on)
            }
            ["fx_send"] => {
                let db = arg_f32(first).unwrap_or(-65.0);
                self.channel_mut(cid).fx_send_db = Some(db);
                self.emit_selected_fader(st, cid, "/2/reverbSend", fx_db_position(db))
            }
            ["fxreturn"] => {
                let db = arg_f32(first).unwrap_or(-65.0);
                self.channel_mut(cid).fx_return_db = Some(db);
                self.emit_selected_fader(st, cid, "/2/reverbReturn", fx_db_position(db))
            }
            ["ms_proc"] => {
                let on = arg_on(first).unwrap_or(false);
                self.channel_mut(cid).ms_proc = Some(on);
                self.emit_selected_onoff(st, cid, "/2/msProc", on)
            }
            ["width"] => {
                let width = arg_f32(first).unwrap_or(0.0);
                self.channel_mut(cid).width = Some(width);
                self.emit_selected_fader(st, cid, "/2/width", width.clamp(0.0, 1.0))
            }
            ["no_trim"] => {
                let on = arg_on(first).unwrap_or(false);
                self.channel_mut(cid).no_trim = Some(on);
                self.emit_selected_onoff(st, cid, "/2/noTrim", on)
            }
            ["talkback"] => {
                let on = arg_on(first).unwrap_or(false);
                self.channel_mut(cid).talkback = Some(on);
                self.emit_selected_onoff(st, cid, "/2/talkbackSel", on)
            }
            ["dynamics"] => {
                let on = arg_on(first).unwrap_or(false);
                self.channel_mut(cid)
                    .dynamics
                    .get_or_insert_with(Dynamics::default)
                    .on = on;
                self.emit_selected_onoff(st, cid, "/2/compexpEnable", on)
            }
            ["dynamics", "on"] => {
                let on = arg_on(first).unwrap_or(false);
                self.channel_mut(cid)
                    .dynamics
                    .get_or_insert_with(Dynamics::default)
                    .on = on;
                self.emit_selected_onoff(st, cid, "/2/compexpEnable", on)
            }
            ["dynamics", "gain"] => {
                let gain = arg_f32(first).unwrap_or(0.0);
                self.channel_mut(cid)
                    .dynamics
                    .get_or_insert_with(Dynamics::default)
                    .gain_db = gain;
                self.emit_selected_fader(st, cid, "/2/compexpGain", (gain + 30.0) / 60.0)
            }
            ["dynamics", "attack"] => {
                let ms = arg_f32(first).unwrap_or(10.0).max(0.0) as u16;
                self.channel_mut(cid)
                    .dynamics
                    .get_or_insert_with(Dynamics::default)
                    .attack_ms = ms;
                self.emit_selected_fader(st, cid, "/2/compexpAttack", f32::from(ms) / 200.0)
            }
            ["dynamics", "release"] => {
                let ms = arg_f32(first).unwrap_or(300.0).max(0.0) as u16;
                self.channel_mut(cid)
                    .dynamics
                    .get_or_insert_with(Dynamics::default)
                    .release_ms = ms;
                self.emit_selected_fader(
                    st,
                    cid,
                    "/2/compexpRelease",
                    (f32::from(ms) - 100.0) / 899.0,
                )
            }
            ["dynamics", "comp", "threshold"] => {
                let db = arg_f32(first).unwrap_or(-30.0);
                self.channel_mut(cid)
                    .dynamics
                    .get_or_insert_with(Dynamics::default)
                    .comp_threshold_db = db;
                self.emit_selected_fader(st, cid, "/2/compTrsh", (db + 60.0) / 60.0)
            }
            ["dynamics", "comp", "ratio"] => {
                let ratio = arg_f32(first).unwrap_or(1.0);
                self.channel_mut(cid)
                    .dynamics
                    .get_or_insert_with(Dynamics::default)
                    .comp_ratio = ratio;
                self.emit_selected_fader(st, cid, "/2/compRatio", (ratio - 1.0) / 9.0)
            }
            ["dynamics", "exp", "threshold"] => {
                let db = arg_f32(first).unwrap_or(-60.0);
                self.channel_mut(cid)
                    .dynamics
                    .get_or_insert_with(Dynamics::default)
                    .exp_threshold_db = db;
                self.emit_selected_fader(st, cid, "/2/expTrsh", (db + 99.0) / 69.0)
            }
            ["dynamics", "exp", "ratio"] => {
                let ratio = arg_f32(first).unwrap_or(1.0);
                self.channel_mut(cid)
                    .dynamics
                    .get_or_insert_with(Dynamics::default)
                    .exp_ratio = ratio;
                self.emit_selected_fader(st, cid, "/2/expRatio", (ratio - 1.0) / 9.0)
            }
            ["dynamics", "auto"] => {
                let on = arg_on(first).unwrap_or(false);
                self.channel_mut(cid)
                    .dynamics
                    .get_or_insert_with(Dynamics::default)
                    .auto_level = on;
                self.emit_selected_onoff(st, cid, "/2/alevEnable", on)
            }
            ["dynamics", "auto", "max"] => {
                let gain = arg_f32(first).unwrap_or(6.0);
                self.channel_mut(cid)
                    .dynamics
                    .get_or_insert_with(Dynamics::default)
                    .auto_max_gain_db = gain;
                self.emit_selected_fader(st, cid, "/2/alevMaxgain", gain / 18.0)
            }
            ["dynamics", "auto", "headroom"] => {
                let db = arg_f32(first).unwrap_or(6.0);
                self.channel_mut(cid)
                    .dynamics
                    .get_or_insert_with(Dynamics::default)
                    .auto_headroom_db = db;
                self.emit_selected_fader(st, cid, "/2/alevHeadroom", (db - 3.0) / 9.0)
            }
            ["dynamics", "auto", "rise"] => {
                let seconds = arg_f32(first).unwrap_or(5.0);
                self.channel_mut(cid)
                    .dynamics
                    .get_or_insert_with(Dynamics::default)
                    .auto_rise_s = seconds;
                self.emit_selected_fader(st, cid, "/2/alevRisetime", (seconds - 0.1) / 9.8)
            }
            _ => Vec::new(),
        }
    }

    fn translate_reverb(&mut self, field: &str, value: &OscType) -> Vec<Feedback> {
        let r = self.reverb.get_or_insert_with(Reverb::default);
        match field {
            "on" => r.on = arg_on(value).unwrap_or(false),
            "type" => r.kind = arg_f32(value).unwrap_or(0.0).round().clamp(0.0, 9.0) as u8,
            "predelay" => r.pre_delay_ms = arg_f32(value).unwrap_or(0.0).round() as u16,
            "lowcut" => r.low_cut_hz = arg_f32(value).unwrap_or(20.0).round() as u16,
            "highcut" => r.high_cut_hz = arg_f32(value).unwrap_or(5000.0).round() as u16,
            "room" => r.room_scale = arg_f32(value).unwrap_or(1.0),
            "attack" => r.attack_ms = arg_f32(value).unwrap_or(5.0).round() as u16,
            "hold" => r.hold_ms = arg_f32(value).unwrap_or(5.0).round() as u16,
            "release" => r.release_ms = arg_f32(value).unwrap_or(5.0).round() as u16,
            "time" => r.reverb_time_ds = arg_f32(value).unwrap_or(1.0).round() as u16,
            "highdamp" => r.high_damp_hz = arg_f32(value).unwrap_or(1000.0).round() as u16,
            "smooth" => {
                r.smoothness = arg_f32(value).unwrap_or(0.0).round().clamp(0.0, 100.0) as u8
            }
            "volume" => r.volume_db = arg_f32(value).unwrap_or(-65.0),
            "width" => r.width = arg_f32(value).unwrap_or(100.0),
            _ => return Vec::new(),
        }
        let mut msgs = Vec::new();
        match field {
            "on" => push_bool(&mut msgs, "/3/reverbEnable", r.on),
            "type" => msgs.push(("/3/reverbType".into(), vec![OscType::Float(r.kind as f32)])),
            "predelay" => msgs.push((
                "/3/reverbPredelay".into(),
                vec![OscType::Float(
                    (r.pre_delay_ms as f32 / 999.0).clamp(0.0, 1.0),
                )],
            )),
            "lowcut" => msgs.push((
                "/3/reverbLowcut".into(),
                vec![OscType::Float(freq_position(
                    r.low_cut_hz as f32,
                    20.0,
                    500.0,
                ))],
            )),
            "highcut" => msgs.push((
                "/3/reverbHighcut".into(),
                vec![OscType::Float(freq_position(
                    r.high_cut_hz as f32,
                    5000.0,
                    20_000.0,
                ))],
            )),
            "room" => msgs.push((
                "/3/reverbRoomscale".into(),
                vec![OscType::Float(((r.room_scale - 0.5) / 2.5).clamp(0.0, 1.0))],
            )),
            "attack" => msgs.push((
                "/3/reverbAttack".into(),
                vec![OscType::Float(
                    ((r.attack_ms as f32 - 5.0) / 395.0).clamp(0.0, 1.0),
                )],
            )),
            "hold" => msgs.push((
                "/3/reverbHold".into(),
                vec![OscType::Float(
                    ((r.hold_ms as f32 - 5.0) / 395.0).clamp(0.0, 1.0),
                )],
            )),
            "release" => msgs.push((
                "/3/reverbRelease".into(),
                vec![OscType::Float(
                    ((r.release_ms as f32 - 5.0) / 495.0).clamp(0.0, 1.0),
                )],
            )),
            "time" => msgs.push((
                "/3/reverbTime".into(),
                vec![OscType::Float(
                    ((r.reverb_time_ds as f32 - 1.0) / 49.0).clamp(0.0, 1.0),
                )],
            )),
            "highdamp" => msgs.push((
                "/3/reverbHighdamp".into(),
                vec![OscType::Float(freq_position(
                    r.high_damp_hz as f32,
                    1000.0,
                    20_000.0,
                ))],
            )),
            "smooth" => msgs.push((
                "/3/reverbSmooth".into(),
                vec![OscType::Float(r.smoothness as f32 / 100.0)],
            )),
            "width" => msgs.push((
                "/3/reverbWidth".into(),
                vec![OscType::Float((r.width / 100.0).clamp(0.0, 1.0))],
            )),
            "volume" => {
                msgs.push((
                    "/3/reverbVolume".into(),
                    vec![OscType::Float(fx_db_position(r.volume_db))],
                ));
                msgs.push((
                    "/3/reverbVolumeVal".into(),
                    vec![OscType::String(db_label(r.volume_db))],
                ));
            }
            _ => {}
        }
        msgs
    }

    fn translate_echo(&mut self, field: &str, value: &OscType) -> Vec<Feedback> {
        let e = self.echo.get_or_insert_with(Echo::default);
        match field {
            "on" => e.on = arg_on(value).unwrap_or(false),
            "type" => e.kind = arg_f32(value).unwrap_or(0.0).round().clamp(0.0, 2.0) as u8,
            "delay" => e.delay_ms = arg_f32(value).unwrap_or(0.0).round() as u16,
            "feedback" => {
                e.feedback = arg_f32(value).unwrap_or(0.0).round().clamp(0.0, 100.0) as u8
            }
            "volume" => e.volume_db = arg_f32(value).unwrap_or(-65.0),
            "width" => e.width = arg_f32(value).unwrap_or(100.0),
            _ => return Vec::new(),
        }
        let mut msgs = Vec::new();
        match field {
            "on" => push_bool(&mut msgs, "/3/echoEnable", e.on),
            "type" => msgs.push(("/3/echoType".into(), vec![OscType::Float(e.kind as f32)])),
            "delay" => msgs.push((
                "/3/echoDelaytime".into(),
                vec![OscType::Float((e.delay_ms as f32 / 2000.0).clamp(0.0, 1.0))],
            )),
            "feedback" => msgs.push((
                "/3/echoFeedback".into(),
                vec![OscType::Float(e.feedback as f32 / 100.0)],
            )),
            "volume" => {
                msgs.push((
                    "/3/echoVolume".into(),
                    vec![OscType::Float(fx_db_position(e.volume_db))],
                ));
                msgs.push((
                    "/3/echoVolumeVal".into(),
                    vec![OscType::String(db_label(e.volume_db))],
                ));
            }
            "width" => msgs.push((
                "/3/echoWidth".into(),
                vec![OscType::Float((e.width / 100.0).clamp(0.0, 1.0))],
            )),
            _ => {}
        }
        msgs
    }
}

fn dump_eq(msgs: &mut Vec<Feedback>, mirror: &ChannelMirror) {
    let Some(eq) = &mirror.eq else {
        return;
    };
    push_bool(msgs, "/2/eqEnable", eq.enabled);
    push_bool(msgs, "/2/lowcutEnable", eq.low_cut_enabled);
    msgs.push((
        "/2/lowcutFreq".into(),
        vec![OscType::Float(freq_position(
            eq.low_cut_freq_hz as f32,
            20.0,
            500.0,
        ))],
    ));
    msgs.push((
        "/2/lowcutGrade".into(),
        vec![OscType::Float(lowcut_grade_position(
            eq.low_cut_slope_db_oct,
        ))],
    ));
    msgs.push((
        "/2/lowcutGradeVal".into(),
        vec![OscType::String(lowcut_grade_name(eq.low_cut_slope_db_oct))],
    ));
    for (band, values) in eq.bands.iter().enumerate() {
        let Some(position) = eq_type_position(values.band_type) else {
            continue;
        };
        msgs.push((
            format!("/2/eqType{}", band + 1),
            vec![OscType::Float(position)],
        ));
        msgs.push((
            format!("/2/eqType{}Val", band + 1),
            vec![OscType::String(eq_type_name(values.band_type).into())],
        ));
        msgs.push((
            format!("/2/eqFreq{}", band + 1),
            vec![OscType::Float(freq_position(
                values.freq_hz as f32,
                20.0,
                20_000.0,
            ))],
        ));
        msgs.push((
            format!("/2/eqQ{}", band + 1),
            vec![OscType::Float(
                ((values.q.log10() + 1.0) / 2.0).clamp(0.0, 1.0),
            )],
        ));
        msgs.push((
            format!("/2/eqGain{}", band + 1),
            vec![OscType::Float(
                ((values.gain_db + 24.0) / 48.0).clamp(0.0, 1.0),
            )],
        ));
    }
}

fn dump_dynamics(msgs: &mut Vec<Feedback>, mirror: &ChannelMirror) {
    let Some(d) = mirror.dynamics else {
        return;
    };
    push_bool(msgs, "/2/compexpEnable", d.on);
    msgs.push((
        "/2/compexpGain".into(),
        vec![OscType::Float(((d.gain_db + 30.0) / 60.0).clamp(0.0, 1.0))],
    ));
    msgs.push((
        "/2/compexpAttack".into(),
        vec![OscType::Float((d.attack_ms as f32 / 200.0).clamp(0.0, 1.0))],
    ));
    msgs.push((
        "/2/compexpRelease".into(),
        vec![OscType::Float(
            ((d.release_ms as f32 - 100.0) / 899.0).clamp(0.0, 1.0),
        )],
    ));
    msgs.push((
        "/2/compTrsh".into(),
        vec![OscType::Float(
            ((d.comp_threshold_db + 60.0) / 60.0).clamp(0.0, 1.0),
        )],
    ));
    msgs.push((
        "/2/compRatio".into(),
        vec![OscType::Float(((d.comp_ratio - 1.0) / 9.0).clamp(0.0, 1.0))],
    ));
    msgs.push((
        "/2/expTrsh".into(),
        vec![OscType::Float(
            ((d.exp_threshold_db + 99.0) / 69.0).clamp(0.0, 1.0),
        )],
    ));
    msgs.push((
        "/2/expRatio".into(),
        vec![OscType::Float(((d.exp_ratio - 1.0) / 9.0).clamp(0.0, 1.0))],
    ));
    push_bool(msgs, "/2/alevEnable", d.auto_level);
    msgs.push((
        "/2/alevMaxgain".into(),
        vec![OscType::Float((d.auto_max_gain_db / 18.0).clamp(0.0, 1.0))],
    ));
    msgs.push((
        "/2/alevHeadroom".into(),
        vec![OscType::Float(
            ((d.auto_headroom_db - 3.0) / 9.0).clamp(0.0, 1.0),
        )],
    ));
    msgs.push((
        "/2/alevRisetime".into(),
        vec![OscType::Float(
            ((d.auto_rise_s - 0.1) / 9.8).clamp(0.0, 1.0),
        )],
    ));
}

/// True when an incoming command changes which page the controller shows.
pub fn page_changed(addr: &str) -> bool {
    matches!(
        addr,
        "/setBankStart"
            | "/setOffsetInBank"
            | "/setSubmix"
            | "/1/busInput"
            | "/1/busPlayback"
            | "/1/busOutput"
            | "/1/track+"
            | "/1/track-"
            | "/1/bank+"
            | "/1/bank-"
            | "/2/track+"
            | "/2/track-"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page1_volume_uses_the_bank_and_submix() {
        let mut st = RmeState {
            bank: 2,
            submix: 1,
            ..Default::default()
        };
        let msgs = parse("/1/volume1", &[OscType::Float(0.5)], &mut st).unwrap();
        match &msgs[0] {
            Message::VolumeChanged(cid, out, v) => {
                assert_eq!(*cid, ChannelId::Input(2));
                assert_eq!(*out, 1);
                assert!(*v > 0.0);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn page1_volume_zero_is_silence() {
        let mut st = RmeState::default();
        let msgs = parse("/1/volume1", &[OscType::Float(0.0)], &mut st).unwrap();
        match &msgs[0] {
            Message::VolumeChanged(_, _, v) => assert_eq!(*v, 0.0),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn page1_pan_maps_lin01() {
        let mut st = RmeState::default();
        let msgs = parse("/1/pan1", &[OscType::Float(0.0)], &mut st).unwrap();
        assert!(matches!(msgs[0], Message::PanChanged(_, _, -100)));
    }

    #[test]
    fn bus_and_bank_navigation() {
        let mut st = RmeState::default();
        parse("/1/busPlayback", &[OscType::Int(1)], &mut st).unwrap();
        assert_eq!(st.bus, Bus::Playback);
        parse("/1/bank+", &[OscType::Int(1)], &mut st).unwrap();
        assert_eq!(st.bank, 8);
        parse("/1/track-", &[OscType::Int(1)], &mut st).unwrap();
        assert_eq!(st.bank, 7);
    }

    #[test]
    fn monitor_controls() {
        let mut st = RmeState::default();
        assert!(matches!(
            parse("/1/mainTalkback", &[OscType::Int(1)], &mut st).as_deref(),
            Some([Message::TalkbackChanged(true)])
        ));
        assert!(matches!(
            parse("/1/mainMono", &[OscType::Int(1)], &mut st).as_deref(),
            Some([Message::ControlRoomMono(true)])
        ));
        assert!(matches!(
            parse("/1/mainDim", &[OscType::Int(1)], &mut st).as_deref(),
            Some([Message::Dim(true)])
        ));
    }

    #[test]
    fn submix_selects_the_output() {
        let mut st = RmeState::default();
        let msgs = parse("/setSubmix", &[OscType::Int(3)], &mut st).unwrap();
        assert!(matches!(msgs.as_slice(), [Message::SelectOutput(3)]));
        assert_eq!(st.submix, 3);
    }

    #[test]
    fn unsupported_paths_return_none() {
        let mut st = RmeState::default();
        assert!(parse("/1/select/1/1", &[OscType::Int(1)], &mut st).is_none());
        assert!(parse("/2/compexpGain", &[OscType::Float(0.5)], &mut st).is_none());
    }

    #[test]
    fn page2_eq_band_paths_are_one_based() {
        let mut st = RmeState {
            channel: 2,
            ..Default::default()
        };
        let msgs = parse("/2/eqType1", &[OscType::Float(1.0)], &mut st).unwrap();
        assert!(matches!(
            msgs[0],
            Message::EqBandType(ChannelId::Input(2), 0, EqBandType::HighShelf)
        ));
        let msgs = parse("/2/eqFreq2", &[OscType::Float(1.0)], &mut st).unwrap();
        assert!(matches!(
            msgs[0],
            Message::EqBandFreq(ChannelId::Input(2), 1, 20_000)
        ));
        assert!(parse("/2/eqType3", &[OscType::Float(1.0)], &mut st).is_some());
    }

    fn value<'a>(msgs: &'a [Feedback], addr: &str) -> Option<&'a [OscType]> {
        msgs.iter()
            .find(|(path, _)| path == addr)
            .map(|(_, args)| args.as_slice())
    }

    #[test]
    fn feedback_page1_uses_the_bank_and_submix() {
        let st = RmeState {
            bank: 2,
            submix: 1,
            ..Default::default()
        };
        let mut fb = RmeFeedback::default();
        let msgs = fb.translate(OscOutbound::Volume(ChannelId::Input(2), 1, 1.0), &st);
        assert!(value(&msgs, "/1/volume1").is_some());
        assert_eq!(
            value(&msgs, "/1/volume1Val"),
            Some(&[OscType::String("0.0 dB".into())][..])
        );
        assert!(value(&msgs, "/2/volume").is_none());
    }

    #[test]
    fn feedback_only_sends_the_visible_bank() {
        let st = RmeState {
            bank: 8,
            channel: 10,
            ..Default::default()
        };
        let mut fb = RmeFeedback::default();
        assert!(fb
            .translate(OscOutbound::Volume(ChannelId::Input(0), 0, 1.0), &st)
            .is_empty());
        let msgs = fb.translate(OscOutbound::Volume(ChannelId::Input(8), 0, 1.0), &st);
        assert!(value(&msgs, "/1/volume1").is_some());
    }

    #[test]
    fn feedback_only_sends_the_selected_submix() {
        let st = RmeState {
            submix: 1,
            ..Default::default()
        };
        let mut fb = RmeFeedback::default();
        assert!(fb
            .translate(OscOutbound::Volume(ChannelId::Input(0), 0, 1.0), &st)
            .is_empty());
        let msgs = fb.translate(OscOutbound::Volume(ChannelId::Input(0), 1, 1.0), &st);
        assert!(value(&msgs, "/1/volume1").is_some());
    }

    #[test]
    fn feedback_page2_addresses_the_selected_channel() {
        let st = RmeState {
            bank: 8,
            channel: 2,
            ..Default::default()
        };
        let mut fb = RmeFeedback::default();
        let msgs = fb.translate(OscOutbound::Volume(ChannelId::Input(2), 0, 1.0), &st);
        assert!(value(&msgs, "/1/volume1").is_none());
        assert!(value(&msgs, "/2/volume").is_some());
        let msgs = fb.translate(OscOutbound::Mute(ChannelId::Input(2), true), &st);
        assert_eq!(value(&msgs, "/2/mute"), Some(&[on_value(true)][..]));
    }

    #[test]
    fn feedback_maps_fx_to_page3() {
        let st = RmeState::default();
        let mut fb = RmeFeedback::default();
        let msgs = fb.translate(
            OscOutbound::Raw {
                addr: "/fx/reverb/lowcut".into(),
                args: vec![OscType::Int(500)],
            },
            &st,
        );
        assert_eq!(
            value(&msgs, "/3/reverbLowcut"),
            Some(&[OscType::Float(1.0)][..])
        );
        let msgs = fb.translate(
            OscOutbound::Raw {
                addr: "/fx/reverb/volume".into(),
                args: vec![OscType::Float(-6.5)],
            },
            &st,
        );
        assert_eq!(
            value(&msgs, "/3/reverbVolume"),
            Some(&[OscType::Float(0.9)][..])
        );
        assert_eq!(
            value(&msgs, "/3/reverbVolumeVal"),
            Some(&[OscType::String("-6.5 dB".into())][..])
        );
        let msgs = fb.translate(
            OscOutbound::Raw {
                addr: "/fx/echo/delay".into(),
                args: vec![OscType::Int(1000)],
            },
            &st,
        );
        assert_eq!(
            value(&msgs, "/3/echoDelaytime"),
            Some(&[OscType::Float(0.5)][..])
        );
    }

    #[test]
    fn feedback_maps_meters_to_the_visible_page() {
        let st = RmeState {
            bank: 0,
            channel: 1,
            ..Default::default()
        };
        let mut fb = RmeFeedback::default();
        let msgs = fb.translate(
            OscOutbound::Raw {
                addr: "/meter/input/0".into(),
                args: vec![OscType::Float(0.25)],
            },
            &st,
        );
        assert!(value(&msgs, "/1/level1Left").is_some());
        assert!(value(&msgs, "/1/level1Right").is_some());
        assert!(value(&msgs, "/2/levelLeft").is_none());
        let msgs = fb.translate(
            OscOutbound::Raw {
                addr: "/meter/input/1".into(),
                args: vec![OscType::Float(0.5)],
            },
            &st,
        );
        assert_eq!(
            value(&msgs, "/2/levelLeft"),
            Some(&[OscType::Float(0.5)][..])
        );
        assert_eq!(
            value(&msgs, "/2/levelRight"),
            Some(&[OscType::Float(0.5)][..])
        );
    }

    #[test]
    fn page_dump_resends_the_selected_channel() {
        let st = RmeState {
            bank: 0,
            channel: 0,
            submix: 0,
            ..Default::default()
        };
        let mut fb = RmeFeedback::default();
        fb.translate(OscOutbound::Volume(ChannelId::Input(0), 0, 1.0), &st);
        fb.translate(OscOutbound::Mute(ChannelId::Input(0), false), &st);
        fb.translate(
            OscOutbound::Raw {
                addr: "/input/0/gain".into(),
                args: vec![OscType::Float(32.0)],
            },
            &st,
        );
        fb.translate(
            OscOutbound::Raw {
                addr: "/input/0/eq".into(),
                args: vec![on_value(true)],
            },
            &st,
        );
        fb.translate(
            OscOutbound::Raw {
                addr: "/input/0/eq/0/type".into(),
                args: vec![OscType::Int(1)],
            },
            &st,
        );
        fb.translate(
            OscOutbound::Raw {
                addr: "/dim".into(),
                args: vec![on_value(true)],
            },
            &st,
        );
        let msgs = fb.page_dump(&st);
        assert!(value(&msgs, "/1/volume1").is_some());
        assert!(value(&msgs, "/1/micgain1").is_some());
        assert!(value(&msgs, "/2/volume").is_some());
        assert!(value(&msgs, "/2/gain").is_some());
        assert_eq!(value(&msgs, "/2/eqEnable"), Some(&[on_value(true)][..]));
        assert_eq!(
            value(&msgs, "/2/eqType1Val"),
            Some(&[OscType::String("Bell".into())][..])
        );
        assert_eq!(value(&msgs, "/1/mainDim"), Some(&[on_value(true)][..]));
    }

    #[test]
    fn page_selection_commands_request_a_dump() {
        for path in [
            "/setBankStart",
            "/setOffsetInBank",
            "/setSubmix",
            "/1/busInput",
            "/1/bank+",
        ] {
            assert!(page_changed(path), "{path}");
        }
        assert!(!page_changed("/1/volume1"));
        assert!(!page_changed("/2/volume"));
    }
}
