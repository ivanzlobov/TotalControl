//! Playback-following sample rate.
//!
//! The graph has one sample rate.  This module reads the PipeWire graph with
//! `pw-dump` and works out which player stream is feeding SW playback 1/2
//! (`playback_AUX0` / `playback_AUX1`).  The owner policy itself is pure so
//! it can be exercised without PipeWire or a device:
//!
//! * a live owner keeps the rate while its node exists;
//! * a different client never steals it;
//! * a same-client replacement (the normal track-change case) inherits it;
//! * a missing owner reserves the client for two seconds, after which the
//!   most recent remaining candidate takes over;
//! * a new rate is applied only after it has been stable for 300 ms.
//!
//! The GUI applies the returned rate through its ordinary engine-rate path;
//! this module only chooses it.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// How long a disappeared owner's client keeps the right to the rate.
pub const HANDOVER: Duration = Duration::from_secs(2);
/// How long a candidate rate must stay unchanged before it is forced.
pub const STABLE: Duration = Duration::from_millis(300);

/// A node in a captured PipeWire graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphNode {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub media_class: String,
    pub state: String,
    pub client_id: Option<u64>,
    pub process_id: Option<u64>,
    pub application_name: Option<String>,
    pub serial: u64,
    pub rate: Option<u32>,
}

/// A port in a captured PipeWire graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphPort {
    pub id: u32,
    pub node_id: u32,
    pub name: String,
    pub direction: String,
}

/// A link between two ports.  `state` is `error` for a failed link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphLink {
    pub output_node: u32,
    pub output_port: u32,
    pub input_node: u32,
    pub input_port: u32,
    pub state: String,
}

/// The part of `pw-dump` the follower uses.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GraphSnapshot {
    pub nodes: Vec<GraphNode>,
    pub ports: Vec<GraphPort>,
    pub links: Vec<GraphLink>,
}

/// How a stream is identified across a player recreating its node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientIdentity {
    pub client_id: Option<u64>,
    pub process_id: Option<u64>,
    pub application_name: String,
}

impl ClientIdentity {
    /// The same client, as PipeWire exposes it.  `client.id` is the stable
    /// identity; `application.process.id` is the fallback when it is absent.
    pub fn same_client(&self, other: &Self) -> bool {
        if self.application_name != other.application_name {
            return false;
        }
        match (self.client_id, other.client_id) {
            (Some(a), Some(b)) => a == b,
            _ => match (self.process_id, other.process_id) {
                (Some(a), Some(b)) => a == b,
                _ => false,
            },
        }
    }
}

/// A stream that ultimately feeds SW playback 1/2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateCandidate {
    pub node_id: u32,
    /// Creation order; larger is newer.
    pub serial: u64,
    pub identity: ClientIdentity,
    pub application_name: String,
    pub rate: Option<u32>,
}

impl RateCandidate {
    fn owner(&self) -> Owner {
        Owner {
            identity: self.identity.clone(),
            node_id: self.node_id,
            application_name: self.application_name.clone(),
            rate: self.rate.unwrap_or(0),
        }
    }
}

/// The stream currently setting the graph rate.  Its rate can be outside the
/// device's supported list; the rate is displayed, but not forced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Owner {
    pub identity: ClientIdentity,
    pub node_id: u32,
    pub application_name: String,
    pub rate: u32,
}

#[derive(Debug, Clone)]
struct Handover {
    identity: ClientIdentity,
    until: Instant,
}

#[derive(Debug, Clone, Copy)]
struct Pending {
    rate: u32,
    since: Instant,
}

/// Runtime owner state.  It is deliberately not persisted: the mode is a
/// preference, the owner is what is playing now.
#[derive(Debug, Clone, Default)]
pub struct RateFollower {
    owner: Option<Owner>,
    handover: Option<Handover>,
    pending: Option<Pending>,
    last_applied_rate: Option<u32>,
}

impl RateFollower {
    pub fn new() -> Self {
        Self::default()
    }

