//! TotalMix FX OSC front-end.
//!
//! Implements the classic RME OSC address space (the `OscTableTotalMix`
//! surface used by TotalMix FX 2.10/2.11).  The surface is banked: page 1
//! addresses `bank + n`, page 2 addresses one selected channel, and
//! `/setSubmix` selects the output whose mix is being edited.
//!
//! The parser is pure.  It only mutates [`RmeState`] and returns the
//! ordinary GUI messages to dispatch, so navigation and page feedback stay
//! testable without a device.

use rosc::OscType;

use totalcontrol_core::channel::{EqBandType, InputLevel};
use totalcontrol_core::ChannelId;

use crate::app::Message;

/// Which row the banked page-1 commands address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Bus {
    #[default]
    Input,
    Playback,
    Output,
}

/// OSC-side navigation state.  Not part of the mixer scene.
#[derive(Debug, Clone, Copy)]
pub struct RmeState {
    pub bus: Bus,
    /// First channel of the page-1 bank, relative to the row.
    pub bank: usize,
    /// Channel selected for page 2, relative to the row.
    pub channel: usize,
    /// Output pair whose submix is being edited.
    pub submix: usize,
    /// Current page, for the full-state re-send.
    pub page: u8,
    /// Preamp gain range of inputs 3/4 in raw units (24 on the UCX, 36 on
    /// the UC).  Inputs 1/2 always use 0..65.
    pub gain_max_34: u32,
    /// Number of hardware output pairs / submixes.
    pub submix_count: usize,
    /// Visible faders per page-1 bank.
    pub bank_size: usize,
    /// Channels exposed in each row: inputs, playbacks, outputs.
    pub counts: [usize; 3],
}

impl Default for RmeState {
    fn default() -> Self {
        Self {
            bus: Bus::Input,
            bank: 0,
            channel: 0,
            submix: 0,
            page: 1,
            gain_max_34: 24,
            submix_count: 9,
            bank_size: 8,
            counts: [CHANNELS; 3],
        }
    }
}

impl RmeState {
    /// The part of the navigation state that changes which controls are
    /// visible.  Used to decide when a full page re-send is needed.
    pub fn nav(self) -> (Bus, usize, usize, usize, u8) {
        (self.bus, self.bank, self.channel, self.submix, self.page)
    }
}

pub const CHANNELS: usize = 18;
pub const ROW_COUNT: usize = 3;
pub const TOTAL_CHANNELS: usize = CHANNELS * ROW_COUNT;

/// A page-2 or page-1 control whose reference scale is `Toggle`: a non-zero
/// value flips the current state, zero is ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RmeToggle {
    Mute(ChannelId),
    Solo(ChannelId),
    Phase(ChannelId),
    PhaseRight(ChannelId),
    Phantom(usize),
    Instrument(usize),
    Pad(usize),
    Autoset(usize),
    MsProc(ChannelId),
    Stereo(ChannelId),
    Cue(usize),
    TalkbackSel(usize),
    NoTrim(usize),
    Loopback(usize),
    LowCutEnable(ChannelId),
    EqEnable(ChannelId),
    CompexpEnable(ChannelId),
    AlevEnable(ChannelId),
    GlobalMute,
    GlobalSolo,
    MainDim,
    MainSpeakerB,
    MainMuteFx,
    MainMono,
    MainTalkback,
    SpeakerBLinked,
    ReverbEnable,
    EchoEnable,
}

/// The documented TotalMix fader curve: normalized position to dB.
pub fn fader_db(p: f32) -> f32 {
    let p = p.clamp(0.0, 1.0);
    let f = p * 1023.0;
    if f >= 649.0 {
        f * 0.0320855615 - 26.8235294118
    } else {
        -(f * f) / 11033.0 + f * 0.1497326203 - 65.0
    }
}

/// Inverse fader curve: dB to normalized position.
pub fn fader_norm(db: f32) -> f32 {
    let p = if db >= -6.0 {
        ((db + 26.8235294118) / 0.0320855615) / 1023.0
    } else {
        (826.0 - (-34869.0 - 11033.0 * db).max(0.0).sqrt()) / 1023.0
    };
    p.clamp(0.0, 1.0)
}

/// Linear gain to a fader position.  Zero is the bottom of the throw.
pub fn gain_to_fader(gain: f32) -> f32 {
    if gain <= 0.0 {
        0.0
    } else {
        fader_norm(20.0 * gain.log10())
    }
}

