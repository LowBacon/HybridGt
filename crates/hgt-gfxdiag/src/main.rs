//! `hgt-gfxdiag`: runs every check and prints one line per check.
//! Exit code 1 if any check failed (skips do not fail the run). After a
//! device removal the remaining checks are reported as not run, since their
//! results would only repeat the removal.

use hgt_gfxdiag::*;

fn main() {
    let gpu = match Gpu::from_env() {
        Ok(g) => g,
        Err(e) => {
            println!("adapter: {e}");
            std::process::exit(if matches!(e, Error::Skip(_)) { 0 } else { 1 });
        }
    };
    let i = &gpu.info;
    println!(
        "adapter: {} (software: {}, VRAM {} MiB, SM {}.{}, mesh tier {}, debug layer: {})",
        i.name,
        i.software,
        i.dedicated_video_memory >> 20,
        i.highest_shader_model >> 4,
        i.highest_shader_model & 0xF,
        i.mesh_shader_tier,
        debug_layer_requested()
    );
    let mut failed = 0;
    let mut removed = false;
    let mut report = |name: &str, run: &dyn Fn() -> Result<Timings>| {
        if removed {
            println!("NOT RUN {name}: device removed earlier");
            return;
        }
        match run() {
            Ok(t) => println!("PASS {name} {}", fmt_timings(&t)),
            Err(Error::Skip(s)) => println!("SKIP {name}: {s}"),
            Err(Error::Fail(s)) => {
                failed += 1;
                removed = s.contains("device removed") || s.contains("0x887A0005") || s.contains("0x887A0006");
                println!("FAIL {name}: {s}");
            }
        }
    };
    match max_view_heap(&gpu, 1 << 16, 1 << 24) {
        Ok(n) => println!("INFO largest shader-visible view heap: {n} slots"),
        Err(e) => println!("INFO largest shader-visible view heap: {e}"),
    }
    println!("INFO GPU_UPLOAD heap supported: {}", gpu_upload_heap_supported(&gpu));
    for p in [Placement::SystemL0, Placement::Video, Placement::GpuUpload] {
        match bench_buffer_read(&gpu, p, 256) {
            Ok(d) => println!(
                "INFO read 256 MiB from {p:?}: {:.2} ms ({:.1} GB/s)",
                d.as_secs_f64() * 1e3,
                0.268435456 / d.as_secs_f64()
            ),
            Err(e) => println!("INFO read 256 MiB from {p:?}: {e}"),
        }
    }
    report("clear", &|| check_clear(&gpu));
    report("bc1-texture", &|| check_bc1_texture(&gpu));
    let stale = |producer| CountSource::Produced { value: 2, stale: u32::MAX, producer };
    use CountGuard::{Clamp, None as Unguarded};
    let cases: [(&str, CountSource, u32, CountGuard, u32); 10] = [
        ("indirect uploaded count 3 of 8", CountSource::Uploaded(3), 8, Unguarded, 3),
        ("indirect uploaded count 0", CountSource::Uploaded(0), 8, Unguarded, 0),
        ("indirect produced count 2", stale(Producer::Runs), 8, Unguarded, 2),
        ("indirect producer skipped, outputs zeroed", stale(Producer::SkippedZeroed), 8, Unguarded, 0),
        ("indirect count 0xFFFFFFFF, max 5, unguarded", CountSource::Uploaded(u32::MAX), 5, Unguarded, 5),
        ("indirect producer skipped, stale count, unguarded", stale(Producer::SkippedLeave), 8, Unguarded, 8),
        ("indirect count 3 of 8, clamp guard", CountSource::Uploaded(3), 8, Clamp, 3),
        ("indirect count 0xFFFFFFFF, max 5, clamp guard", CountSource::Uploaded(u32::MAX), 5, Clamp, 5),
        ("indirect produced count 2, clamp guard", stale(Producer::Runs), 8, Clamp, 2),
        ("indirect producer skipped, stale count, clamp guard", stale(Producer::SkippedLeave), 8, Clamp, 8),
    ];
    for (name, source, max, guard, want) in cases {
        report(name, &|| {
            indirect_draws(&gpu, source, max, guard).and_then(|(n, t)| {
                if n == want { Ok(t) } else { Err(Error::Fail(format!("{n} draws ran, expected {want}"))) }
            })
        });
    }
    std::process::exit(if failed > 0 { 1 } else { 0 });
}

fn fmt_timings(t: &Timings) -> String {
    t.0.iter().map(|(k, d)| format!("{k}={:.2}ms", d.as_secs_f64() * 1e3)).collect::<Vec<_>>().join(" ")
}