    /// Forget the owner and any pending decision.  Used when Follow is
    /// selected or the device reconnects; the next observation chooses the
    /// current owner again.
    pub fn reset(&mut self) {
        self.owner = None;
        self.handover = None;
        self.pending = None;
        self.last_applied_rate = None;
    }

    /// Release the owner without forgetting which rate TotalControl last
    /// forced.  A mode change to Auto keeps the graph where it is.
    pub fn clear_owner(&mut self) {
        self.owner = None;
        self.handover = None;
        self.pending = None;
    }

    pub fn owner(&self) -> Option<&Owner> {
        self.owner.as_ref()
    }

    pub fn last_applied_rate(&self) -> Option<u32> {
        self.last_applied_rate
    }

    /// Start a new Follow session from the rate the graph is already at, so
    /// selecting Follow does not rewrite an identical override.
    pub fn seed_applied_rate(&mut self, rate: Option<u32>) {
        self.last_applied_rate = rate.filter(|r| *r > 0);
    }

    /// Observe the graph and return a rate to force once it has been stable.
    ///
    /// `supported` is the connected device's rate list.  A candidate with an
    /// unsupported rate can still own the graph when it already did; it is
    /// shown but not forced.
    pub fn observe(
        &mut self,
        candidates: &[RateCandidate],
        supported: &[u32],
        now: Instant,
    ) -> Option<u32> {
        // A live owner is kept, but a same-client replacement (a track change
        // that recreates the node) takes over immediately.
        if let Some(mut owner) = self.owner.clone() {
            let replacement = candidates
                .iter()
                .filter(|c| c.identity.same_client(&owner.identity))
                .max_by_key(|c| c.serial);
            if let Some(candidate) = replacement {
                owner.node_id = candidate.node_id;
                owner.application_name = candidate.application_name.clone();
                if let Some(rate) = candidate.rate {
                    owner.rate = rate;
                }
                self.owner = Some(owner);
                self.handover = None;
            } else {
                // The owner's node is gone or no longer linked to SW
                // playback 1/2.  Reserve the client for a moment so a player
                // reopening its stream on a track change keeps ownership.
                let identity = owner.identity.clone();
                let keep = self
                    .handover
                    .as_ref()
                    .is_some_and(|h| h.identity.same_client(&identity));
                if !keep {
                    self.handover = Some(Handover {
                        identity,
                        until: now + HANDOVER,
                    });
                }
                self.owner = None;
            }
        }

        // No owner: inherit within the handover window, or expire it and take
        // the most recent remaining candidate.
        if self.owner.is_none() {
            if let Some(handover) = self.handover.clone() {
                let same_client = candidates
                    .iter()
                    .filter(|c| c.identity.same_client(&handover.identity))
                    .max_by_key(|c| c.serial);
                if let Some(candidate) = same_client {
                    self.owner = Some(candidate.owner());
                    self.handover = None;
                } else if now >= handover.until {
                    self.handover = None;
                    if let Some(candidate) = newest_supported(candidates, supported) {
                        self.owner = Some(candidate.owner());
                    }
                }
            } else if let Some(candidate) = newest_supported(candidates, supported) {
                self.owner = Some(candidate.owner());
            }
        }

        let Some(owner) = &self.owner else {
            self.pending = None;
            return None;
        };
        if !supported.contains(&owner.rate) {
            // Keep ownership and the last forced rate; show the unsupported
            // value until the source settles on a supported rate again.
            self.pending = None;
            return None;
        }
        if self.last_applied_rate == Some(owner.rate) {
            self.pending = None;
            return None;
        }
        match self.pending {
            Some(pending) if pending.rate == owner.rate => {
                if now.duration_since(pending.since) >= STABLE {
                    self.last_applied_rate = Some(owner.rate);
                    self.pending = None;
                    return Some(owner.rate);
                }
            }
            _ => {
                self.pending = Some(Pending {
                    rate: owner.rate,
                    since: now,
                });
            }
        }
        None
    }
}