pub fn fader_gain(v: &OscType) -> Option<f32> {
    let p = arg_f32(v)?;
    let db = fader_db(p);
    if db <= -65.0 {
        Some(0.0)
    } else {
        Some(10f32.powf(db / 20.0).clamp(0.0, 2.0))
    }
}

pub fn pan_value(v: &OscType) -> Option<i8> {
    let p = arg_f32(v)?.clamp(0.0, 1.0);
    Some((p * 200.0 - 100.0).round() as i8)
}

pub fn pan_norm(pan: i8) -> f32 {
    (f32::from(pan) + 100.0) / 200.0
}

/// The documented frequency curve: normalized position to Hz.
pub fn freq_hz(p: f32, lo: f32, hi: f32) -> u16 {
    let p = p.clamp(0.0, 1.0);
    (lo * (hi / lo).powf(p)).round() as u16
}

pub fn hz_freq(hz: f32, lo: f32, hi: f32) -> f32 {
    if hz <= 0.0 {
        0.0
    } else {
        ((hz.ln() - lo.ln()) / (hi.ln() - lo.ln())).clamp(0.0, 1.0)
    }
}

fn eq_type_band1(v: &OscType) -> Option<EqBandType> {
    if let OscType::String(s) = v {
        return match s.to_ascii_lowercase().as_str() {
            "bell" | "peak" => Some(EqBandType::Bell),
            "lowshelf" | "low shelf" | "low" => Some(EqBandType::LowShelf),
            _ => None,
        };
    }
    let p = arg_f32(v)?.clamp(0.0, 1.0);
    Some(if p < 0.5 {
        EqBandType::Bell
    } else {
        EqBandType::LowShelf
    })
}

