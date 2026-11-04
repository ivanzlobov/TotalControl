//! OSC (Open Sound Control) bridge.
//!
//! Lets a TotalMix FX controller, a script or a second UI read and write the
//! mixer over UDP alongside the GUI.  Opt-in via `--osc` (see `main.rs`); off
//! by default and bound to loopback only, so enabling it never silently
//! exposes control to the LAN.
//!
//! The wire surface is the TotalMix FX OSC address space in
//! `docs/spec/OSC-CONTROL-SPEC.md`: `osc_rme` decodes incoming paths into the
//! same [`Message`]s the GUI produces.  The GUI keeps publishing its state as
//! [`OscOutbound`] values in its own internal form; this worker translates
//! those to the TotalMix feedback paths, using the bank, selected channel and
//! submix state it already tracks for incoming commands.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::stream;
use rosc::{OscMessage, OscPacket, OscType};
use tokio::net::UdpSocket;

use tuxmix_core::ChannelId;

use crate::app::Message;

/// Bind/target ports for the OSC bridge — `Hash` because it's threaded
/// through `Subscription::run_with` as the recipe's identifying data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OscConfig {
    pub recv_port: u16,
    pub send_port: u16,
    pub send_host: IpAddr,
}

/// A state change to echo out to any connected OSC client — sent both for
/// GUI-originated changes and for changes that came in as a controller
/// command, so a controller and the GUI never drift out of sync with each
/// other.
///
/// Addresses in [`OscOutbound::Raw`] are internal to the application.  The
/// worker maps them to the TotalMix paths the controller can address; see
/// `osc_rme`.
#[derive(Debug, Clone, PartialEq)]
pub enum OscOutbound {
    Volume(ChannelId, usize, f32),
    Pan(ChannelId, usize, i8),
    Mute(ChannelId, bool),
    Solo(ChannelId, bool),
    OutputVolume(usize, f32),
    /// Any of the device controls not covered by the matrix variants.
    /// Address and arguments are already in their internal form, which
    /// keeps the feedback path from growing a variant per knob.
    Raw {
        addr: String,
        args: Vec<OscType>,
    },
}

/// One human-readable line per OSC message crossing the bridge in either
/// direction — for `Message::OscLog`/the debug log panel in `app.rs`. Incoming
/// lines are built from the raw address and args before `osc_rme::parse`
/// gets a chance to reject anything, so a controller sending a malformed or
/// unmapped address still shows up. Outgoing lines carry the translated
/// TotalMix path, which is what the controller actually receives.
fn log_line(dir: &str, addr: &str, args: &[OscType]) -> String {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = elapsed.as_secs() % 86400;
    let (h, m, s) = (secs / 3600, (secs % 3600) / 60, secs % 60);
    format!(
        "{h:02}:{m:02}:{s:02}.{:03} {dir} {addr} {args:?}",
        elapsed.subsec_millis()
    )
}

/// Runs the OSC bridge as an iced `Subscription` recipe: binds a UDP
/// socket on loopback, decodes incoming packets into `Message`s, and drains
/// an internal channel (handed back to the app via `Message::OscReady`) to
/// send outgoing feedback packets. Every message in either direction also
/// gets echoed out as a `Message::OscLog` line, for the debug log panel — a
/// no-op from the bridge's own perspective if that panel is never opened,
/// since the log is a plain bounded ring buffer on the `update()` side, not
/// something this worker needs to know about.
///
/// Never touches `state.device` — the `alsa` mixer handle it wraps has no
/// `Send`/`Sync` impl, so it must stay owned by the single update-loop
/// thread. This worker only ever exchanges plain values with `update()`,
/// the same way the existing global keyboard/mouse listener does.
pub fn worker(config: &OscConfig) -> impl Stream<Item = Message> {
    let config = *config;
    stream::channel(100, async move |mut output| {
        let recv_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), config.recv_port);
        let socket = match UdpSocket::bind(recv_addr).await {
            Ok(s) => s,
            Err(err) => {
                log::error!("OSC: failed to bind {recv_addr}: {err}");
                return;
            }
        };
        let send_addr = SocketAddr::new(config.send_host, config.send_port);

        let (tx, mut rx) = mpsc::channel::<OscOutbound>(4096);
        let mut rme = crate::osc_rme::RmeState::default();
        let mut feedback = crate::osc_rme::RmeFeedback::default();
        if output.send(Message::OscReady(tx)).await.is_err() {
            return;
        }

        log::info!("OSC: listening on {recv_addr}, feedback to {send_addr}");
        let mut buf = [0u8; rosc::decoder::MTU];
        loop {
            tokio::select! {
                res = socket.recv_from(&mut buf) => {
                    let Ok((size, _from)) = res else { continue };
                    let Ok((_, packet)) = rosc::decoder::decode_udp(&buf[..size]) else { continue };
                    if let OscPacket::Message(msg) = packet {
                        let _ = output
                            .send(Message::OscLog(log_line("IN ", &msg.addr, &msg.args)))
                            .await;
                        if let Some(messages) =
                            crate::osc_rme::parse(&msg.addr, &msg.args, &mut rme)
                        {
                            for message in messages {
                                if output.send(message).await.is_err() {
                                    return;
                                }
                            }
                        }
                        // Selecting a page makes the controller throw its
                        // cached values away; the RME table requires a full
                        // resend of the page the controller now addresses.
                        if crate::osc_rme::page_changed(&msg.addr) {
                            for (addr, args) in feedback.page_dump(&rme) {
                                let packet = OscPacket::Message(OscMessage { addr, args });
                                if let OscPacket::Message(ref msg) = packet {
                                    let _ = output
                                        .send(Message::OscLog(log_line("OUT", &msg.addr, &msg.args)))
                                        .await;
                                }
                                if let Ok(bytes) = rosc::encoder::encode(&packet) {
                                    let _ = socket.send_to(&bytes, send_addr).await;
                                }
                            }
                        }
                    }
                }
                outbound = rx.next() => {
                    let Some(outbound) = outbound else { continue };
                    for (addr, args) in feedback.translate(outbound, &rme) {
                        let packet = OscPacket::Message(OscMessage { addr, args });
                        if let OscPacket::Message(ref msg) = packet {
                            let _ = output
                                .send(Message::OscLog(log_line("OUT", &msg.addr, &msg.args)))
                                .await;
                        }
                        if let Ok(bytes) = rosc::encoder::encode(&packet) {
                            let _ = socket.send_to(&bytes, send_addr).await;
                        }
                    }
                }
            }
        }
    })
}
