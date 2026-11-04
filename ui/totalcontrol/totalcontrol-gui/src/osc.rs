//! OSC (Open Sound Control) transport.
//!
//! The wire surface is the classic TotalMix FX address space implemented in
//! [`crate::osc_rme`].  This module only owns the UDP socket, the debug log
//! and the outgoing feedback queue; it never decodes a control path itself.
//!
//! Opt-in from the app settings or `--osc` (see `main.rs`).  Off by default,
//! and bound to loopback only, so enabling it never silently exposes control
//! to the LAN.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::stream;
use rosc::{OscMessage, OscPacket, OscType};
use tokio::net::UdpSocket;

use totalcontrol_core::ChannelId;

use crate::app::Message;

/// Bind/target ports for the OSC bridge — `Hash` because it's threaded
/// through `Subscription::run_with` as the recipe's identifying data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OscConfig {
    pub recv_port: u16,
    pub send_port: u16,
    pub send_host: IpAddr,
}

/// An internal change hint consumed by the app's RME feedback layer.
/// Only [`OscOutbound::Raw`] is ever put on the wire.
#[derive(Debug, Clone, PartialEq)]
pub enum OscOutbound {
    Volume(ChannelId, usize, f32),
    Pan(ChannelId, usize, i8),
    Mute(ChannelId, bool),
    Solo(ChannelId, bool),
    /// Any feedback control.  Address and arguments are already in
    /// their wire form, which keeps the feedback path from growing a
    /// variant per knob.
    Raw {
        addr: String,
        args: Vec<OscType>,
    },
}

fn channel_prefix(id: ChannelId) -> (&'static str, usize) {
    match id {
        ChannelId::Input(i) => ("input", i),
        ChannelId::Playback(i) => ("playback", i),
        ChannelId::Output(i) => ("output", i),
    }
}

impl OscOutbound {
    fn into_packet(self) -> OscPacket {
        let (addr, args) = match self {
            OscOutbound::Volume(id, out, v) => {
                if let ChannelId::Output(i) = id {
                    // Outputs have a single master, not a per-submix
                    // matrix: keep the public `/output/<id>/volume`
                    // address instead of the matrix form.
                    (format!("/output/{i}/volume"), vec![OscType::Float(v)])
                } else {
                    let (kind, i) = channel_prefix(id);
                    (format!("/{kind}/{i}/volume/{out}"), vec![OscType::Float(v)])
                }
            }
            OscOutbound::Pan(id, out, p) => {
                let (kind, i) = channel_prefix(id);
                (
                    format!("/{kind}/{i}/pan/{out}"),
                    vec![OscType::Float(p as f32)],
                )
            }
            OscOutbound::Mute(id, m) => {
                let (kind, i) = channel_prefix(id);
                (
                    format!("/{kind}/{i}/mute"),
                    vec![OscType::Float(if m { 1.0 } else { 0.0 })],
                )
            }
            OscOutbound::Solo(id, s) => {
                let (kind, i) = channel_prefix(id);
                (
                    format!("/{kind}/{i}/solo"),
                    vec![OscType::Float(if s { 1.0 } else { 0.0 })],
                )
            }
            OscOutbound::Raw { addr, args } => (addr, args),
        };
        OscPacket::Message(OscMessage { addr, args })
    }
}

/// One human-readable line per OSC message crossing the bridge in either
/// direction — for `Message::OscLog`/the debug log panel in `app.rs`.  It is
/// built before the RME parser runs, so an unknown or malformed address
/// still shows up; that is the point of a debug log: seeing why nothing
/// happened, not just confirming when something did.
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
/// socket on loopback, forwards incoming packets as `Message::OscRmeIn`,
/// and drains an internal channel (handed back to the app via
/// `Message::OscReady`) to send outgoing feedback packets. Every message in
/// either direction also gets echoed out as a `Message::OscLog` line, for
/// the debug log panel — a no-op from the bridge's own perspective if that
/// panel is never opened, since the log is a plain bounded ring buffer on
/// the `update()` side, not something this worker needs to know about.
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
                        if output
                            .send(Message::OscRmeIn(msg.addr, msg.args))
                            .await
                            .is_err()
                        {
                            return;
                        }
                    }
                }
                outbound = rx.next() => {
                    let Some(outbound) = outbound else { continue };
                    let packet = outbound.into_packet();
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_outbound_round_trips() {
        let packet = OscOutbound::Raw {
            addr: "/1/mute/1/1".into(),
            args: vec![OscType::Float(1.0)],
        }
        .into_packet();
        let bytes = rosc::encoder::encode(&packet).unwrap();
        let (_, decoded) = rosc::decoder::decode_udp(&bytes).unwrap();
        let OscPacket::Message(msg) = decoded else {
            panic!("expected a message");
        };
        assert_eq!(msg.addr, "/1/mute/1/1");
        assert_eq!(msg.args, vec![OscType::Float(1.0)]);
    }

    #[test]
    fn the_worker_logs_the_classic_address() {
        let line = log_line("IN ", "/2/volume", &[OscType::Float(0.5)]);
        assert!(line.contains("IN  /2/volume"));
    }
}
