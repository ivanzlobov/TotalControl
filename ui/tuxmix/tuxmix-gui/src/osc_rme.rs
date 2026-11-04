//! TotalMix FX OSC front-end.
//!
//! Implements the address space in `docs/spec/OSC-CONTROL-SPEC.md` so a
//! controller written for TotalMix FX drives TotalControl unchanged.  This
//! slice covers the none-paged navigation commands, page 1 and the monitor
//! controls.  Page 2-4 follow in later slices.
//!
//! The RME surface is banked: page 1 addresses `bank + n`, page 2 addresses
//! one selected channel, and `/setSubmix` selects the output whose mix is
//! being edited.

use rosc::OscType;

use tuxmix_core::channel::{EqBandType, InputLevel};
use tuxmix_core::ChannelId;

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
            let out = if st.bus == Bus::Output { idx } else { st.submix };
            Some(vec![Message::VolumeChanged(cid, out, gain)])
        }
        ["1", pan] if pan.starts_with("pan") => {
            let n: usize = pan.trim_start_matches("pan").parse().ok()?;
            let idx = banked(st, n)?;
            let cid = channel(st.bus, idx)?;
            let value = pan_value(first)?;
            let out = if st.bus == Bus::Output { idx } else { st.submix };
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
            Some(vec![Message::CueChanged(ChannelId::Output(idx), arg_on(first)?)])
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
            let out = if st.bus == Bus::Output { st.channel } else { st.submix };
            Some(vec![Message::VolumeChanged(cid, out, fader_gain(first)?)])
        }
        ["2", "pan"] => {
            let cid = channel(st.bus, st.channel)?;
            let out = if st.bus == Bus::Output { st.channel } else { st.submix };
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
            Bus::Input => Some(vec![Message::InputMsProcChanged(st.channel, arg_on(first)?)]),
            Bus::Playback => Some(vec![Message::PlaybackMsProcChanged(
                st.channel,
                arg_on(first)?,
            )]),
            Bus::Output => None,
        },
        ["2", "loopback"] => Some(vec![Message::LoopbackChanged(
            st.channel,
            arg_on(first)?,
        )]),
        ["2", "cue"] if st.bus == Bus::Output => Some(vec![Message::CueChanged(
            ChannelId::Output(st.channel),
            arg_on(first)?,
        )]),
        ["2", "talkbackSel"] if st.bus == Bus::Output => Some(vec![
            Message::TalkbackReceiveChanged(st.channel, arg_on(first)?),
        ]),
        ["2", "noTrim"] if st.bus == Bus::Output => Some(vec![Message::NoTrimChanged(
            st.channel,
            arg_on(first)?,
        )]),
        ["2", "width"] => {
            let cid = channel(st.bus, st.channel)?;
            Some(vec![Message::ChannelWidthChanged(cid, arg_f32(first)?.clamp(0.0, 1.0))])
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
            Some(vec![Message::EqLowCutFreq(cid, freq_hz(arg_f32(first)?, 20.0, 500.0))])
        }
        ["2", "lowcutGrade"] => {
            let cid = channel(st.bus, st.channel)?;
            let p = arg_f32(first)?.clamp(0.0, 1.0);
            Some(vec![Message::EqLowCutSlope(cid, 6 + (p * 3.0).round() as u8 * 6)])
        }
        ["2", "eqEnable"] => {
            let cid = channel(st.bus, st.channel)?;
            Some(vec![Message::EqEnabled(cid, arg_on(first)?)])
        }
        ["2", path] if path.starts_with("eqType") => {
            let band: usize = path.trim_start_matches("eqType").parse().ok()?;
            let cid = channel(st.bus, st.channel)?;
            Some(vec![Message::EqBandType(cid, band, eq_type(first)?)])
        }
        ["2", path] if path.starts_with("eqGain") => {
            let band: usize = path.trim_start_matches("eqGain").parse().ok()?;
            let cid = channel(st.bus, st.channel)?;
            let db = arg_f32(first)? * 48.0 - 24.0;
            Some(vec![Message::EqBandGain(cid, band, db.clamp(-24.0, 24.0))])
        }
        ["2", path] if path.starts_with("eqFreq") => {
            let band: usize = path.trim_start_matches("eqFreq").parse().ok()?;
            let cid = channel(st.bus, st.channel)?;
            Some(vec![Message::EqBandFreq(cid, band, freq_hz(arg_f32(first)?, 20.0, 20_000.0))])
        }
        ["2", path] if path.starts_with("eqQ") => {
            let band: usize = path.trim_start_matches("eqQ").parse().ok()?;
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
            (1..=8).contains(&n).then_some(vec![Message::SnapshotClicked(n)])
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
}
