//! First-generation Babyface through the kernel hwdep transport.
//!
//! The unit has no state readback for its mixer, so the host owns the mix:
//! the channel model (faders, pans, mutes, solos, output masters) is kept in
//! memory and every change is realised as the set of `0x12` crosspoint and
//! output-master writes that differ from what was last sent.  The register
//! map is in `docs/spec/BABYFACE-PROTOCOL-SPEC.md`.
//!
//! - crosspoint: `addr = 24 * out + src`; `src` 0..9 are the hardware
//!   inputs, 10..21 the software playback channels;
//! - output master: `addr = 0x3e0 + out`;
//! - value: an 18-bit linear level (`0x8000` = 0 dB), sent as
//!   `wValue = level >> 2`, `wIndex = (level & 3) << 14 | addr`;
//! - mic gain: `0x1a`, `wIndex` 0/1, `wValue` 0..18 = 6..60 dB in 3 dB
//!   steps;
//! - +48 V: `0x17`, `wValue` = bits, `wIndex` = mask; bit 0 = Mic 1,
//!   bit 1 = Mic 2.
//!
//! Nothing is written at open.  The first full write (matrix, masters, mic
//! gain and +48 V) happens when the saved scene is applied, or at
//! `begin_metering` when there is none, so the unit never passes through a
//! default mix.  Controls the backend does not drive return an error.
//!
//! Meters come from the DSP read-back on IF1 bulk IN `0x85`, which the
//! driver delivers as `EV_METER` events.  A packet is 456 bytes: 38
//! little-endian `u64` mean-square values, then 38 `u32` peak values.
//! Slot order is inputs 0..9, playback 0..13, outputs 0..13 (the last two
//! outputs are the FX send bus).  Peak full scale is `2^27`, power full
//! scale `2^55`, and 31 marks a channel below range.

use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};
use std::collections::HashMap;

use crate::capabilities::DeviceCapabilities;
use crate::channel::{
    ChannelId, ChannelType, Dynamics, Echo, EqBandType, InputChannel, InputEq, InputLevel,
    OutputChannel, PlaybackChannel, Reverb, Sensitivity,
};
use crate::device::{DeviceSettings, RmeDevice};
use crate::error::Error;
use crate::fireface::abi::{EV_METER, MODEL_BABYFACE};
use crate::fireface::hwdep::{Direction, Hwdep};
use crate::mock::MockBabyfacePro;
use crate::profiles::babyface::PROFILE;
use crate::scene::Scene;

const MODEL: &str = "Babyface";
const REQ_MIXER: u8 = 0x12;
const REQ_PGA: u8 = 0x1a;
/// Settings register: `wValue` = new bits, `wIndex` = mask.
const REQ_SETTINGS: u8 = 0x17;
/// +48 V on Mic 1 / Mic 2: bits 0 / 1 of the settings register.
const PHANTOM_BIT: [u16; 2] = [0x0001, 0x0002];
const STRIDE: u16 = 24;
const PLAYBACK_BASE: u16 = 10;
const MASTER_BASE: u16 = 0x3e0;
const OUTPUTS: usize = 12;
const INPUTS: usize = 10;
const UNITY: f32 = 32768.0;
/// Mic gain: `dB = MIC_GAIN_MIN_DB + 3 * raw`, raw 0..18.
const MIC_GAIN_MIN_DB: u32 = 6;
const MIC_GAIN_MAX_DB: u32 = 60;
const MIC_GAIN_STEP_DB: u32 = 3;
/// How often the stream rate is re-read from the driver.
const RATE_POLL: Duration = Duration::from_millis(500);

fn unsupported(what: &str) -> Error {
    Error::InvalidChannel(format!("{what} is not supported on the Babyface"))
}

fn capabilities() -> DeviceCapabilities {
    DeviceCapabilities {
        protocol_features: false,
        eq: false,
        low_cut: false,
        loopback: false,
        no_trim: false,
        talkback: false,
        phase: false,
        phase_without_eq: false,
        output_phase: false,
        output_balance: false,
        stereo_split: false,
        matrix_cells: false,
        class_compliant: false,
        input_phantom: true,
        input_pad: false,
        input_level: false,
        input_autoset: false,
        dynamics: false,
        fx: false,
        pitch: false,
        width: false,
        channel_width: false,
        ms_proc: false,
        an12: false,
        input_link: false,
        eq_for_record: false,
        optical_out: false,
    }
}

/// Linear pan law shared with the Fireface UC backend.
fn pan_split(volume: f32, pan: i8) -> (f32, f32) {
    let v = volume.clamp(0.0, 2.0);
    let p = pan.clamp(-100, 100) as f32 / 100.0;
    let l = v * if p > 0.0 { 1.0 - p } else { 1.0 };
    let r = v * if p < 0.0 { 1.0 + p } else { 1.0 };
    (l, r)
}

