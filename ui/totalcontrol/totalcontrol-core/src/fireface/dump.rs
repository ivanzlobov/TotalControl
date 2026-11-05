//! Decode the UCX full state dump into scene fields.
//!
//! The dump is requested with `0x3f99 = 0x0812`; see
//! `docs/spec/FIREFFACE-UCX-PROTOCOL-SPEC.md`, appendix A.

use std::collections::HashMap;

use crate::channel::InputLevel;
use crate::scene::Scene;

/// Output pairs in the requested dump.  TotalControl's pair 0 is Phones;
/// the dump stores Phones at 0x18 and AN1/2 at 0x12.
const OUTPUT_BLOCKS: [u16; 9] = [0x18, 0x12, 0x14, 0x16, 0x1a, 0x1c, 0x1e, 0x20, 0x22];

#[derive(Clone, Debug, Default)]
pub struct UcxDump {
    pub words: HashMap<u16, u16>,
}

impl UcxDump {
    pub fn new(words: HashMap<u16, u16>) -> Self {
        Self { words }
    }

    fn input(&self, idx: usize, offset: u16) -> Option<u16> {
        self.words.get(&(((idx as u16) << 8) | offset)).copied()
    }

    fn output(&self, pair: usize, side: usize, offset: u16) -> Option<u16> {
        let block = *OUTPUT_BLOCKS.get(pair)? + side as u16;
        self.words.get(&((block << 8) | offset)).copied()
    }

    fn matrix_at(&self, pair: usize, side: usize, source: usize) -> Option<(u16, u16)> {
        let block = *OUTPUT_BLOCKS.get(pair)? + side as u16;
        let offset = if source < 8 {
            0xa0 + 2 * source as u16
        } else if source < 10 {
            0xc0 + 2 * (source as u16 - 8)
        } else if source < 18 {
            0xe0 + 2 * (source as u16 - 10)
        } else {
            return None;
        };
        Some((
            self.words.get(&((block << 8) | offset)).copied()?,
            self.words.get(&((block << 8) | offset + 1)).copied()?,
        ))
    }

    fn signed(word: u16) -> i16 {
        word as i16
    }

    fn tenths_db(word: u16) -> f32 {
        Self::signed(word) as f32 / 10.0
    }

    fn db_to_linear(db: f32) -> f32 {
        10f32.powf(db / 20.0)
    }

    fn matrix_level(word: u16) -> f32 {
        // A zero record is no send (not unity).
        if word == 0 {
            return 0.0;
        }
        let db = (Self::signed(word) >> 4) as f32 / 10.0;
        Self::db_to_linear(db)
    }

    fn matrix_pan(word: u16) -> i8 {
        Self::signed(word).clamp(-100, 100) as i8
    }

    fn pan_split(volume: f32, pan: i8) -> (f32, f32) {
        let v = volume.clamp(0.0, 1.0);
        let p = pan.clamp(-100, 100) as f32 / 100.0;
        (
            v * if p > 0.0 { 1.0 - p } else { 1.0 },
            v * if p < 0.0 { 1.0 + p } else { 1.0 },
        )
    }

