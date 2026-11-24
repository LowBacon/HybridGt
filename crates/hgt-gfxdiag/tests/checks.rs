//! The harness's checks as tests. They run on WARP unless
//! `HYBRIDGT_ADAPTER=hw`. A check that cannot run prints `SKIPPED` and passes;
//! report skips, never count them as coverage.

use hgt_gfxdiag::*;

fn gpu() -> Option<Gpu> {
    match Gpu::from_env() {
        Ok(g) => Some(g),
        Err(Error::Skip(s)) => {
            eprintln!("SKIPPED: {s}");
            None
        }
        Err(e) => panic!("{e}"),
    }
}

fn expect<T>(name: &str, r: Result<T>) -> Option<T> {
    match r {
        Ok(v) => Some(v),
        Err(Error::Skip(s)) => {
            eprintln!("SKIPPED {name}: {s}");
            None
        }
        Err(e) => panic!("{name}: {e}"),
    }
}

fn draws(name: &str, source: CountSource, max: u32) -> Option<u32> {
    guarded(name, source, max, CountGuard::None)
}

fn guarded(name: &str, source: CountSource, max: u32, guard: CountGuard) -> Option<u32> {
    let gpu = gpu()?;
    expect(name, indirect_draws(&gpu, source, max, guard)).map(|(n, _)| n)
}

#[test]
fn clear_reads_back() {
    let Some(gpu) = gpu() else { return };
    expect("clear", check_clear(&gpu));
}

#[test]
fn bc1_texture_matches_reference() {
    let Some(gpu) = gpu() else { return };
    expect("bc1", check_bc1_texture(&gpu));
}

#[test]
fn indirect_uploaded_count_is_used() {
    if let Some(n) = draws("uploaded 3", CountSource::Uploaded(3), 8) {
        assert_eq!(n, 3);
    }
    if let Some(n) = draws("uploaded 0", CountSource::Uploaded(0), 8) {
        assert_eq!(n, 0);
    }
}

#[test]
fn indirect_count_is_clamped_to_max() {
    if let Some(n) = draws("uploaded max", CountSource::Uploaded(u32::MAX), 5) {
        assert_eq!(n, 5);
    }
}

#[test]
fn indirect_count_from_producer_dispatch() {
    let s = CountSource::Produced { value: 2, stale: u32::MAX, producer: Producer::Runs };
    if let Some(n) = draws("produced", s, 8) {
        assert_eq!(n, 2);
    }
}

/// The failure mode GTA6-PC-Native's README describes for skipped world
/// draws: the consumer reads a stale count. Here it runs up to `max` draws.
#[test]
fn skipped_producer_leaves_a_stale_count() {
    let s = CountSource::Produced { value: 2, stale: u32::MAX, producer: Producer::SkippedLeave };
    if let Some(n) = draws("skipped leave", s, 8) {
        assert_eq!(n, 8);
    }
}

/// The policy that keeps the consumer bounded: zero what the skipped
/// producer would have written.
#[test]
fn skipped_producer_with_zeroed_outputs_draws_nothing() {
    let s = CountSource::Produced { value: 2, stale: u32::MAX, producer: Producer::SkippedZeroed };
    if let Some(n) = draws("skipped zeroed", s, 8) {
        assert_eq!(n, 0);
    }
}

#[test]
fn clamp_guard_keeps_in_range_counts() {
    if let Some(n) = guarded("guard 3", CountSource::Uploaded(3), 8, CountGuard::Clamp) {
        assert_eq!(n, 3);
    }
    let s = CountSource::Produced { value: 2, stale: u32::MAX, producer: Producer::Runs };
    if let Some(n) = guarded("guard produced", s, 8, CountGuard::Clamp) {
        assert_eq!(n, 2);
    }
}

#[test]
fn clamp_guard_bounds_garbage_counts() {
    if let Some(n) = guarded("guard max", CountSource::Uploaded(u32::MAX), 5, CountGuard::Clamp) {
        assert_eq!(n, 5);
    }
    let s = CountSource::Produced { value: 2, stale: u32::MAX, producer: Producer::SkippedLeave };
    if let Some(n) = guarded("guard stale", s, 8, CountGuard::Clamp) {
        assert_eq!(n, 8);
    }
}

#[test]
fn hang_prone_cases_are_identified() {
    use CountGuard::*;
    assert!(may_hang(CountSource::Uploaded(u32::MAX), 5, None));
    assert!(!may_hang(CountSource::Uploaded(u32::MAX), 5, Clamp));
    assert!(!may_hang(CountSource::Uploaded(5), 5, None));
    let stale = |producer| CountSource::Produced { value: 2, stale: u32::MAX, producer };
    assert!(may_hang(stale(Producer::SkippedLeave), 8, None));
    assert!(!may_hang(stale(Producer::SkippedZeroed), 8, None));
    assert!(!may_hang(stale(Producer::Runs), 8, None));
}

#[test]
fn wave64_two_half_emulation_matches_console() {
    let Some(gpu) = gpu() else { return };
    let Some(r) = expect("wave64", check_wave64(&gpu)) else { return };
    eprintln!("wave64: {r:?}");
    assert!(r.emulated_matches_console, "{r:?}");
    // Native waves narrower than 64 cannot give the console's answer.
    assert_eq!(r.native_matches_console, r.host_lanes >= 64, "{r:?}");
}

#[test]
fn wave64_two_half_emulation_matches_console_in_mesh_shader() {
    let Some(gpu) = gpu() else { return };
    let Some(r) = expect("wave64 mesh", check_wave64_stage(&gpu, Wave64Stage::Mesh)) else { return };
    eprintln!("wave64 mesh: {r:?}");
    assert!(r.emulated_matches_console, "{r:?}");
    assert_eq!(r.native_matches_console, r.host_lanes >= 64, "{r:?}");
}