/// Linear gain -> 18-bit level.
fn level18(gain: f32) -> u32 {
    (gain.clamp(0.0, 2.0) * UNITY).round() as u32
}

fn mic_gain_raw(db: u32) -> u16 {
    ((db.clamp(MIC_GAIN_MIN_DB, MIC_GAIN_MAX_DB) - MIC_GAIN_MIN_DB + 1) / MIC_GAIN_STEP_DB) as u16
}

/// The gain the unit actually applies for a requested value.
fn mic_gain_snap(db: u32) -> u32 {
    MIC_GAIN_MIN_DB + MIC_GAIN_STEP_DB * mic_gain_raw(db) as u32
}

const OUT_NAMES: [&str; 12] = [
    "AN1", "AN2", "PH3", "PH4", "AS1", "AS2", "ADAT3", "ADAT4", "ADAT5", "ADAT6", "ADAT7",
    "ADAT8",
];

/// Human-readable form of one `0x12` write, for debug logging.
fn describe(addr: u16, level: u32) -> String {
    let db = if level == 0 {
        "-inf".to_string()
    } else {
        format!("{:+.1} dB", 20.0 * (level as f32 / UNITY).log10())
    };
    if addr >= MASTER_BASE {
        let out = (addr - MASTER_BASE) as usize;
        return format!("master {} = {db}", OUT_NAMES.get(out).unwrap_or(&"?"));
    }
    let out = (addr / STRIDE) as usize;
    let src = addr % STRIDE;
    let src = if src < PLAYBACK_BASE {
        format!("in{}", src + 1)
    } else {
        format!("pb{}", src - PLAYBACK_BASE + 1)
    };
    format!("{src} -> {} = {db}", OUT_NAMES.get(out).unwrap_or(&"?"))
}

/// The first-generation Babyface (vendor mode) over `snd-fireface-uc`.
pub struct BabyfaceGen1 {
    state: MockBabyfacePro,
    hw: Hwdep,
    /// Last level sent per `0x12` address.
    sent: RefCell<HashMap<u16, u32>>,
    /// Last raw mic gain sent per input.
    sent_gain: RefCell<[Option<u16>; 2]>,
    /// Last +48 V state sent per input.
    sent_phantom: RefCell<[Option<bool>; 2]>,
    /// Set by the first scene apply or `begin_metering`; nothing is
    /// written before that.
    armed: Cell<bool>,
    last_rate_poll: Cell<Option<Instant>>,
    /// Peaks from the DSP meter packets, held until read.
    meters: RefCell<DspMeters>,
}

/// Meter slots in one `0x85` packet.
const METER_SLOTS: usize = 38;
const METER_PACKET: usize = METER_SLOTS * 12;
const METER_PLAYBACK_BASE: usize = 10;
const METER_OUTPUT_BASE: usize = 24;
const METER_PLAYBACKS: usize = 14;
/// Peak full scale (`2^27`).
const METER_FULL_SCALE: f32 = 134_217_728.0;

/// Peak per meter slot since the last read, as a fraction of full scale.
struct DspMeters {
    held: [Option<f32>; METER_SLOTS],
    last_packet: Option<Instant>,
}

impl Default for DspMeters {
    fn default() -> Self {
        Self {
            held: [None; METER_SLOTS],
            last_packet: None,
        }
    }
}

impl DspMeters {
    fn apply(&mut self, data: &[u8]) {
        if data.len() < METER_PACKET {
            return;
        }
        let peaks = &data[METER_SLOTS * 8..METER_PACKET];
        for (slot, word) in self.held.iter_mut().zip(peaks.chunks_exact(4)) {
            let raw = u32::from_le_bytes(word.try_into().unwrap());
            let level = (raw as f32 / METER_FULL_SCALE).min(1.0);
            *slot = Some(slot.map_or(level, |held| held.max(level)));
        }
        self.last_packet = Some(Instant::now());
    }

    /// Takes the held peaks of `count` slots starting at `base`.
    fn take(&mut self, base: usize, count: usize) -> Vec<f32> {
        self.held[base..base + count]
            .iter_mut()
            .map(|slot| slot.take().unwrap_or(0.0))
            .collect()
    }
}

impl BabyfaceGen1 {
    /// Whether the kernel driver exposes a Babyface right now.
    pub fn present() -> bool {
        (0..32).any(|card| {
            Hwdep::open(card)
                .and_then(|hw| hw.info())
                .is_ok_and(|i| i.model == MODEL_BABYFACE)
        })
    }

    /// Open the first Babyface the kernel driver exposes.
    pub fn open_first() -> Result<Self, Error> {
        for card in 0..32 {
            let Ok(hw) = Hwdep::open(card) else {
                continue;
            };
            if hw.info().is_ok_and(|i| i.model == MODEL_BABYFACE) {
                return Self::with_hwdep(hw);
            }
        }
        Err(Error::DeviceNotFound {
            model: MODEL.into(),
        })
    }