    /// Compare the dump with `scene` and return human-readable differences.
    /// The dump's known fields are also applied to a clone of the scene.
    pub fn differences_into(&self, scene: &mut Scene) -> Vec<String> {
        let mut out = Vec::new();
        if self.words.is_empty() {
            return out;
        }

        for (idx, input) in scene.inputs.iter_mut().enumerate().take(18) {
            if idx < 4 {
                if let Some(word) = self.input(idx, 0x07) {
                    let gain = if idx < 2 {
                        word as u32 / 10
                    } else {
                        word as u32 / 5
                    };
                    if input.gain != Some(gain) {
                        let old = input
                            .gain
                            .map(|g| format!("{g}"))
                            .unwrap_or_else(|| "-".into());
                        out.push(format!(
                            "AN{} gain: device {gain}, TotalControl {old}",
                            idx + 1
                        ));
                        input.gain = Some(gain);
                    }
                }
            }
            if let Some(word) = self.input(idx, 0x08) {
                if idx < 2 {
                    let phantom = word & 1 != 0;
                    if input.phantom != phantom {
                        out.push(format!(
                            "AN{} 48 V: device {phantom}, TotalControl {}",
                            idx + 1,
                            input.phantom
                        ));
                        input.phantom = phantom;
                    }
                } else if let Some(level) = InputLevel::from_code(word) {
                    if input.level != Some(level) {
                        out.push(format!(
                            "AN{} Level: device {:?}, TotalControl {:?}",
                            idx + 1,
                            level,
                            input.level
                        ));
                        input.level = Some(level);
                        input.ref_level = level.code();
                    }
                }
            }
            if let Some(word) = self.input(idx, 0x09) {
                let on = word != 0;
                if input.instrument != on {
                    out.push(format!(
                        "AN{} Inst: device {on}, TotalControl {}",
                        idx + 1,
                        input.instrument
                    ));
                    input.instrument = on;
                }
            }
            if let Some(word) = self.input(idx, 0x0a) {
                let on = word != 0;
                if input.autoset != on {
                    out.push(format!(
                        "AN{} AutoSet: device {on}, TotalControl {}",
                        idx + 1,
                        input.autoset
                    ));
                    input.autoset = on;
                }
            }
            if let Some(word) = self.input(idx, 0x00) {
                let mute = word != 0;
                if input.mute != mute {
                    out.push(format!(
                        "AN{} mute: device {mute}, TotalControl {}",
                        idx + 1,
                        input.mute
                    ));
                    input.mute = mute;
                }
            }
            if let Some(word) = self.input(idx, 0x01) {
                let db = Self::tenths_db(word);
                if (input.fx_send_db - db).abs() > 0.05 {
                    out.push(format!(
                        "AN{} FX send: device {db:.1} dB, TotalControl {:.1} dB",
                        idx + 1,
                        input.fx_send_db
                    ));
                    input.fx_send_db = db;
                }
            }
            if let Some(word) = self.input(idx, 0x06) {
                let phase = word != 0;
                if input.phase != phase {
                    out.push(format!(
                        "AN{} phase: device {phase}, TotalControl {}",
                        idx + 1,
                        input.phase
                    ));
                    input.phase = phase;
                }
            }
            if let Some(word) = self.input(idx, 0x05) {
                let ms = word != 0;
                if input.ms_proc != ms {
                    out.push(format!(
                        "AN{} MS: device {ms}, TotalControl {}",
                        idx + 1,
                        input.ms_proc
                    ));
                    input.ms_proc = ms;
                }
            }
            if let Some(word) = self.input(idx, 0x20) {
                if let Some(eq) = input.eq.as_mut() {
                    let on = word != 0;
                    if eq.low_cut_enabled != on {
                        out.push(format!(
                            "AN{} low cut: device {on}, TotalControl {}",
                            idx + 1,
                            eq.low_cut_enabled
                        ));
                        eq.low_cut_enabled = on;
                    }
                }
            }
            if let Some(word) = self.input(idx, 0x40) {
                if let Some(eq) = input.eq.as_mut() {
                    let on = word != 0;
                    if eq.enabled != on {
                        out.push(format!(
                            "AN{} EQ: device {on}, TotalControl {}",
                            idx + 1,
                            eq.enabled
                        ));
                        eq.enabled = on;
                    }
                }
            }

            // The dump stores the post-trim, post-pan send.  Compare it
            // with the model's pre-trim fader and pan only for the four
            // mono inputs; stereo pairs use the pair writer's 2x2 matrix
            // and need the width/M-S model to reconstruct.
            if idx < 4 {
                for pair in 0..9 {
                    let Some((left, left_pan)) = self.matrix_at(pair, 0, idx) else {
                        continue;
                    };
                    let Some((right, _)) = self.matrix_at(pair, 1, idx) else {
                        continue;
                    };
                    let volume = input.volumes.get(pair).copied().unwrap_or(0.0);
                    let pan = input.pans.get(pair).copied().unwrap_or(0);
                    let (expected_left, expected_right) = Self::pan_split(volume, pan);
                    let trim = |side: usize| {
                        let no_trim = scene
                            .outputs
                            .get(pair * 2 + side)
                            .is_some_and(|output| output.no_trim);
                        if no_trim {
                            1.0
                        } else {
                            Self::db_to_linear(input.trim)
                        }
                    };
                    let expected_left = expected_left * trim(0);
                    let expected_right = expected_right * trim(1);
                    let device_left = Self::matrix_level(left);
                    let device_right = Self::matrix_level(right);
                    if (device_left - expected_left).abs() > 0.005
                        || (device_right - expected_right).abs() > 0.005
                    {
                        out.push(format!(
                            "AN{} -> output {}: device {:.3}/{:.3}, TotalControl {:.3}/{:.3}",
                            idx + 1,
                            pair,
                            device_left,
                            device_right,
                            expected_left,
                            expected_right
                        ));
                    }
                    if expected_left.max(expected_right) > 0.0005 {
                        let device_pan = Self::matrix_pan(left_pan);
                        if device_pan != pan {
                            out.push(format!(
                                "AN{} -> output {} pan: device {device_pan}, TotalControl {pan}",
                                idx + 1,
                                pair
                            ));
                        }
                    }
                }
            }
        }

        for pair in 0..9 {
            for side in 0..2 {
                let idx = pair * 2 + side;
                let Some(output) = scene.outputs.get_mut(idx) else {
                    continue;
                };
                if let Some(word) = self.output(pair, side, 0x00) {
                    let db = Self::tenths_db(word);
                    let volume = Self::db_to_linear(db);
                    if (output.volume - volume).abs() > 0.005 {
                        out.push(format!(
                            "output {idx} volume: device {db:.1} dB, TotalControl {:.1} dB",
                            linear_db(output.volume)
                        ));
                        output.volume = volume;
                    }
                }
                if let Some(word) = self.output(pair, side, 0x01) {
                    let balance = Self::signed(word).clamp(-100, 100) as i8;
                    if output.balance != balance {
                        out.push(format!(
                            "output {idx} balance: device {balance}, TotalControl {}",
                            output.balance
                        ));
                        output.balance = balance;
                    }
                }
                if let Some(word) = self.output(pair, side, 0x02) {
                    let mute = word != 0;
                    if output.mute != mute {
                        out.push(format!(
                            "output {idx} mute: device {mute}, TotalControl {}",
                            output.mute
                        ));
                        output.mute = mute;
                    }
                }
                if let Some(word) = self.output(pair, side, 0x03) {
                    let db = Self::tenths_db(word);
                    if (output.fx_return_db - db).abs() > 0.05 {
                        out.push(format!(
                            "output {idx} FX return: device {db:.1} dB, TotalControl {:.1} dB",
                            output.fx_return_db
                        ));
                        output.fx_return_db = db;
                    }
                }
                if let Some(word) = self.output(pair, side, 0x07) {
                    let phase = word != 0;
                    if output.phase_l != phase {
                        out.push(format!(
                            "output {idx} phase: device {phase}, TotalControl {}",
                            output.phase_l
                        ));
                        output.phase_l = phase;
                    }
                }
            }
        }

        if let Some(word) = self.words.get(&0x3c00).copied() {
            if let Some(reverb) = scene.settings.reverb.as_mut() {
                let on = word != 0;
                if reverb.on != on {
                    out.push(format!("Reverb: device {on}, TotalControl {}", reverb.on));
                    reverb.on = on;
                }
            }
        }
        if let Some(word) = self.words.get(&0x3c20).copied() {
            if let Some(echo) = scene.settings.echo.as_mut() {
                let on = word != 0;
                if echo.on != on {
                    out.push(format!("Echo: device {on}, TotalControl {}", echo.on));
                    echo.on = on;
                }
            }
        }

        out
    }

