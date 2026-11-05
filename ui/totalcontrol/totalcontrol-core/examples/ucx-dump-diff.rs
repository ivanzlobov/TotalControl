//! Decode the UCX state dump by controlled single-control changes.
//!
//!   cargo run -p totalcontrol-core --example ucx-dump-diff --features alsa -- 2
//!
//! The example requests the full state dump before and after each change and
//! prints the addresses/values that moved.  It restores each control before
//! the next test.

#[cfg(feature = "alsa")]
fn main() {
    use std::collections::HashMap;
    use std::time::Duration;
    use totalcontrol_core::fireface::{Fireface, Model};
    use totalcontrol_core::{ChannelId, InputLevel, RmeDevice};

    let card: i32 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);
    let found = Fireface::enumerate()
        .into_iter()
        .find(|info| info.card == card && info.model == Model::Ucx)
        .expect("no UCX on that card");
    let mut dev = Fireface::open_identity(&found.identity()).expect("open UCX");
    dev.capture_state_dump(Duration::from_millis(2500))
        .expect("baseline dump");
    let mut prev = dev.state_words().clone();
    if prev.is_empty() {
        panic!("no state dump");
    }
    println!("baseline {} words", prev.len());

    fn run<A, U>(
        dev: &mut Fireface,
        prev: &mut HashMap<u16, u16>,
        name: &str,
        apply: A,
        undo: U,
    ) where
        A: FnOnce(&mut Fireface),
        U: FnOnce(&mut Fireface),
    {
        apply(dev);
        dev.capture_state_dump(Duration::from_millis(2500))
            .expect("changed dump");
        let now = dev.state_words().clone();
        let mut diff: Vec<_> = now
            .iter()
            .filter(|(a, v)| !(0x3f00..=0x3f50).contains(*a) && prev.get(a) != Some(v))
            .map(|(a, v)| (*a, prev.get(a).copied(), *v))
            .collect();
        diff.sort_unstable_by_key(|(a, _, _)| *a);
        println!("\n{name}: {} changed", diff.len());
        for (a, old, new) in diff.iter().take(40) {
            println!(
                "  {a:04x}: {} -> {new:04x}",
                old.map(|v| format!("{v:04x}")).unwrap_or_else(|| "-".into())
            );
        }
        undo(dev);
        dev.capture_state_dump(Duration::from_millis(2500))
            .expect("restore dump");
        *prev = dev.state_words().clone();
    }

    run(
        &mut dev,
        &mut prev,
        "input 0 gain +30 dB",
        |d| d.set_input_gain(0, 30).unwrap(),
        |d| d.set_input_gain(0, 0).unwrap(),
    );
    run(
        &mut dev,
        &mut prev,
        "input 0 48 V on",
        |d| d.set_input_phantom(0, true).unwrap(),
        |d| d.set_input_phantom(0, false).unwrap(),
    );
    run(
        &mut dev,
        &mut prev,
        "input 2 Inst on",
        |d| d.set_input_instrument(2, true).unwrap(),
        |d| d.set_input_instrument(2, false).unwrap(),
    );
    let orig_level = dev.inputs()[2].level.unwrap_or(InputLevel::Plus4dBu);
    run(
        &mut dev,
        &mut prev,
        "input 2 Level Minus10",
        |d| d.set_input_level(2, InputLevel::Minus10dBV).unwrap(),
        |d| d.set_input_level(2, orig_level).unwrap(),
    );
    run(
        &mut dev,
        &mut prev,
        "input 0 AutoSet on",
        |d| d.set_autoset(0, true).unwrap(),
        |d| d.set_autoset(0, false).unwrap(),
    );
    run(
        &mut dev,
        &mut prev,
        "input 0 low cut on",
        |d| d.set_eq_low_cut_enabled(0, true).unwrap(),
        |d| d.set_eq_low_cut_enabled(0, false).unwrap(),
    );
    run(
        &mut dev,
        &mut prev,
        "input 0 EQ on",
        |d| d.set_eq_enabled(0, true).unwrap(),
        |d| d.set_eq_enabled(0, false).unwrap(),
    );
    run(
        &mut dev,
        &mut prev,
        "input 0 EQ band 1 +6 dB",
        |d| d.set_eq_band_gain(0, 0, 6.0).unwrap(),
        |d| d.set_eq_band_gain(0, 0, 0.0).unwrap(),
    );
    run(
        &mut dev,
        &mut prev,
        "input 0 phase",
        |d| d.set_phase(0, true).unwrap(),
        |d| d.set_phase(0, false).unwrap(),
    );
    run(
        &mut dev,
        &mut prev,
        "input 0 phase right",
        |d| d.set_input_phase_right(0, true).unwrap(),
        |d| d.set_input_phase_right(0, false).unwrap(),
    );
    run(
        &mut dev,
        &mut prev,
        "input 0 channel width 0.5",
        |d| d.set_channel_width(ChannelId::Input(0), 0.5).unwrap(),
        |d| d.set_channel_width(ChannelId::Input(0), 1.0).unwrap(),
    );
    let out_vol = dev.outputs()[0].volume;
    run(
        &mut dev,
        &mut prev,
        "output 0 volume 0.5",
        |d| d.set_volume(ChannelId::Output(0), 0, 0.5).unwrap(),
        |d| d.set_volume(ChannelId::Output(0), 0, out_vol).unwrap(),
    );
    let out_pan = dev.pan(ChannelId::Output(0), 0).unwrap_or(0);
    run(
        &mut dev,
        &mut prev,
        "output 0 balance -50",
        |d| d.set_pan(ChannelId::Output(0), 0, -50).unwrap(),
        |d| d.set_pan(ChannelId::Output(0), 0, out_pan).unwrap(),
    );
    let in_send = dev.volume(ChannelId::Input(0), 0).unwrap_or(1.0);
    run(
        &mut dev,
        &mut prev,
        "input 0 -> output 0 send 0.5",
        |d| d.set_input_volume(0, 0, 0, 0.5).unwrap(),
        |d| d.set_input_volume(0, 0, 0, in_send).unwrap(),
    );
    let in2_send = dev.volume(ChannelId::Input(2), 0).unwrap_or(1.0);
    run(
        &mut dev,
        &mut prev,
        "input pair 1 -> output 0 send 0.5",
        |d| d.set_input_volume(1, 0, 0, 0.5).unwrap(),
        |d| d.set_input_volume(1, 0, 0, in2_send).unwrap(),
    );
    let pb_send = dev.volume(ChannelId::Playback(0), 0).unwrap_or(1.0);
    run(
        &mut dev,
        &mut prev,
        "playback 0 -> output 0 send 0.9",
        |d| d.set_volume(ChannelId::Playback(0), 0, 0.9).unwrap(),
        |d| d.set_volume(ChannelId::Playback(0), 0, pb_send).unwrap(),
    );
    let in_mute = dev.inputs()[0].mute;
    run(
        &mut dev,
        &mut prev,
        "input 0 mute",
        |d| d.set_mute(ChannelId::Input(0), true).unwrap(),
        |d| d.set_mute(ChannelId::Input(0), in_mute).unwrap(),
    );
    let in_solo = dev.inputs()[0].solo;
    run(
        &mut dev,
        &mut prev,
        "input 0 solo",
        |d| d.set_solo(ChannelId::Input(0), true).unwrap(),
        |d| d.set_solo(ChannelId::Input(0), in_solo).unwrap(),
    );
    let in_fx = dev.inputs()[0].fx_send_db;
    run(
        &mut dev,
        &mut prev,
        "input 0 FX send -10 dB",
        |d| d.set_channel_fx_send(ChannelId::Input(0), -10.0).unwrap(),
        |d| d.set_channel_fx_send(ChannelId::Input(0), in_fx).unwrap(),
    );
    let in_pan = dev.pan(ChannelId::Input(0), 0).unwrap_or(0);
    run(
        &mut dev,
        &mut prev,
        "input 0 pan -50",
        |d| d.set_pan(ChannelId::Input(0), 0, -50).unwrap(),
        |d| d.set_pan(ChannelId::Input(0), 0, in_pan).unwrap(),
    );
    let out_mute = dev.outputs()[0].mute;
    run(
        &mut dev,
        &mut prev,
        "output 0 mute",
        |d| d.set_mute(ChannelId::Output(0), true).unwrap(),
        |d| d.set_mute(ChannelId::Output(0), out_mute).unwrap(),
    );
    let out_phase = dev.outputs()[0].phase_l;
    run(
        &mut dev,
        &mut prev,
        "output 0 phase left",
        |d| d.set_output_phase(0, true).unwrap(),
        |d| d.set_output_phase(0, out_phase).unwrap(),
    );
    let out1_vol = dev.outputs()[1].volume;
    run(
        &mut dev,
        &mut prev,
        "output 1 volume 0.5",
        |d| d.set_volume(ChannelId::Output(1), 0, 0.5).unwrap(),
        |d| d.set_volume(ChannelId::Output(1), 0, out1_vol).unwrap(),
    );
    let out2_vol = dev.outputs()[2].volume;
    run(
        &mut dev,
        &mut prev,
        "output 2 volume 0.5",
        |d| d.set_volume(ChannelId::Output(2), 0, 0.5).unwrap(),
        |d| d.set_volume(ChannelId::Output(2), 0, out2_vol).unwrap(),
    );
    let out_linked = dev.output_linked(0);
    run(
        &mut dev,
        &mut prev,
        "output pair 0 unlink",
        |d| d.set_output_linked(0, false).unwrap(),
        |d| d.set_output_linked(0, out_linked).unwrap(),
    );
    let out_fx = dev.outputs()[0].fx_return_db;
    run(
        &mut dev,
        &mut prev,
        "output 0 FX return -20 dB",
        |d| d.set_fx_return(0, -20.0).unwrap(),
        |d| d.set_fx_return(0, out_fx).unwrap(),
    );
    let mut reverb = dev.settings().reverb.clone().unwrap_or_default();
    let reverb_on = reverb.on;
    reverb.on = !reverb_on;
    run(
        &mut dev,
        &mut prev,
        "reverb toggle",
        |d| d.set_reverb(&reverb).unwrap(),
        |d| {
            let mut r = d.settings().reverb.clone().unwrap_or_default();
            r.on = reverb_on;
            d.set_reverb(&r).unwrap();
        },
    );
    let mut echo = dev.settings().echo.clone().unwrap_or_default();
    let echo_on = echo.on;
    echo.on = !echo_on;
    run(
        &mut dev,
        &mut prev,
        "echo toggle",
        |d| d.set_echo(&echo).unwrap(),
        |d| {
            let mut e = d.settings().echo.clone().unwrap_or_default();
            e.on = echo_on;
            d.set_echo(&e).unwrap();
        },
    );
    // Link pair 0, then isolate MS Proc on the linked pair.
    dev.set_input_pair_linked(0, true).unwrap();
    dev.capture_state_dump(Duration::from_millis(2500)).unwrap();
    prev = dev.state_words().clone();
    run(
        &mut dev,
        &mut prev,
        "input 0 MS proc on linked pair",
        |d| d.set_input_ms_proc(0, true).unwrap(),
        |d| d.set_input_ms_proc(0, false).unwrap(),
    );
    dev.set_input_pair_linked(0, false).unwrap();
    dev.capture_state_dump(Duration::from_millis(2500)).unwrap();

    println!("\nrestored");
}

#[cfg(not(feature = "alsa"))]
fn main() {
    eprintln!("build with --features alsa");
}