    fn with_hwdep(hw: Hwdep) -> Result<Self, Error> {
        let mut state = MockBabyfacePro::open_with(&PROFILE, MODEL, capabilities())?;
        for ch in state.inputs_mut() {
            ch.phantom = false;
            ch.pad = false;
            ch.has_instrument = false;
            ch.eq = None;
            ch.sensitivity = None;
            ch.volumes.iter_mut().for_each(|v| *v = 0.0);
            ch.pans.iter_mut().for_each(|p| *p = 0);
            if ch.channel_type == ChannelType::Mic {
                ch.gain_max = Some(MIC_GAIN_MAX_DB);
                ch.gain = Some(MIC_GAIN_MIN_DB);
            } else {
                ch.gain_max = None;
                ch.gain = None;
            }
        }
        // Playback pair n -> output pair n at 0 dB, as TotalMix sets it up.
        for pb in state.playbacks_mut() {
            let pair = pb.id / 2;
            for (out, v) in pb.volumes.iter_mut().enumerate() {
                *v = if out == pair { 1.0 } else { 0.0 };
            }
            pb.pans.iter_mut().for_each(|p| *p = 0);
        }
        for out in state.outputs_mut() {
            out.volume = 1.0;
        }
        let mut dev = Self {
            state,
            meters: RefCell::new(DspMeters::default()),
            hw,
            sent: RefCell::new(HashMap::new()),
            sent_gain: RefCell::new([None; 2]),
            sent_phantom: RefCell::new([None; 2]),
            armed: Cell::new(false),
            last_rate_poll: Cell::new(None),
        };
        dev.refresh_rate();
        Ok(dev)
    }

    /// The stream rate follows the host; show what the driver runs at.
    fn refresh_rate(&mut self) {
        self.last_rate_poll.set(Some(Instant::now()));
        if let Ok(info) = self.hw.info() {
            if info.rate > 0 {
                self.state.settings_mut().sample_rate = info.rate;
            }
        }
    }

    /// Allow device writes and send the whole model.
    fn arm(&self) {
        self.armed.set(true);
        self.sync();
    }

    fn vendor_out(&self, request: u8, value: u16, index: u16) -> std::io::Result<()> {
        self.hw
            .vendor(request, Direction::Out, value, index, &[])
            .map(|_| ())
    }

    /// The level every `0x12` address should hold for the current model.
    fn desired(&self) -> Vec<(u16, u32)> {
        let st = &self.state;
        let inputs = st.inputs();
        let playbacks = st.playbacks();
        let outputs = st.outputs();
        let solo = inputs.iter().any(|c| c.solo) || playbacks.iter().any(|c| c.solo);
        let mut cells = Vec::with_capacity(OUTPUTS * 22 + OUTPUTS);

        for out in 0..OUTPUTS {
            let pair = out / 2;
            let side = out % 2;
            for (i, ch) in inputs.iter().enumerate() {
                let audible = !ch.mute && (!solo || ch.solo);
                let vol = ch.volumes.get(pair).copied().unwrap_or(0.0);
                let pan = ch.pans.get(pair).copied().unwrap_or(0);
                let gain = if !audible {
                    0.0
                } else if ch.channel_type == ChannelType::Mic {
                    let (l, r) = pan_split(vol, pan);
                    if side == 0 { l } else { r }
                } else if i % 2 == side {
                    // Optical inputs are stereo pairs: each side feeds its
                    // own output side, the pan acts as balance.
                    let base_pan = inputs[i & !1].pans.get(pair).copied().unwrap_or(0);
                    let (l, r) = pan_split(vol, base_pan);
                    if side == 0 { l } else { r }
                } else {
                    0.0
                };
                cells.push((out as u16 * STRIDE + i as u16, level18(gain)));
            }
            for ch in playbacks {
                let audible = !ch.mute && (!solo || ch.solo);
                let vol = ch.volumes.get(pair).copied().unwrap_or(0.0);
                let pan = ch.pans.get(pair).copied().unwrap_or(0);
                let (l, r) = pan_split(vol, pan);
                // A playback channel feeds its own side of each pair; the
                // pan acts as balance, as on the Fireface UC.
                let gain = if !audible || ch.id % 2 != side {
                    0.0
                } else if side == 0 {
                    l
                } else {
                    r
                };
                let src = PLAYBACK_BASE + ch.id as u16;
                cells.push((out as u16 * STRIDE + src, level18(gain)));
            }
            let master = outputs
                .get(out)
                .map(|o| if o.mute { 0.0 } else { o.volume })
                .unwrap_or(0.0);
            cells.push((MASTER_BASE + out as u16, level18(master)));
        }
        cells
    }