fn eq_type_band3(v: &OscType) -> Option<EqBandType> {
    if let OscType::String(s) = v {
        return match s.to_ascii_lowercase().as_str() {
            "bell" | "peak" => Some(EqBandType::Bell),
            "highshelf" | "high shelf" | "high" | "shelf" => Some(EqBandType::HighShelf),
            "lowpass" | "low pass" => Some(EqBandType::LowPass),
            _ => None,
        };
    }
    let p = arg_f32(v)?.clamp(0.0, 1.0);
    Some(match (p * 2.0).round() as u8 {
        0 => EqBandType::Bell,
        1 => EqBandType::HighShelf,
        _ => EqBandType::LowPass,
    })
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

fn arg_string(v: &OscType) -> Option<String> {
    match v {
        OscType::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn row_len(st: &RmeState, bus: Bus) -> usize {
    match bus {
        Bus::Input => st.counts[0],
        Bus::Playback => st.counts[1],
        Bus::Output => st.counts[2],
    }
    .min(CHANNELS)
}

fn channel(st: &RmeState, bus: Bus, idx: usize) -> Option<ChannelId> {
    (idx < row_len(st, bus)).then_some(match bus {
        Bus::Input => ChannelId::Input(idx),
        Bus::Playback => ChannelId::Playback(idx),
        Bus::Output => ChannelId::Output(idx),
    })
}

/// Bank-relative channel number from a page-1 index (1-based path suffix).
fn banked(st: &RmeState, n: usize) -> Option<usize> {
    let off = n.checked_sub(1)?;
    if off >= st.bank_size.max(1) {
        return None;
    }
    let idx = st.bank + off;
    (idx < row_len(st, st.bus)).then_some(idx)
}

fn selected(st: &RmeState) -> Option<ChannelId> {
    channel(st, st.bus, st.channel)
}

fn input_gain_max(st: &RmeState, idx: usize) -> u32 {
    if idx < 2 {
        65
    } else {
        st.gain_max_34.max(1)
    }
}

fn gain_raw(st: &RmeState, idx: usize, value: &OscType) -> Option<u32> {
    let p = arg_f32(value)?.clamp(0.0, 1.0);
    let max = input_gain_max(st, idx) as f32;
    Some((p * max).round().clamp(0.0, max) as u32)
}

fn set_bus(st: &mut RmeState, bus: Bus) {
    st.bus = bus;
    let last = row_len(st, bus).saturating_sub(1);
    st.bank = st.bank.min(last);
    st.channel = st.channel.min(last);
}

fn toggle(v: &OscType, target: RmeToggle) -> Option<Vec<Message>> {
    Some(if arg_on(v)? {
        vec![Message::RmeToggle(target)]
    } else {
        Vec::new()
    })
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

fn dynamics_field(path: &str) -> Option<&'static str> {
    Some(match path {
        "compexpGain" => "gain",
        "compexpAttack" => "attack",
        "compexpRelease" => "release",
        "compTrsh" => "comp_threshold",
        "compRatio" => "comp_ratio",
        "expTrsh" => "exp_threshold",
        "expRatio" => "exp_ratio",
        "alevMaxgain" => "auto_max_gain",
        "alevHeadroom" => "auto_headroom",
        "alevRisetime" => "auto_rise",
        _ => return None,
    })
}

/// Parse one message.  Returns the messages to dispatch, or `None` when the
/// path is outside the classic RME surface or unsupported by this device.
pub fn parse(addr: &str, args: &[OscType], st: &mut RmeState) -> Option<Vec<Message>> {
    let parts: Vec<&str> = addr.split('/').filter(|s| !s.is_empty()).collect();
    let first = args.first()?;
    match parts.as_slice() {
        // ---- non-paged navigation -------------------------------------
        ["setSubmix"] => {
            let idx = arg_f32(first)?.max(0.0) as usize;
            if idx >= st.submix_count.max(1) {
                return None;
            }
            st.submix = idx;
            Some(vec![Message::SelectOutput(idx)])
        }
        ["setBankStart"] => {
            let abs = arg_f32(first)?.max(0.0) as usize;
            if abs >= TOTAL_CHANNELS {
                return None;
            }
            let bus = match abs / CHANNELS {
                0 => Bus::Input,
                1 => Bus::Playback,
                _ => Bus::Output,
            };
            st.bus = bus;
            let idx = abs % CHANNELS;
            if idx >= row_len(st, bus) {
                return None;
            }
            st.bank = idx;
            st.channel = idx;
            Some(Vec::new())
        }
        ["setOffsetInBank"] => {
            let off = arg_f32(first)?.max(0.0) as usize;
            if off >= st.bank_size.max(1) {
                return None;
            }
            let last = row_len(st, st.bus).saturating_sub(1);
            st.channel = (st.bank + off).min(last);
            Some(Vec::new())
        }
        ["loadQuickWorkspace"] => None,

        // ---- page 1 navigation -----------------------------------------
        ["1", "busInput"] => {
            set_bus(st, Bus::Input);
            st.page = 1;
            Some(Vec::new())
        }
        ["1", "busPlayback"] => {
            set_bus(st, Bus::Playback);
            st.page = 1;
            Some(Vec::new())
        }
        ["1", "busOutput"] => {
            set_bus(st, Bus::Output);
            st.page = 1;
            Some(Vec::new())
        }
        ["1", "track+"] => {
            let last = row_len(st, st.bus).saturating_sub(1);
            st.bank = (st.bank + 1).min(last);
            st.channel = st.bank;
            st.page = 1;
            Some(Vec::new())
        }
        ["1", "track-"] => {
            st.bank = st.bank.saturating_sub(1);
            st.channel = st.bank;
            st.page = 1;
            Some(Vec::new())
        }
        ["1", "bank+"] => {
            let last = row_len(st, st.bus).saturating_sub(1);
            st.bank = (st.bank + st.bank_size.max(1)).min(last);
            st.channel = st.bank;
            st.page = 1;
            Some(Vec::new())
        }
        ["1", "bank-"] => {
            st.bank = st.bank.saturating_sub(st.bank_size.max(1));
            st.channel = st.bank;
            st.page = 1;
            Some(Vec::new())
        }

        // ---- page 1 channel commands -----------------------------------
        ["1", "mastervolume"] => {
            st.page = 1;
            Some(vec![Message::RmeMasterVolume(fader_gain(first)?)])
        }
        ["1", vol] if vol.starts_with("volume") => {
            st.page = 1;
            let n: usize = vol.trim_start_matches("volume").parse().ok()?;
            let idx = banked(st, n)?;
            let cid = channel(st, st.bus, idx)?;
            let gain = fader_gain(first)?;
            let out = if st.bus == Bus::Output {
                idx
            } else {
                st.submix
            };
            Some(vec![Message::VolumeChanged(cid, out, gain)])
        }
        ["1", pan] if pan.starts_with("pan") => {
            st.page = 1;
            let n: usize = pan.trim_start_matches("pan").parse().ok()?;
            let idx = banked(st, n)?;
            let cid = channel(st, st.bus, idx)?;
            let value = pan_value(first)?;
            let out = if st.bus == Bus::Output {
                idx
            } else {
                st.submix
            };
            Some(vec![Message::PanChanged(cid, out, value)])
        }
        ["1", "mute", _, n] => {
            st.page = 1;
            let idx = banked(st, n.parse().ok()?)?;
            let cid = channel(st, st.bus, idx)?;
            Some(vec![Message::Mute(cid, arg_on(first)?)])
        }
        ["1", "solo", _, n] => {
            st.page = 1;
            let idx = banked(st, n.parse().ok()?)?;
            let cid = channel(st, st.bus, idx)?;
            Some(vec![Message::Solo(cid, arg_on(first)?)])
        }
        ["1", "select", _, n] => {
            st.page = 1;
            let idx = banked(st, n.parse().ok()?)?;
            let cid = channel(st, st.bus, idx)?;
            Some(vec![Message::RmeSelect(cid, arg_on(first)?)])
        }
        ["1", "cue", _, n] => {
            st.page = 1;
            if st.bus != Bus::Output {
                return None;
            }
            let idx = banked(st, n.parse().ok()?)?;
            Some(vec![Message::CueChanged(
                ChannelId::Output(idx),
                arg_on(first)?,
            )])
        }
        ["1", gain] if gain.starts_with("micgain") => {
            st.page = 1;
            if st.bus != Bus::Input {
                return None;
            }
            let n: usize = gain.trim_start_matches("micgain").parse().ok()?;
            let idx = banked(st, n)?;
            Some(vec![Message::Gain(idx, gain_raw(st, idx, first)?)])
        }
        ["1", "phantom", _, n] => {
            st.page = 1;
            if st.bus != Bus::Input {
                return None;
            }
            let idx = banked(st, n.parse().ok()?)?;
            Some(vec![Message::Phantom(idx, arg_on(first)?)])
        }
        ["1", name] if name.starts_with("trackname") => {
            st.page = 1;
            let n: usize = name.trim_start_matches("trackname").parse().ok()?;
            let idx = banked(st, n)?;
            let cid = channel(st, st.bus, idx)?;
            Some(vec![Message::RmeRename(cid, arg_string(first)?)])
        }
        ["1", "labelSubmix"] | ["1", "trim"] => None,
        ["1", level] if level.starts_with("level") => None,

        // ---- page 1 monitor and global controls -------------------------
        ["1", "globalMute"] => toggle(first, RmeToggle::GlobalMute),
        ["1", "globalSolo"] => toggle(first, RmeToggle::GlobalSolo),
        ["1", "mainDim"] => toggle(first, RmeToggle::MainDim),
        ["1", "mainRecall"] => Some(if arg_on(first)? {
            vec![Message::ControlRoomRecall]
        } else {
            Vec::new()
        }),
        ["1", "mainSpeakerB"] => toggle(first, RmeToggle::MainSpeakerB),
        ["1", "speakerBLinked"] => toggle(first, RmeToggle::SpeakerBLinked),
        ["1", "mainMuteFx"] => toggle(first, RmeToggle::MainMuteFx),
        ["1", "mainMono"] => toggle(first, RmeToggle::MainMono),
        ["1", "mainTalkback"] => toggle(first, RmeToggle::MainTalkback),
        ["1", "mainExtIn"] => None,

        // ---- page 2 navigation ------------------------------------------
        ["2", "busInput"] => {
            set_bus(st, Bus::Input);
            st.page = 2;
            Some(Vec::new())
        }
        ["2", "busPlayback"] => {
            set_bus(st, Bus::Playback);
            st.page = 2;
            Some(Vec::new())
        }
        ["2", "busOutput"] => {
            set_bus(st, Bus::Output);
            st.page = 2;
            Some(Vec::new())
        }
        ["2", "track+"] => {
            let last = row_len(st, st.bus).saturating_sub(1);
            st.channel = (st.channel + 1).min(last);
            st.page = 2;
            Some(Vec::new())
        }
        ["2", "track-"] => {
            st.channel = st.channel.saturating_sub(1);
            st.page = 2;
            Some(Vec::new())
        }

        // ---- page 2: selected channel ----------------------------------
        ["2", "volume"] => {
            st.page = 2;
            let cid = selected(st)?;
            let out = if st.bus == Bus::Output {
                st.channel
            } else {
                st.submix
            };
            Some(vec![Message::VolumeChanged(cid, out, fader_gain(first)?)])
        }
        ["2", "pan"] => {
            st.page = 2;
            let cid = selected(st)?;
            let out = if st.bus == Bus::Output {
                st.channel
            } else {
                st.submix
            };
            Some(vec![Message::PanChanged(cid, out, pan_value(first)?)])
        }
        ["2", "mute"] => {
            st.page = 2;
            let cid = selected(st)?;
            toggle(first, RmeToggle::Mute(cid))
        }
        ["2", "solo"] => {
            st.page = 2;
            let cid = selected(st)?;
            toggle(first, RmeToggle::Solo(cid))
        }
        ["2", "stereo"] => {
            st.page = 2;
            let cid = selected(st)?;
            toggle(first, RmeToggle::Stereo(cid))
        }
        ["2", "phase"] => {
            st.page = 2;
            if st.bus == Bus::Playback {
                return None;
            }
            let cid = selected(st)?;
            toggle(first, RmeToggle::Phase(cid))
        }
        ["2", "phaseRight"] => {
            st.page = 2;
            if st.bus == Bus::Playback {
                return None;
            }
            let cid = selected(st)?;
            toggle(first, RmeToggle::PhaseRight(cid))
        }
        ["2", "phantom"] => {
            st.page = 2;
            if st.bus != Bus::Input {
                return None;
            }
            // `Toggle`; the UCX also accepts 0.0 as a direct off command.
            if !arg_on(first)? {
                Some(vec![Message::Phantom(st.channel, false)])
            } else {
                Some(vec![Message::RmeToggle(RmeToggle::Phantom(st.channel))])
            }
        }
        ["2", "instrument"] => {
            st.page = 2;
            if st.bus != Bus::Input {
                return None;
            }
            toggle(first, RmeToggle::Instrument(st.channel))
        }
        ["2", "pad"] => {
            st.page = 2;
            if st.bus != Bus::Input {
                return None;
            }
            toggle(first, RmeToggle::Pad(st.channel))
        }
        ["2", "autoset"] => {
            st.page = 2;
            if st.bus != Bus::Input {
                return None;
            }
            toggle(first, RmeToggle::Autoset(st.channel))
        }
        ["2", "refLevel"] => {
            st.page = 2;
            if st.bus != Bus::Input {
                return None;
            }
            let code = arg_f32(first)?.round().clamp(0.0, 3.0) as u16;
            Some(vec![Message::LevelChanged(
                st.channel,
                InputLevel::from_code(code)?,
            )])
        }
        ["2", "gain"] => {
            st.page = 2;
            if st.bus != Bus::Input {
                return None;
            }
            Some(vec![Message::Gain(
                st.channel,
                gain_raw(st, st.channel, first)?,
            )])
        }
        ["2", "gainRight"] => {
            st.page = 2;
            if st.bus != Bus::Input {
                return None;
            }
            Some(vec![Message::RmeGainRight(
                st.channel,
                gain_raw(st, st.channel, first)?,
            )])
        }
        ["2", "msProc"] => {
            st.page = 2;
            let cid = selected(st)?;
            match st.bus {
                Bus::Input | Bus::Playback => toggle(first, RmeToggle::MsProc(cid)),
                Bus::Output => None,
            }
        }
        ["2", "loopback"] => {
            st.page = 2;
            if st.bus != Bus::Output {
                return None;
            }
            toggle(first, RmeToggle::Loopback(st.channel))
        }
        ["2", "cue"] => {
            st.page = 2;
            if st.bus != Bus::Output {
                return None;
            }
            toggle(first, RmeToggle::Cue(st.channel))
        }
        ["2", "talkbackSel"] => {
            st.page = 2;
            if st.bus != Bus::Output {
                return None;
            }
            toggle(first, RmeToggle::TalkbackSel(st.channel))
        }
        ["2", "noTrim"] => {
            st.page = 2;
            if st.bus != Bus::Output {
                return None;
            }
            toggle(first, RmeToggle::NoTrim(st.channel))
        }
        ["2", "width"] => {
            st.page = 2;
            let cid = selected(st)?;
            if st.bus == Bus::Output {
                return None;
            }
            Some(vec![Message::ChannelWidthChanged(
                cid,
                arg_f32(first)?.clamp(0.0, 1.0),
            )])
        }
        ["2", "reverbSend"] => {
            st.page = 2;
            if st.bus == Bus::Output {
                return None;
            }
            let cid = selected(st)?;
            let db = -65.0 * (1.0 - arg_f32(first)?.clamp(0.0, 1.0));
            Some(vec![Message::FxSendChanged(cid, db)])
        }
        ["2", "reverbReturn"] => {
            st.page = 2;
            if st.bus != Bus::Output {
                return None;
            }
            let db = -65.0 * (1.0 - arg_f32(first)?.clamp(0.0, 1.0));
            Some(vec![Message::FxReturnChanged(st.channel, db)])
        }
        ["2", "lowcutEnable"] => {
            st.page = 2;
            if st.bus == Bus::Playback {
                return None;
            }
            let cid = selected(st)?;
            toggle(first, RmeToggle::LowCutEnable(cid))
        }
        ["2", "lowcutFreq"] => {
            st.page = 2;
            if st.bus == Bus::Playback {
                return None;
            }
            let cid = selected(st)?;
            Some(vec![Message::EqLowCutFreq(
                cid,
                freq_hz(arg_f32(first)?, 20.0, 500.0),
            )])
        }
        ["2", "lowcutGrade"] => {
            st.page = 2;
            if st.bus == Bus::Playback {
                return None;
            }
            let cid = selected(st)?;
            let p = arg_f32(first)?.clamp(0.0, 1.0);
            Some(vec![Message::EqLowCutSlope(
                cid,
                6 + (p * 3.0).round() as u8 * 6,
            )])
        }
        ["2", "eqEnable"] => {
            st.page = 2;
            if st.bus == Bus::Playback {
                return None;
            }
            let cid = selected(st)?;
            toggle(first, RmeToggle::EqEnable(cid))
        }
        ["2", "eqType1"] => {
            st.page = 2;
            if st.bus == Bus::Playback {
                return None;
            }
            let cid = selected(st)?;
            Some(vec![Message::EqBandType(cid, 0, eq_type_band1(first)?)])
        }
        ["2", "eqType3"] => {
            st.page = 2;
            if st.bus == Bus::Playback {
                return None;
            }
            let cid = selected(st)?;
            Some(vec![Message::EqBandType(cid, 2, eq_type_band3(first)?)])
        }
        ["2", path] if path.starts_with("eqType") => None,
        ["2", path] if path.starts_with("eqGain") => {
            st.page = 2;
            if st.bus == Bus::Playback {
                return None;
            }
            let band: usize = path
                .trim_start_matches("eqGain")
                .parse::<usize>()
                .ok()?
                .checked_sub(1)?;
            if band > 2 {
                return None;
            }
            let cid = selected(st)?;
            let db = arg_f32(first)? * 48.0 - 24.0;
            Some(vec![Message::EqBandGain(cid, band, db.clamp(-24.0, 24.0))])
        }
        ["2", path] if path.starts_with("eqFreq") => {
            st.page = 2;
            if st.bus == Bus::Playback {
                return None;
            }
            let band: usize = path
                .trim_start_matches("eqFreq")
                .parse::<usize>()
                .ok()?
                .checked_sub(1)?;
            if band > 2 {
                return None;
            }
            let cid = selected(st)?;
            Some(vec![Message::EqBandFreq(
                cid,
                band,
                freq_hz(arg_f32(first)?, 20.0, 20_000.0),
            )])
        }
        ["2", path] if path.starts_with("eqQ") => {
            st.page = 2;
            if st.bus == Bus::Playback {
                return None;
            }
            let band: usize = path
                .trim_start_matches("eqQ")
                .parse::<usize>()
                .ok()?
                .checked_sub(1)?;
            if band > 2 {
                return None;
            }
            let cid = selected(st)?;
            let q = 10f32.powf(-1.0 + arg_f32(first)?.clamp(0.0, 1.0) * 2.0);
            Some(vec![Message::EqBandQ(cid, band, q.clamp(0.05, 10.0))])
        }
        ["2", "compexpEnable"] => {
            st.page = 2;
            if st.bus == Bus::Playback {
                return None;
            }
            let cid = selected(st)?;
            toggle(first, RmeToggle::CompexpEnable(cid))
        }
        ["2", "alevEnable"] => {
            st.page = 2;
            if st.bus == Bus::Playback {
                return None;
            }
            let cid = selected(st)?;
            toggle(first, RmeToggle::AlevEnable(cid))
        }
        ["2", path] if dynamics_field(path).is_some() => {
            st.page = 2;
            if st.bus == Bus::Playback {
                return None;
            }
            let cid = selected(st)?;
            Some(vec![Message::RmeDynamicsField {
                channel: cid,
                field: dynamics_field(path)?,
                value: arg_f32(first)?,
            }])
        }
        ["2", "recordEnable"] | ["2", "playChannel"] => None,
        ["2", "trackname"] => {
            st.page = 2;
            Some(vec![Message::RmeRename(selected(st)?, arg_string(first)?)])
        }
        ["2", "labelSubmix"] | ["2", "levelLeft"] | ["2", "levelRight"] => None,

        // ---- page 3: reverb and echo -----------------------------------
        ["3", "reverbEnable"] => {
            st.page = 3;
            toggle(first, RmeToggle::ReverbEnable)
        }
        ["3", path] if path.starts_with("reverb") => {
            st.page = 3;
            let field = reverb_field(path)?;
            Some(vec![Message::OscRmeFx {
                echo: false,
                field,
                value: arg_f32(first)?,
            }])
        }
        ["3", "echoEnable"] => {
            st.page = 3;
            toggle(first, RmeToggle::EchoEnable)
        }
        ["3", path] if path.starts_with("echo") => {
            st.page = 3;
            let field = echo_field(path)?;
            Some(vec![Message::OscRmeFx {
                echo: true,
                field,
                value: arg_f32(first)?,
            }])
        }

        // ---- page 3: snapshots, groups, undo ---------------------------
        ["3", "snapshots", n, _] => {
            st.page = 3;
            let n: u8 = n.parse().ok()?;
            if !(1..=8).contains(&n) {
                return None;
            }
            if !arg_on(first)? {
                return Some(Vec::new());
            }
            // RME numbering is reversed inside the path.
            Some(vec![Message::SnapshotClicked(9 - n)])
        }
        ["3", "muteGroups", n, _] => {
            st.page = 3;
            group_toggle(first, n, GroupKind::Mute)
        }
        ["3", "soloGroups", n, _] => {
            st.page = 3;
            group_toggle(first, n, GroupKind::Solo)
        }
        ["3", "faderGroups", n, _] => {
            st.page = 3;
            group_toggle(first, n, GroupKind::Fader)
        }
        ["3", "undo"] => {
            st.page = 3;
            Some(if arg_on(first)? {
                vec![Message::Undo]
            } else {
                Vec::new()
            })
        }
        ["3", "redo"] => {
            st.page = 3;
            Some(if arg_on(first)? {
                vec![Message::Redo]
            } else {
                Vec::new()
            })
        }
        ["3", rec] if rec.starts_with("record") => None,

        // ---- page 4: Room EQ (not present on the UCX/UC) ---------------
        ["4", ..] => None,

        _ => None,
    }
}

#[derive(Clone, Copy)]
enum GroupKind {
    Mute,
    Solo,
    Fader,
}

fn group_toggle(arg: &OscType, n: &str, kind: GroupKind) -> Option<Vec<Message>> {
    let n: usize = n.parse().ok()?;
    if !(1..=4).contains(&n) {
        return None;
    }
    if !arg_on(arg)? {
        return Some(Vec::new());
    }
    let idx = 4 - n;
    let link = match kind {
        GroupKind::Mute => crate::sidebar::GroupLink::Mute,
        GroupKind::Solo => crate::sidebar::GroupLink::Solo,
        GroupKind::Fader => crate::sidebar::GroupLink::Fader,
    };
    Some(vec![Message::GroupLinkToggle(idx, link)])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(name: &str) -> OscType {
        OscType::String(name.into())
    }

    #[test]
    fn set_bank_start_selects_the_absolute_row() {
        let mut st = RmeState::default();
        parse("/setBankStart", &[OscType::Float(20.0)], &mut st).unwrap();
        assert_eq!(st.bus, Bus::Playback);
        assert_eq!(st.channel, 2);
        assert_eq!(st.bank, 2);

        parse("/setBankStart", &[OscType::Float(40.0)], &mut st).unwrap();
        assert_eq!(st.bus, Bus::Output);
        assert_eq!(st.channel, 4);
    }

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
    fn page2_toggle_is_edge_triggered() {
        let mut st = RmeState::default();
        assert!(matches!(
            parse("/2/mute", &[OscType::Float(1.0)], &mut st).as_deref(),
            Some([Message::RmeToggle(RmeToggle::Mute(ChannelId::Input(0)))])
        ));
        assert!(matches!(
            parse("/2/mute", &[OscType::Float(0.0)], &mut st).as_deref(),
            Some([])
        ));
    }

    #[test]
    fn page1_mute_is_direct() {
        let mut st = RmeState::default();
        assert!(matches!(
            parse("/1/mute/1/1", &[OscType::Float(1.0)], &mut st).as_deref(),
            Some([Message::Mute(ChannelId::Input(0), true)])
        ));
    }

    #[test]
    fn phantom_zero_is_a_direct_off() {
        let mut st = RmeState::default();
        assert!(matches!(
            parse("/2/phantom", &[OscType::Float(0.0)], &mut st).as_deref(),
            Some([Message::Phantom(0, false)])
        ));
    }

    #[test]
    fn monitor_controls() {
        let mut st = RmeState::default();
        assert!(matches!(
            parse("/1/mainTalkback", &[OscType::Int(1)], &mut st).as_deref(),
            Some([Message::RmeToggle(RmeToggle::MainTalkback)])
        ));
        assert!(matches!(
            parse("/1/mainMono", &[OscType::Int(1)], &mut st).as_deref(),
            Some([Message::RmeToggle(RmeToggle::MainMono)])
        ));
        assert!(matches!(
            parse("/1/mainDim", &[OscType::Int(1)], &mut st).as_deref(),
            Some([Message::RmeToggle(RmeToggle::MainDim)])
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
    fn snapshots_and_groups_use_reversed_numbering() {
        let mut st = RmeState::default();
        assert!(matches!(
            parse("/3/snapshots/8/1", &[OscType::Float(1.0)], &mut st).as_deref(),
            Some([Message::SnapshotClicked(1)])
        ));
        assert!(matches!(
            parse("/3/snapshots/1/1", &[OscType::Float(1.0)], &mut st).as_deref(),
            Some([Message::SnapshotClicked(8)])
        ));
        assert!(matches!(
            parse("/3/muteGroups/4/1", &[OscType::Float(1.0)], &mut st).as_deref(),
            Some([Message::GroupLinkToggle(0, crate::sidebar::GroupLink::Mute)])
        ));
    }

    #[test]
    fn snapshots_require_a_nonzero_argument() {
        let mut st = RmeState::default();
        assert!(matches!(
            parse("/3/snapshots/8/1", &[OscType::Float(0.0)], &mut st).as_deref(),
            Some([])
        ));
    }

    #[test]
    fn eq_band_types_are_device_specific() {
        let mut st = RmeState::default();
        assert!(matches!(
            parse("/2/eqType1", &[OscType::Float(0.0)], &mut st).as_deref(),
            Some([Message::EqBandType(_, 0, EqBandType::Bell)])
        ));
        assert!(matches!(
            parse("/2/eqType1", &[OscType::Float(1.0)], &mut st).as_deref(),
            Some([Message::EqBandType(_, 0, EqBandType::LowShelf)])
        ));
        assert!(matches!(
            parse("/2/eqType3", &[OscType::Float(0.0)], &mut st).as_deref(),
            Some([Message::EqBandType(_, 2, EqBandType::Bell)])
        ));
        assert!(matches!(
            parse("/2/eqType3", &[OscType::Float(0.5)], &mut st).as_deref(),
            Some([Message::EqBandType(_, 2, EqBandType::HighShelf)])
        ));
        assert!(matches!(
            parse("/2/eqType3", &[OscType::Float(1.0)], &mut st).as_deref(),
            Some([Message::EqBandType(_, 2, EqBandType::LowPass)])
        ));
    }

    #[test]
    fn gain_scale_depends_on_the_input() {
        let mut st = RmeState::default();
        let msgs = parse("/1/micgain1", &[OscType::Float(0.5)], &mut st).unwrap();
        assert!(matches!(msgs[0], Message::Gain(0, 33)));
        parse("/setBankStart", &[OscType::Float(2.0)], &mut st).unwrap();
        let msgs = parse("/2/gain", &[OscType::Float(1.0)], &mut st).unwrap();
        assert!(matches!(msgs[0], Message::Gain(2, 24)));
    }

    #[test]
    fn unsupported_paths_are_rejected() {
        let mut st = RmeState::default();
        assert!(parse("/loadQuickWorkspace", &[OscType::Int(1)], &mut st).is_none());
        assert!(parse("/2/recordEnable", &[OscType::Float(1.0)], &mut st).is_none());
        assert!(parse("/2/playChannel", &[OscType::Float(1.0)], &mut st).is_none());
        assert!(parse("/3/recordRecordStart", &[OscType::Float(1.0)], &mut st).is_none());
        assert!(parse("/4/reqEnable", &[OscType::Float(1.0)], &mut st).is_none());
        assert!(parse("/1/mainExtIn", &[OscType::Float(1.0)], &mut st).is_none());
    }

    #[test]
    fn trackname_renames_the_selected_channel() {
        let mut st = RmeState::default();
        let msgs = parse("/2/trackname", &[input("Vocal")], &mut st).unwrap();
        assert!(matches!(
            msgs.as_slice(),
            [Message::RmeRename(ChannelId::Input(0), name)] if name == "Vocal"
        ));
    }
}
