use super::*;

#[test]
fn stream_starts_at_mouth_rises_detaches_on_shutoff_and_tracks_pulses() {
    let mut track = SmokeTrack::default();
    let eye = Vec3::ZERO;
    let mouth = Vec3::new(0.0, 2.0, -5.0);
    let key = source_key(TAG_PROCESSOR, IVec3::new(0, 2, -5), 0);
    let cone = ViewCone::new(0.0, 0.0, 0.0, 1.0); // 0 fov disables culling

    let mask_of = |t: &SmokeTrack| t.out[7];

    // Cold start: turning on lights only bit 0 (right at the mouth), never the whole plume at once.
    track.begin_frame(0.016, 0.0);
    track.record(key, mouth, 0.5, 0.25, 1.0, KIND_CHIMNEY);
    track.finish(eye, 100.0, &cone, |_| DAYLIGHT, |_| true);
    assert_eq!(track.out.len(), SMOKE_FLOATS);
    assert_eq!(mask_of(&track), 0x0000_0001, "starts only at the chimney lip");

    // Run for ~half a plume lifetime: stream rises progressively from bit 0 upward.
    for _ in 0..160 {
        track.begin_frame(0.016, 0.0);
        track.record(key, mouth, 0.5, 0.25, 1.0, KIND_CHIMNEY);
        track.finish(eye, 100.0, &cone, |_| DAYLIGHT, |_| true);
    }
    let mid = mask_of(&track);
    assert!(mid > 0x0000_00FF && mid < u32::MAX, "mid-rise mask={mid:#010x}");

    // Shut the machine off: bit 0 clears immediately while the existing detached stream keeps flowing!
    track.begin_frame(0.016, 0.0);
    track.finish(eye, 100.0, &cone, |_| DAYLIGHT, |_| true);
    let shut = mask_of(&track);
    assert_eq!(shut & 1, 0, "mouth cuts off immediately on shutoff");
    assert_ne!(shut, 0, "existing stream remains aloft");

    // Toggle back on after a brief pause: creates two distinct rising packets separated by zero bits!
    for _ in 0..40 {
        track.begin_frame(0.016, 0.0);
        track.finish(eye, 100.0, &cone, |_| DAYLIGHT, |_| true);
    }
    for _ in 0..30 {
        track.begin_frame(0.016, 0.0);
        track.record(key, mouth, 0.5, 0.25, 1.0, KIND_CHIMNEY);
        track.finish(eye, 100.0, &cone, |_| DAYLIGHT, |_| true);
    }
    let pulsed = mask_of(&track);
    assert_eq!(pulsed & 1, 1, "new packet active at mouth");
    assert!(pulsed.trailing_ones() < 32 - pulsed.leading_zeros(), "zero gap separates the two packets: {pulsed:#010x}");

    // Leave off for over a full lifetime: detached tail finishes dissipating and is pruned cleanly.
    for _ in 0..400 {
        track.begin_frame(0.016, 0.0);
        track.finish(eye, 100.0, &cone, |_| DAYLIGHT, |_| true);
    }
    assert!(track.out.is_empty() && track.entries.is_empty(), "finished stream pruned to zero work");
}

#[test]
fn cap_emitters_keeps_nearest_up_to_limit() {
    let mut out = Vec::new();
    for i in 0..(MAX_EMITTERS + 40) {
        let d = (MAX_EMITTERS + 40 - i) as f64;
        push_raw(&mut out, Vec3::new(d, 0.0, 0.0), 0.5, 0.1, 1.0, DAYLIGHT, KIND_CHIMNEY, u32::MAX);
    }
    cap_emitters(&mut out);
    assert_eq!(out.len(), MAX_EMITTERS * SMOKE_FLOATS);
    let min_x = out.chunks_exact(SMOKE_FLOATS).map(|r| f32::from_bits(r[0])).fold(f32::INFINITY, f32::min);
    let max_x = out.chunks_exact(SMOKE_FLOATS).map(|r| f32::from_bits(r[0])).fold(0.0_f32, f32::max);
    assert_eq!(min_x, 1.0);
    assert_eq!(max_x, MAX_EMITTERS as f32);
}
