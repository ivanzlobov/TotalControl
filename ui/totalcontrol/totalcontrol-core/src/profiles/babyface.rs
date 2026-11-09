//! Babyface (first generation) device profile.
//!
//! Ten hardware inputs (two analog, eight optical), six output pairs and
//! twelve software playback channels.  The optical port carries ADAT or
//! S/PDIF; in S/PDIF mode the first optical pair is S/PDIF L/R.  The
//! output blocks are in device order, so the display order is the
//! identity.

use crate::channel::ChannelType;
use crate::profile::{DeviceProfile, InputSpec, OutputPairSpec};

const fn input(name: &'static str, channel_type: ChannelType) -> InputSpec {
    InputSpec { name, channel_type }
}

const fn pair(left: &'static str, right: &'static str) -> OutputPairSpec {
    OutputPairSpec { left, right }
}

const PAIRS: &[OutputPairSpec] = &[
    pair("AN1", "AN2"),
    pair("PH3", "PH4"),
    pair("AS1", "AS2"),
    pair("ADAT3", "ADAT4"),
    pair("ADAT5", "ADAT6"),
    pair("ADAT7", "ADAT8"),
];

pub const PROFILE: DeviceProfile = DeviceProfile {
    card_substring: "Babyface",
    model_name: "Babyface",
    inputs: &[
        input("AN1", ChannelType::Mic),
        input("AN2", ChannelType::Mic),
        input("AS1", ChannelType::ADAT),
        input("AS2", ChannelType::ADAT),
        input("ADAT3", ChannelType::ADAT),
        input("ADAT4", ChannelType::ADAT),
        input("ADAT5", ChannelType::ADAT),
        input("ADAT6", ChannelType::ADAT),
        input("ADAT7", ChannelType::ADAT),
        input("ADAT8", ChannelType::ADAT),
    ],
    outputs: PAIRS,
    playback_pairs: PAIRS,
    output_order: &[0, 1, 2, 3, 4, 5],
};

#[cfg(test)]
mod tests {
    use super::PROFILE;

    #[test]
    fn topology_is_ten_in_twelve_out() {
        assert_eq!(PROFILE.input_count(), 10);
        assert_eq!(PROFILE.output_pair_count(), 6);
        assert_eq!(PROFILE.playback_pairs.len(), 6);
    }
}