fn newest_supported<'a>(
    candidates: &'a [RateCandidate],
    supported: &[u32],
) -> Option<&'a RateCandidate> {
    candidates
        .iter()
        .filter(|c| c.rate.is_some_and(|r| supported.contains(&r)))
        .max_by_key(|c| c.serial)
}

/// Parse `pw-dump` JSON.  Malformed objects are skipped; `None` only when
/// the document itself is not a PipeWire dump array.
pub fn parse_dump(json: &str) -> Option<GraphSnapshot> {
    let root: serde_json::Value = serde_json::from_str(json).ok()?;
    let objects = root.as_array()?;
    let mut snapshot = GraphSnapshot::default();
    for object in objects {
        match object.get("type").and_then(|v| v.as_str()) {
            Some("PipeWire:Interface:Node") => {
                if let Some(node) = parse_node(object) {
                    snapshot.nodes.push(node);
                }
            }
            Some("PipeWire:Interface:Port") => {
                if let Some(port) = parse_port(object) {
                    snapshot.ports.push(port);
                }
            }
            Some("PipeWire:Interface:Link") => {
                if let Some(link) = parse_link(object) {
                    snapshot.links.push(link);
                }
            }
            _ => {}
        }
    }
    Some(snapshot)
}

/// Read the live graph.  `None` when `pw-dump` is unavailable or fails.
pub fn read_graph() -> Option<GraphSnapshot> {
    let output = std::process::Command::new("pw-dump").output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse_dump(&String::from_utf8_lossy(&output.stdout))
}

/// Candidate streams feeding SW playback 1/2.  `sink` is an optional exact
/// device sink node name; without it the first Fireface/MR18 sink with
/// `playback_AUX0`/`playback_AUX1` is used.
pub fn candidates(snapshot: &GraphSnapshot, sink: Option<&str>) -> Vec<RateCandidate> {
    let Some(sink) = device_sink(snapshot, sink) else {
        return Vec::new();
    };
    // Only SW playback 1/2 counts: a stream linked exclusively to 3/4 or
    // another pair is outside the feature.
    let aux_ports: HashSet<u32> = snapshot
        .ports
        .iter()
        .filter(|port| {
            port.node_id == sink.id
                && port.direction == "in"
                && (port.name == "playback_AUX0" || port.name == "playback_AUX1")
        })
        .map(|port| port.id)
        .collect();
    let mut found = HashMap::<u32, RateCandidate>::new();
    let mut visited = HashSet::new();
    let incoming: Vec<u32> = snapshot
        .links
        .iter()
        .filter(|link| aux_ports.contains(&link.input_port) && link.state != "error")
        .map(|link| link.output_node)
        .collect();
    for node_id in incoming {
        visit_producer(snapshot, node_id, &mut visited, &mut found);
    }
    let mut candidates: Vec<_> = found.into_values().collect();
    candidates.sort_by_key(|c| c.serial);
    candidates
}

fn device_sink<'a>(snapshot: &'a GraphSnapshot, preferred: Option<&str>) -> Option<&'a GraphNode> {
    if let Some(named) = preferred {
        if let Some(node) = snapshot
            .nodes
            .iter()
            .find(|n| n.media_class == "Audio/Sink" && n.name == named)
        {
            return Some(node);
        }
    }
    let has_aux = |id: u32| {
        snapshot.ports.iter().any(|p| {
            p.node_id == id
                && p.direction == "in"
                && (p.name == "playback_AUX0" || p.name == "playback_AUX1")
        })
    };
    let is_fireface = |node: &GraphNode| {
        let name = node.name.to_ascii_lowercase();
        name.contains("fireface") || name.contains("mr18")
    };
    snapshot
        .nodes
        .iter()
        .filter(|n| n.media_class == "Audio/Sink" && has_aux(n.id))
        .find(|n| is_fireface(n))
        .or_else(|| {
            snapshot
                .nodes
                .iter()
                .find(|n| n.media_class == "Audio/Sink" && has_aux(n.id))
        })
}