    /// Send every crosspoint, master, preamp gain and +48 V that changed.
    fn sync(&self) {
        if !self.armed.get() {
            return;
        }
        let mut sent = self.sent.borrow_mut();
        for (addr, level) in self.desired() {
            if sent.get(&addr) == Some(&level) {
                continue;
            }
            let value = (level >> 2) as u16;
            let index = (((level & 3) << 14) as u16) | addr;
            log::debug!("{}", describe(addr, level));
            match self.vendor_out(REQ_MIXER, value, index) {
                Ok(()) => {
                    sent.insert(addr, level);
                }
                Err(e) => {
                    // Retried on the next sync; the preamp writes below
                    // are independent of the matrix.
                    log::warn!("Babyface: 0x12 {addr:#05x} failed: {e}");
                    break;
                }
            }
        }
        drop(sent);

        let mut sent_gain = self.sent_gain.borrow_mut();
        for (idx, ch) in self.state.inputs().iter().take(2).enumerate() {
            let Some(db) = ch.gain else { continue };
            let raw = mic_gain_raw(db);
            if sent_gain[idx] == Some(raw) {
                continue;
            }
            match self.vendor_out(REQ_PGA, raw, idx as u16) {
                Ok(()) => sent_gain[idx] = Some(raw),
                Err(e) => log::warn!("Babyface: mic gain {idx} failed: {e}"),
            }
        }
        drop(sent_gain);

        let mut sent_phantom = self.sent_phantom.borrow_mut();
        for (idx, ch) in self.state.inputs().iter().take(2).enumerate() {
            if sent_phantom[idx] == Some(ch.phantom) {
                continue;
            }
            let bit = PHANTOM_BIT[idx];
            match self.vendor_out(REQ_SETTINGS, if ch.phantom { bit } else { 0 }, bit) {
                Ok(()) => {
                    log::info!(
                        "Babyface: 48V on Mic {} {}",
                        idx + 1,
                        if ch.phantom { "on" } else { "off" }
                    );
                    sent_phantom[idx] = Some(ch.phantom);
                }
                Err(e) => log::warn!("Babyface: 48V {idx} failed: {e}"),
            }
        }
    }
}

impl BabyfaceGen1 {
    /// Whether DSP meter packets are arriving.
    pub fn meters_available(&self) -> bool {
        self.pump_meters();
        self.meters
            .borrow()
            .last_packet
            .is_some_and(|t| t.elapsed() < Duration::from_secs(1))
    }

    /// Software playback peaks, one per playback channel.
    pub fn playback_meters(&self) -> Vec<f32> {
        self.pump_meters();
        let n = self.state.playbacks().len().min(METER_PLAYBACKS);
        self.meters.borrow_mut().take(METER_PLAYBACK_BASE, n)
    }

    /// Hardware output peaks, one per output channel (FX bus excluded).
    pub fn output_meters(&self) -> Vec<f32> {
        self.pump_meters();
        let n = self.state.outputs().len().min(OUTPUTS);
        self.meters.borrow_mut().take(METER_OUTPUT_BASE, n)
    }

    /// Reads every pending driver event and folds meter packets in.
    fn pump_meters(&self) {
        let mut meters = self.meters.borrow_mut();
        for _ in 0..512 {
            match self.hw.wait(Duration::ZERO) {
                Ok(true) => match self.hw.read_event() {
                    Ok(e) if e.kind == EV_METER => meters.apply(&e.data),
                    Ok(_) => {}
                    Err(_) => break,
                },
                _ => break,
            }
        }
    }
}

impl RmeDevice for BabyfaceGen1 {
    fn model_name(&self) -> &str {
        MODEL
    }

    fn canonical_model(&self) -> &str {
        MODEL
    }

    fn is_connected(&self) -> bool {
        self.hw.is_connected()
    }

    fn open() -> Result<Self, Error> {
        Self::open_first()
    }

    fn begin_metering(&mut self) {
        self.arm();
    }

    fn meters(&self) -> Option<Vec<f32>> {
        self.pump_meters();
        let n = self.state.inputs().len().min(INPUTS);
        Some(self.meters.borrow_mut().take(0, n))
    }

    fn poll_events(&mut self) -> Result<(), Error> {
        if self
            .last_rate_poll
            .get()
            .is_none_or(|t| t.elapsed() >= RATE_POLL)
        {
            self.refresh_rate();
        }
        // Picks up edits made through the `*_mut` accessors.
        self.sync();
        Ok(())
    }

    fn set_phantom(&mut self, idx: usize, on: bool) -> Result<(), Error> {
        if idx >= PHANTOM_BIT.len() {
            return Err(Error::InvalidChannel(format!("Input {idx} has no 48V")));
        }
        self.state.set_phantom(idx, on)?;
        self.sync();
        Ok(())
    }