    pub fn apply_to_scene(&self, scene: &mut Scene) -> Vec<String> {
        self.differences_into(scene)
    }
}

fn linear_db(v: f32) -> f32 {
    if v <= 0.0 {
        -65.0
    } else {
        20.0 * v.log10()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::{ChannelType, InputChannel, OutputChannel};
    use std::collections::HashMap;

    fn scene() -> Scene {
        let mut scene = Scene::new("test");
        for idx in 0..18 {
            scene.inputs.push(InputChannel::new(
                idx,
                &format!("AN{}", idx + 1),
                ChannelType::Line,
                9,
            ));
            scene
                .outputs
                .push(OutputChannel::new(idx, &format!("OUT {idx}")));
        }
        scene
    }

    fn level_word(tenths: i16) -> u16 {
        (tenths.clamp(-650, 60) << 4) as u16
    }

    #[test]
    fn matrix_comparison_applies_trim_and_pan() {
        let mut scene = scene();
        let input = &mut scene.inputs[0];
        input.volumes[1] = 1.0;
        input.pans[1] = -100;
        input.trim = -17.223_293;

        let mut words = HashMap::new();
        // AN1 -> output pair 1 (AN1/2), after trim and hard-left pan.
        words.insert(0x12a0, level_word(-172));
        words.insert(0x12a1, (-100i16) as u16);
        words.insert(0x13a0, 0);
        words.insert(0x13a1, (-100i16) as u16);

        let diff = UcxDump::new(words).differences_into(&mut scene);
        assert!(diff.is_empty(), "{diff:?}");
    }

    #[test]
    fn matrix_comparison_ignores_pan_when_send_is_silent() {
        let mut scene = scene();
        let input = &mut scene.inputs[0];
        input.volumes[1] = 0.0;
        input.pans[1] = 100;

        let mut words = HashMap::new();
        words.insert(0x12a0, 0);
        words.insert(0x12a1, 0);
        words.insert(0x13a0, 0);
        words.insert(0x13a1, 0);

        let diff = UcxDump::new(words).differences_into(&mut scene);
        assert!(diff.is_empty(), "{diff:?}");
    }
}