fn collect_from_sink(
    snapshot: &GraphSnapshot,
    sink: &GraphNode,
    visited: &mut HashSet<u32>,
    found: &mut HashMap<u32, RateCandidate>,
) {
    let incoming: Vec<u32> = snapshot
        .links
        .iter()
        .filter(|l| l.input_node == sink.id && l.state != "error")
        .map(|l| l.output_node)
        .collect();
    for node_id in incoming {
        visit_producer(snapshot, node_id, visited, found);
    }
}

fn visit_producer(
    snapshot: &GraphSnapshot,
    node_id: u32,
    visited: &mut HashSet<u32>,
    found: &mut HashMap<u32, RateCandidate>,
) {
    if !visited.insert(node_id) {
        return;
    }
    let Some(node) = snapshot.nodes.iter().find(|n| n.id == node_id) else {
        return;
    };
    if node.media_class == "Stream/Output/Audio" {
        if is_totalcontrol_loopback(node) {
            // The loopback output is the sink's monitor side, not the rate
            // source.  Continue through the sink it belongs to: that is where
            // the player stream links.
            if let Some(peer) = loopback_sink(snapshot, node) {
                collect_from_sink(snapshot, peer, visited, found);
            }
            return;
        }
        // A suspended stream can still hold its negotiated format and its
        // link (a paused player); only a dormant node without one is ignored.
        if node.rate.is_none() {
            return;
        }
        let candidate = RateCandidate {
            node_id: node.id,
            serial: node.serial,
            identity: ClientIdentity {
                client_id: node.client_id,
                process_id: node.process_id,
                application_name: node
                    .application_name
                    .clone()
                    .unwrap_or_else(|| node.name.clone()),
            },
            application_name: node
                .application_name
                .clone()
                .unwrap_or_else(|| node.name.clone()),
            rate: node.rate,
        };
        found.insert(node.id, candidate);
        return;
    }
    // An intermediate node (filter, loopback sink, ...).  Walk its inputs.
    let incoming: Vec<u32> = snapshot
        .links
        .iter()
        .filter(|l| l.input_node == node.id && l.state != "error")
        .map(|l| l.output_node)
        .collect();
    for upstream in incoming {
        visit_producer(snapshot, upstream, visited, found);
    }
}

fn is_totalcontrol_loopback(node: &GraphNode) -> bool {
    node.name.starts_with("output.totalcontrol-surround")
}

fn loopback_sink<'a>(snapshot: &'a GraphSnapshot, output: &GraphNode) -> Option<&'a GraphNode> {
    let base = output
        .name
        .strip_prefix("output.")
        .unwrap_or(output.name.as_str());
    snapshot
        .nodes
        .iter()
        .find(|n| n.media_class == "Audio/Sink" && n.name == base)
        .or_else(|| {
            snapshot.nodes.iter().find(|n| {
                n.media_class == "Audio/Sink"
                    && n.client_id == output.client_id
                    && n.description.contains("TotalControl Surround")
            })
        })
}

fn parse_node(object: &serde_json::Value) -> Option<GraphNode> {
    let info = object.get("info")?;
    let props = info.get("props")?;
    let id = u32_value(object.get("id"))?;
    Some(GraphNode {
        id,
        name: string_value(props.get("node.name")).unwrap_or_default(),
        description: string_value(props.get("node.description")).unwrap_or_default(),
        media_class: string_value(props.get("media.class")).unwrap_or_default(),
        state: string_value(info.get("state")).unwrap_or_default(),
        client_id: u64_value(props.get("client.id")),
        process_id: u64_value(props.get("application.process.id")),
        application_name: string_value(props.get("application.name")),
        serial: u64_value(props.get("object.serial")).unwrap_or(id as u64),
        rate: node_rate(info),
    })
}

fn node_rate(info: &serde_json::Value) -> Option<u32> {
    if let Some(formats) = info
        .get("params")
        .and_then(|params| params.get("Format"))
        .and_then(|format| format.as_array())
    {
        for format in formats {
            if let Some(rate) = u32_value(format.get("rate")) {
                return Some(rate);
            }
        }
    }
    // `node.rate` is a fraction such as `1/96000`.
    info.get("props")
        .and_then(|props| props.get("node.rate"))
        .and_then(|value| match value {
            serde_json::Value::Number(n) => n.as_u64().map(|v| v as u32),
            serde_json::Value::String(s) => s.split('/').next_back()?.trim().parse().ok(),
            _ => None,
        })
}