    fn set_input_phantom(&mut self, idx: usize, on: bool) -> Result<(), Error> {
        self.set_phantom(idx, on)
    }

    fn set_pad(&mut self, _idx: usize, _on: bool) -> Result<(), Error> {
        Err(Error::InvalidChannel("the Babyface has no pad".into()))
    }

    fn set_input_pad(&mut self, idx: usize, on: bool) -> Result<(), Error> {
        self.set_pad(idx, on)
    }

    fn inputs_mut(&mut self) -> &mut [InputChannel] {
        self.state.inputs_mut()
    }

    fn playbacks_mut(&mut self) -> &mut [PlaybackChannel] {
        self.state.playbacks_mut()
    }

    fn outputs_mut(&mut self) -> &mut [OutputChannel] {
        self.state.outputs_mut()
    }

    fn settings_mut(&mut self) -> &mut DeviceSettings {
        self.state.settings_mut()
    }

    fn sample_rates(&self) -> &'static [u32] {
        self.state.sample_rates()
    }

    fn protocol_features(&self) -> bool {
        self.state.protocol_features()
    }

    fn loopback_supported(&self) -> bool {
        self.state.loopback_supported()
    }

    fn low_cut_supported(&self) -> bool {
        self.state.low_cut_supported()
    }

    fn phase_supported(&self) -> bool {
        self.state.phase_supported()
    }

    fn output_balance_supported(&self) -> bool {
        self.state.output_balance_supported()
    }

    fn capabilities(&self) -> DeviceCapabilities {
        self.state.capabilities()
    }

    fn stereo_split_supported(&self) -> bool {
        self.state.stereo_split_supported()
    }

    fn class_compliant(&self) -> bool {
        // Vendor mode; only the RME DSP feature set is limited.
        false
    }

    fn output_pair_count(&self) -> usize {
        self.state.output_pair_count()
    }

    fn output_display_order(&self) -> Vec<usize> {
        self.state.output_display_order()
    }

    fn matrix_uses_display_order(&self) -> bool {
        self.state.matrix_uses_display_order()
    }

    fn inputs(&self) -> &[InputChannel] {
        self.state.inputs()
    }

    fn playbacks(&self) -> &[PlaybackChannel] {
        self.state.playbacks()
    }

    fn outputs(&self) -> &[OutputChannel] {
        self.state.outputs()
    }

    fn settings(&self) -> &DeviceSettings {
        self.state.settings()
    }

    fn set_volume(&mut self, channel: ChannelId, output: usize, volume: f32) -> Result<(), Error> {
        let r = self.state.set_volume(channel, output, volume);
        self.sync();
        r
    }

    fn volume(&self, channel: ChannelId, output: usize) -> Result<f32, Error> {
        self.state.volume(channel, output)
    }

    fn matrix_cells_supported(&self) -> bool {
        self.state.matrix_cells_supported()
    }

    fn matrix_cell(&self, channel: ChannelId, block: usize) -> Option<f32> {
        self.state.matrix_cell(channel, block)
    }

    fn set_matrix_cell(&mut self, _channel: ChannelId, _block: usize, _gain: f32) -> Result<(), Error> {
        Err(unsupported("set_matrix_cell"))
    }

    fn set_pan(&mut self, channel: ChannelId, output: usize, pan: i8) -> Result<(), Error> {
        let r = self.state.set_pan(channel, output, pan);
        self.sync();
        r
    }

    fn pan(&self, channel: ChannelId, output: usize) -> Result<i8, Error> {
        self.state.pan(channel, output)
    }

    fn set_mute(&mut self, channel: ChannelId, mute: bool) -> Result<(), Error> {
        let r = self.state.set_mute(channel, mute);
        self.sync();
        r
    }

    fn mute(&self, channel: ChannelId) -> Result<bool, Error> {
        self.state.mute(channel)
    }

    fn set_solo(&mut self, channel: ChannelId, solo: bool) -> Result<(), Error> {
        let r = self.state.set_solo(channel, solo);
        self.sync();
        r
    }

    fn solo(&self, channel: ChannelId) -> Result<bool, Error> {
        self.state.solo(channel)
    }

    fn set_instrument(&mut self, _idx: usize, _on: bool) -> Result<(), Error> {
        Err(unsupported("set_instrument"))
    }

    fn set_input_instrument(&mut self, _idx: usize, _on: bool) -> Result<(), Error> {
        Err(unsupported("set_input_instrument"))
    }

    fn set_gain(&mut self, idx: usize, gain: u32) -> Result<(), Error> {
        let r = self.state.set_gain(idx, mic_gain_snap(gain));
        self.sync();
        r
    }

    fn set_input_gain(&mut self, idx: usize, gain: u32) -> Result<(), Error> {
        let r = self.state.set_input_gain(idx, mic_gain_snap(gain));
        self.sync();
        r
    }

    fn set_input_level(&mut self, _idx: usize, _level: InputLevel) -> Result<(), Error> {
        Err(unsupported("set_input_level"))
    }

    fn set_autoset(&mut self, _idx: usize, _on: bool) -> Result<(), Error> {
        Err(unsupported("set_autoset"))
    }

    fn set_dynamics(&mut self, _channel: ChannelId, _dynamics: &Dynamics) -> Result<(), Error> {
        Err(unsupported("set_dynamics"))
    }

    fn set_reverb(&mut self, _reverb: &Reverb) -> Result<(), Error> {
        Err(unsupported("set_reverb"))
    }

    fn set_echo(&mut self, _echo: &Echo) -> Result<(), Error> {
        Err(unsupported("set_echo"))
    }

    fn set_fx_return(&mut self, _output: usize, _db: f32) -> Result<(), Error> {
        Err(unsupported("set_fx_return"))
    }

    fn set_pitch(&mut self, _pitch_percent: f32) -> Result<(), Error> {
        Err(unsupported("set_pitch"))
    }

    fn set_sample_rate(&mut self, rate: u32) -> Result<(), Error> {
        // The rate follows the host stream, as on the UC; nothing is
        // written.  `poll_events` re-reads the running rate.
        self.state.set_sample_rate(rate)
    }

    fn set_loopback(&mut self, _out: usize, _on: bool) -> Result<(), Error> {
        Err(unsupported("set_loopback"))
    }

    fn set_loopback_channel(&mut self, _channel: usize, _on: bool) -> Result<(), Error> {
        Err(unsupported("set_loopback_channel"))
    }

    fn set_ms_proc(&mut self, _on: bool) -> Result<(), Error> {
        Err(unsupported("set_ms_proc"))
    }

    fn set_an12(&mut self, _on: bool) -> Result<(), Error> {
        Err(unsupported("set_an12"))
    }

    fn set_dim(&mut self, _on: bool) -> Result<(), Error> {
        Err(unsupported("set_dim"))
    }

    fn set_dim_with(&mut self, _on: bool, _amount_db: f32) -> Result<(), Error> {
        Err(unsupported("set_dim_with"))
    }

    fn set_main_recall(&mut self, _db: f32) -> Result<(), Error> {
        Err(unsupported("set_main_recall"))
    }

    fn set_main_out(&mut self, _pair: Option<usize>) -> Result<(), Error> {
        Err(unsupported("set_main_out"))
    }

    fn set_main_out_b(&mut self, _pair: Option<usize>) -> Result<(), Error> {
        Err(unsupported("set_main_out_b"))
    }

    fn set_mute_fx(&mut self, _on: bool) -> Result<(), Error> {
        Err(unsupported("set_mute_fx"))
    }

    fn set_external_input(&mut self, _on: bool, _source: Option<usize>, _gain_db: f32) -> Result<(), Error> {
        Err(unsupported("set_external_input"))
    }

    fn set_cue_target(&mut self, _pair: Option<usize>) -> Result<(), Error> {
        Err(unsupported("set_cue_target"))
    }

    fn set_pfl(&mut self, _channel: ChannelId, _on: bool) -> Result<(), Error> {
        Err(unsupported("set_pfl"))
    }

    fn set_pfl_mode(&mut self, _pfl: bool, _exclusive: bool) -> Result<(), Error> {
        Err(unsupported("set_pfl_mode"))
    }

    fn set_speaker_b(&mut self, _on: bool) -> Result<(), Error> {
        Err(unsupported("set_speaker_b"))
    }

    fn set_mono(&mut self, _on: bool) -> Result<(), Error> {
        Err(unsupported("set_mono"))
    }

    fn set_speaker_b_linked(&mut self, _on: bool) -> Result<(), Error> {
        Err(unsupported("set_speaker_b_linked"))
    }

    fn set_eq_for_record(&mut self, _on: bool) -> Result<(), Error> {
        Err(unsupported("set_eq_for_record"))
    }

    fn set_optical_out_format(&mut self, _spdif: bool) -> Result<(), Error> {
        Err(unsupported("set_optical_out_format"))
    }

    fn set_cue(&mut self, _idx: usize, _on: bool) -> Result<(), Error> {
        Err(unsupported("set_cue"))
    }

    fn set_input_link(&mut self, linked: bool) -> Result<(), Error> {
        let r = self.state.set_input_link(linked);
        self.sync();
        r
    }

    fn output_linked(&self, pair: usize) -> bool {
        self.state.output_linked(pair)
    }

    fn set_output_linked(&mut self, pair: usize, linked: bool) -> Result<(), Error> {
        let r = self.state.set_output_linked(pair, linked);
        self.sync();
        r
    }

    fn set_output_volume(&mut self, pair: usize, which: usize, volume: f32) -> Result<(), Error> {
        let r = self.state.set_output_volume(pair, which, volume);
        self.sync();
        r
    }

    fn set_output_mute(&mut self, pair: usize, which: usize, mute: bool) -> Result<(), Error> {
        let r = self.state.set_output_mute(pair, which, mute);
        self.sync();
        r
    }

    fn set_output_solo(&mut self, _pair: usize, _which: usize, _solo: bool) -> Result<(), Error> {
        Err(unsupported("set_output_solo"))
    }

    fn input_pair_linked(&self, pair: usize) -> bool {
        self.state.input_pair_linked(pair)
    }

    fn apply_input_eq(&mut self, _idx: usize, _eq: &InputEq) -> Result<(), Error> {
        Err(unsupported("apply_input_eq"))
    }

    fn set_input_pair_linked(&mut self, pair: usize, linked: bool) -> Result<(), Error> {
        let r = self.state.set_input_pair_linked(pair, linked);
        self.sync();
        r
    }

    fn set_input_volume(&mut self, pair: usize, which: usize, output: usize, volume: f32) -> Result<(), Error> {
        let r = self.state.set_input_volume(pair, which, output, volume);
        self.sync();
        r
    }

    fn set_input_balance(&mut self, pair: usize, output: usize, balance: i8) -> Result<(), Error> {
        let r = self.state.set_input_balance(pair, output, balance);
        self.sync();
        r
    }

    fn set_input_mute(&mut self, pair: usize, which: usize, mute: bool) -> Result<(), Error> {
        let r = self.state.set_input_mute(pair, which, mute);
        self.sync();
        r
    }

    fn set_input_solo(&mut self, pair: usize, which: usize, solo: bool) -> Result<(), Error> {
        let r = self.state.set_input_solo(pair, which, solo);
        self.sync();
        r
    }

    fn playback_linked(&self, pair: usize) -> bool {
        self.state.playback_linked(pair)
    }

    fn set_playback_linked(&mut self, pair: usize, linked: bool) -> Result<(), Error> {
        let r = self.state.set_playback_linked(pair, linked);
        self.sync();
        r
    }

    fn set_playback_volume(&mut self, pair: usize, which: usize, output: usize, volume: f32) -> Result<(), Error> {
        let r = self.state.set_playback_volume(pair, which, output, volume);
        self.sync();
        r
    }

    fn set_playback_balance(&mut self, pair: usize, output: usize, balance: i8) -> Result<(), Error> {
        let r = self.state.set_playback_balance(pair, output, balance);
        self.sync();
        r
    }

    fn set_playback_mute(&mut self, pair: usize, which: usize, mute: bool) -> Result<(), Error> {
        let r = self.state.set_playback_mute(pair, which, mute);
        self.sync();
        r
    }

    fn set_playback_solo(&mut self, pair: usize, which: usize, solo: bool) -> Result<(), Error> {
        let r = self.state.set_playback_solo(pair, which, solo);
        self.sync();
        r
    }

    fn set_trim(&mut self, _channel: ChannelId, _db: f32) -> Result<(), Error> {
        Err(unsupported("set_trim"))
    }

    fn set_phase(&mut self, _idx: usize, _invert: bool) -> Result<(), Error> {
        Err(unsupported("set_phase"))
    }

    fn set_input_phase_right(&mut self, _idx: usize, _invert: bool) -> Result<(), Error> {
        Err(unsupported("set_input_phase_right"))
    }

    fn set_input_gain_right(&mut self, _idx: usize, _gain: u32) -> Result<(), Error> {
        Err(unsupported("set_input_gain_right"))
    }

    fn set_output_phase(&mut self, _idx: usize, _invert: bool) -> Result<(), Error> {
        Err(unsupported("set_output_phase"))
    }

    fn set_no_trim(&mut self, _idx: usize, _on: bool) -> Result<(), Error> {
        Err(unsupported("set_no_trim"))
    }

    fn set_fx_send(&mut self, _db: f32) -> Result<(), Error> {
        Err(unsupported("set_fx_send"))
    }

    fn set_channel_fx_send(&mut self, _channel: ChannelId, _db: f32) -> Result<(), Error> {
        Err(unsupported("set_channel_fx_send"))
    }

    fn set_stereo_split(&mut self, _pb: usize, _split: bool) -> Result<(), Error> {
        Err(unsupported("set_stereo_split"))
    }

    fn set_width(&mut self, _width: f32) -> Result<(), Error> {
        Err(unsupported("set_width"))
    }

    fn set_channel_width(&mut self, _channel: ChannelId, _width: f32) -> Result<(), Error> {
        Err(unsupported("set_channel_width"))
    }

    fn set_ref_level(&mut self, _idx: usize, _raw: u16) -> Result<(), Error> {
        Err(unsupported("set_ref_level"))
    }

    fn set_eq_enabled(&mut self, _idx: usize, _on: bool) -> Result<(), Error> {
        Err(unsupported("set_eq_enabled"))
    }

    fn set_eq_band_type(&mut self, _idx: usize, _band: usize, _band_type: EqBandType) -> Result<(), Error> {
        Err(unsupported("set_eq_band_type"))
    }

    fn set_eq_band_freq(&mut self, _idx: usize, _band: usize, _freq_hz: u16) -> Result<(), Error> {
        Err(unsupported("set_eq_band_freq"))
    }

    fn set_eq_band_q(&mut self, _idx: usize, _band: usize, _q: f32) -> Result<(), Error> {
        Err(unsupported("set_eq_band_q"))
    }

    fn set_eq_band_gain(&mut self, _idx: usize, _band: usize, _gain_db: f32) -> Result<(), Error> {
        Err(unsupported("set_eq_band_gain"))
    }

    fn set_eq_low_cut_enabled(&mut self, _idx: usize, _on: bool) -> Result<(), Error> {
        Err(unsupported("set_eq_low_cut_enabled"))
    }

    fn set_eq_low_cut_freq(&mut self, _idx: usize, _freq_hz: u16) -> Result<(), Error> {
        Err(unsupported("set_eq_low_cut_freq"))
    }

    fn set_eq_low_cut_slope(&mut self, _idx: usize, _slope_db_oct: u8) -> Result<(), Error> {
        Err(unsupported("set_eq_low_cut_slope"))
    }

    fn set_output_eq(&mut self, _idx: usize, _eq: &InputEq) -> Result<(), Error> {
        Err(unsupported("set_output_eq"))
    }

    fn set_sensitivity(&mut self, _idx: usize, _sensitivity: Sensitivity) -> Result<(), Error> {
        Err(unsupported("set_sensitivity"))
    }

    fn set_clock_source(&mut self, _source: &str) -> Result<(), Error> {
        Err(unsupported("set_clock_source"))
    }

    fn capture_scene(&self) -> Scene {
        self.state.capture_scene()
    }

    fn apply_scene(&mut self, scene: &Scene) -> Result<(), Error> {
        let r = self.state.apply_scene(scene);
        for ch in self.state.inputs_mut() {
            ch.gain = ch.gain.map(mic_gain_snap);
        }
        self.refresh_rate();
        self.arm();
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_encoding_matches_the_unit() {
        assert_eq!(level18(1.0), 0x8000);
        assert_eq!(level18(2.0), 0x10000);
        assert_eq!(level18(0.0), 0);
    }

    #[test]
    fn mic_gain_steps_are_3_db_from_6() {
        assert_eq!(mic_gain_raw(6), 0);
        assert_eq!(mic_gain_raw(9), 1);
        assert_eq!(mic_gain_raw(39), 11);
        assert_eq!(mic_gain_raw(60), 18);
        assert_eq!(mic_gain_raw(0), 0);
    }

    #[test]
    fn mic_gain_snaps_to_the_unit_grid() {
        assert_eq!(mic_gain_snap(6), 6);
        assert_eq!(mic_gain_snap(7), 6);
        assert_eq!(mic_gain_snap(8), 9);
        assert_eq!(mic_gain_snap(40), 39);
        assert_eq!(mic_gain_snap(0), 6);
        assert_eq!(mic_gain_snap(99), 60);
    }

    fn meter_packet(peaks: &[(usize, u32)]) -> Vec<u8> {
        let mut p = vec![0u8; METER_PACKET];
        for &(slot, v) in peaks {
            let o = METER_SLOTS * 8 + slot * 4;
            p[o..o + 4].copy_from_slice(&v.to_le_bytes());
        }
        p
    }

    #[test]
    fn meter_packet_slots_and_peak_hold() {
        let mut m = DspMeters::default();
        // -20 dBFS on playback 1, routed to AN1 (out 0).
        let tenth = (METER_FULL_SCALE / 10.0) as u32;
        m.apply(&meter_packet(&[(10, tenth), (24, tenth)]));
        m.apply(&meter_packet(&[(10, tenth / 2), (0, 1 << 27)]));
        let inputs = m.take(0, INPUTS);
        assert_eq!(inputs[0], 1.0);
        let pb = m.take(METER_PLAYBACK_BASE, 2);
        assert!((pb[0] - 0.1).abs() < 1e-4, "held peak, not the later lower one");
        assert_eq!(pb[1], 0.0);
        let out = m.take(METER_OUTPUT_BASE, OUTPUTS);
        assert!((out[0] - 0.1).abs() < 1e-4);
        // Taking resets the hold.
        assert_eq!(m.take(METER_PLAYBACK_BASE, 1), vec![0.0]);
        // Short packets are ignored.
        m.apply(&[0u8; 16]);
        assert_eq!(m.take(0, 1), vec![0.0]);
    }
}
