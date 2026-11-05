//! Request the UCX state dump and compare it with a saved scene.
//!
//!   cargo run -p totalcontrol-core --example ucx-dump-compare --features alsa -- <scene.json> [card]
//!
//! Read-only: the device is not written.

#[cfg(feature = "alsa")]
fn main() {
    use std::time::Duration;
    use totalcontrol_core::fireface::{Fireface, Model, UcxDump};
    use totalcontrol_core::scene::Scene;

    let mut args = std::env::args().skip(1);
    let path = args.next().expect("scene.json path");
    let card: i32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(2);
    let found = Fireface::enumerate()
        .into_iter()
        .find(|info| info.card == card && info.model == Model::Ucx)
        .expect("no UCX on that card");
    let mut dev = Fireface::open_identity(&found.identity()).expect("open UCX");
    dev.capture_state_dump(Duration::from_millis(2500))
        .expect("state dump");
    let mut scene = Scene::from_json(&std::fs::read_to_string(&path).expect("read scene"))
        .expect("parse scene");
    let dump = UcxDump::new(dev.state_words().clone());
    let diff = dump.apply_to_scene(&mut scene);
    println!("dump words: {}", dev.state_words().len());
    println!("differences: {}", diff.len());
    for item in diff {
        println!("  {item}");
    }
}

#[cfg(not(feature = "alsa"))]
fn main() {
    eprintln!("build with --features alsa");
}