fn parse_port(object: &serde_json::Value) -> Option<GraphPort> {
    let props = object.get("info")?.get("props")?;
    Some(GraphPort {
        id: u32_value(object.get("id"))?,
        node_id: u32_value(props.get("node.id"))?,
        name: string_value(props.get("port.name")).unwrap_or_default(),
        direction: string_value(props.get("port.direction")).unwrap_or_default(),
    })
}

fn parse_link(object: &serde_json::Value) -> Option<GraphLink> {
    let info = object.get("info")?;
    Some(GraphLink {
        output_node: u32_value(info.get("output-node-id"))?,
        output_port: u32_value(info.get("output-port-id"))?,
        input_node: u32_value(info.get("input-node-id"))?,
        input_port: u32_value(info.get("input-port-id"))?,
        state: string_value(info.get("state")).unwrap_or_default(),
    })
}

fn string_value(value: Option<&serde_json::Value>) -> Option<String> {
    value?.as_str().map(str::to_owned)
}

fn u32_value(value: Option<&serde_json::Value>) -> Option<u32> {
    u64_value(value).and_then(|v| u32::try_from(v).ok())
}

fn u64_value(value: Option<&serde_json::Value>) -> Option<u64> {
    let value = value?;
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(node_id: u32, serial: u64, client: u64, app: &str, rate: u32) -> RateCandidate {
        RateCandidate {
            node_id,
            serial,
            identity: ClientIdentity {
                client_id: Some(client),
                process_id: None,
                application_name: app.to_string(),
            },
            application_name: app.to_string(),
            rate: Some(rate),
        }
    }

    const RATES: [u32; 9] = [
        32_000, 44_100, 48_000, 64_000, 88_200, 96_000, 128_000, 176_400, 192_000,
    ];

    #[test]
    fn the_first_owner_is_taken_after_its_rate_is_stable() {
        let mut follower = RateFollower::new();
        let now = Instant::now();
        let stream = candidate(1, 10, 7, "Player", 96_000);
        assert_eq!(follower.observe(&[stream.clone()], &RATES, now), None);
        assert_eq!(
            follower.observe(&[stream], &RATES, now + STABLE),
            Some(96_000)
        );
        assert_eq!(follower.owner().unwrap().application_name, "Player");
    }

    #[test]
    fn another_client_does_not_steal_a_live_owner() {
        let mut follower = RateFollower::new();
        let now = Instant::now();
        let owner = candidate(1, 10, 7, "Player", 96_000);
        follower.observe(&[owner.clone()], &RATES, now);
        follower.observe(&[owner.clone()], &RATES, now + STABLE);
        let other = candidate(2, 20, 8, "Browser", 48_000);
        assert_eq!(
            follower.observe(&[owner, other], &RATES, now + STABLE * 2),
            None
        );
        assert_eq!(follower.owner().unwrap().application_name, "Player");
    }

    #[test]
    fn a_same_client_replacement_inherits_across_a_track_change() {
        let mut follower = RateFollower::new();
        let now = Instant::now();
        let old = candidate(1, 10, 7, "Player", 192_000);
        follower.observe(&[old.clone()], &RATES, now);
        follower.observe(&[old], &RATES, now + STABLE);
        let replacement = candidate(2, 20, 7, "Player", 44_100);
        assert_eq!(
            follower.observe(&[replacement.clone()], &RATES, now + STABLE * 2),
            None
        );
        assert_eq!(follower.owner().unwrap().node_id, 2);
        assert_eq!(
            follower.observe(&[replacement], &RATES, now + STABLE * 3),
            Some(44_100)
        );
    }

    #[test]
    fn a_handover_reserves_the_client_then_falls_back_to_the_newest() {
        let mut follower = RateFollower::new();
        let now = Instant::now();
        let owner = candidate(1, 10, 7, "Player", 96_000);
        follower.observe(&[owner.clone()], &RATES, now);
        follower.observe(&[owner], &RATES, now + STABLE);
        // The owner disappears.  A different client must wait out the window.
        let other = candidate(2, 20, 8, "Browser", 44_100);
        let mut newer = candidate(3, 30, 9, "Editor", 48_000);
        newer.rate = Some(48_000);
        assert_eq!(
            follower.observe(&[other.clone(), newer.clone()], &RATES, now + STABLE * 2),
            None
        );
        assert!(follower.owner().is_none());
        assert_eq!(
            follower.observe(&[other, newer], &RATES, now + HANDOVER + STABLE * 2),
            None
        );
        assert_eq!(follower.owner().unwrap().node_id, 3);
        let after = now + HANDOVER + STABLE * 2;
        assert_eq!(
            follower.observe(
                &[candidate(3, 30, 9, "Editor", 48_000)],
                &RATES,
                after + STABLE
            ),
            Some(48_000)
        );
    }

    #[test]
    fn unsupported_rates_do_not_release_or_change_ownership() {
        let mut follower = RateFollower::new();
        let now = Instant::now();
        let owner = candidate(1, 10, 7, "Player", 96_000);
        follower.observe(&[owner.clone()], &RATES, now);
        follower.observe(&[owner], &RATES, now + STABLE);
        let mut odd = candidate(1, 10, 7, "Player", 50_000);
        odd.rate = Some(50_000);
        assert_eq!(follower.observe(&[odd], &RATES, now + STABLE * 2), None);
        assert_eq!(follower.owner().unwrap().rate, 50_000);
        let supported = candidate(1, 10, 7, "Player", 44_100);
        assert_eq!(
            follower.observe(&[supported.clone()], &RATES, now + STABLE * 3),
            None
        );
        assert_eq!(
            follower.observe(&[supported], &RATES, now + STABLE * 4),
            Some(44_100)
        );
    }

    #[test]
    fn the_graph_follows_a_continuous_192_96_441_sequence() {
        let mut follower = RateFollower::new();
        let now = Instant::now();
        for (i, rate) in [192_000, 96_000, 44_100].into_iter().enumerate() {
            let stream = candidate(1, 10, 7, "Player", rate);
            let base = now + STABLE * (i as u32 * 2);
            assert_eq!(follower.observe(&[stream.clone()], &RATES, base), None);
            assert_eq!(
                follower.observe(&[stream], &RATES, base + STABLE),
                Some(rate)
            );
        }
    }

    #[test]
    fn parsing_finds_a_direct_stream_on_sw_playback() {
        let dump = r#"[
          {"id":10,"type":"PipeWire:Interface:Node","info":{"state":"running","props":{
            "media.class":"Audio/Sink","node.name":"alsa_output.usb-RME_Fireface_UCX-00.pro-output-0",
            "client.id":3,"object.serial":10},"params":{}}},
          {"id":11,"type":"PipeWire:Interface:Port","info":{"props":{
            "node.id":10,"port.name":"playback_AUX0","port.direction":"in"}}},
          {"id":12,"type":"PipeWire:Interface:Port","info":{"props":{
            "node.id":10,"port.name":"playback_AUX1","port.direction":"in"}}},
          {"id":13,"type":"PipeWire:Interface:Port","info":{"props":{
            "node.id":10,"port.name":"playback_AUX2","port.direction":"in"}}},
          {"id":20,"type":"PipeWire:Interface:Node","info":{"state":"running","props":{
            "media.class":"Stream/Output/Audio","node.name":"Player",
            "application.name":"Player","client.id":7,"object.serial":20},
            "params":{"Format":[{"mediaType":"audio","rate":96000,"channels":2}]}}},
          {"id":21,"type":"PipeWire:Interface:Port","info":{"props":{
            "node.id":20,"port.name":"output_FL","port.direction":"out"}}},
          {"id":22,"type":"PipeWire:Interface:Port","info":{"props":{
            "node.id":20,"port.name":"output_FR","port.direction":"out"}}},
          {"id":30,"type":"PipeWire:Interface:Link","info":{
            "output-node-id":20,"output-port-id":21,"input-node-id":10,"input-port-id":11,"state":"active"}},
          {"id":31,"type":"PipeWire:Interface:Link","info":{
            "output-node-id":20,"output-port-id":22,"input-node-id":10,"input-port-id":12,"state":"active"}},
          {"id":23,"type":"PipeWire:Interface:Node","info":{"state":"running","props":{
            "media.class":"Stream/Output/Audio","node.name":"Other pair",
            "application.name":"Other pair","client.id":8,"object.serial":23},
            "params":{"Format":[{"mediaType":"audio","rate":44100,"channels":2}]}}},
          {"id":24,"type":"PipeWire:Interface:Port","info":{"props":{
            "node.id":23,"port.name":"output_FL","port.direction":"out"}}},
          {"id":32,"type":"PipeWire:Interface:Link","info":{
            "output-node-id":23,"output-port-id":24,"input-node-id":10,"input-port-id":13,"state":"active"}}
        ]"#;
        let graph = parse_dump(dump).unwrap();
        let found = candidates(&graph, None);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].application_name, "Player");
        assert_eq!(found[0].rate, Some(96_000));
    }

    #[test]
    fn a_surround_sink_is_followed_through_its_player_not_its_loopback() {
        let dump = r#"[
          {"id":10,"type":"PipeWire:Interface:Node","info":{"state":"running","props":{
            "media.class":"Audio/Sink","node.name":"Fireface pro-output-0",
            "client.id":3,"object.serial":10},"params":{}}},
          {"id":11,"type":"PipeWire:Interface:Port","info":{"props":{
            "node.id":10,"port.name":"playback_AUX0","port.direction":"in"}}},
          {"id":12,"type":"PipeWire:Interface:Port","info":{"props":{
            "node.id":10,"port.name":"playback_AUX1","port.direction":"in"}}},
          {"id":20,"type":"PipeWire:Interface:Node","info":{"state":"running","props":{
            "media.class":"Stream/Output/Audio","node.name":"output.totalcontrol-surround51",
            "client.id":8,"object.serial":20},"params":{"Format":[{"rate":48000}]}}},
          {"id":21,"type":"PipeWire:Interface:Port","info":{"props":{
            "node.id":20,"port.name":"output_FL","port.direction":"out"}}},
          {"id":30,"type":"PipeWire:Interface:Link","info":{
            "output-node-id":20,"output-port-id":21,"input-node-id":10,"input-port-id":11,"state":"active"}},
          {"id":40,"type":"PipeWire:Interface:Node","info":{"state":"running","props":{
            "media.class":"Audio/Sink","node.name":"totalcontrol-surround51",
            "node.description":"TotalControl Surround 5.1","client.id":8,"object.serial":40},
            "params":{}}},
          {"id":41,"type":"PipeWire:Interface:Port","info":{"props":{
            "node.id":40,"port.name":"playback_FL","port.direction":"in"}}},
          {"id":50,"type":"PipeWire:Interface:Node","info":{"state":"running","props":{
            "media.class":"Stream/Output/Audio","node.name":"Film Player",
            "application.name":"Film Player","client.id":9,"object.serial":50},
            "params":{"Format":[{"rate":44100}]}}},
          {"id":51,"type":"PipeWire:Interface:Port","info":{"props":{
            "node.id":50,"port.name":"output_FL","port.direction":"out"}}},
          {"id":61,"type":"PipeWire:Interface:Link","info":{
            "output-node-id":50,"output-port-id":51,"input-node-id":40,"input-port-id":41,"state":"active"}}
        ]"#;
        let graph = parse_dump(dump).unwrap();
        let found = candidates(&graph, None);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].application_name, "Film Player");
        assert_eq!(found[0].rate, Some(44_100));
    }
}
