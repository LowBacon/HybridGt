//! HLSL for the checks, compiled with DXC at run time.
//!
//! DXC is found through `HYBRIDGT_DXC`, else `dxc` on `PATH`. A missing DXC
//! makes the checks report "skipped", never "passed".

use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use crate::Error;

pub const FULLSCREEN_VS: &str = r#"
float4 main(uint id : SV_VertexID) : SV_Position {
    float2 uv = float2((id << 1) & 2, id & 2);
    return float4(uv * float2(2, -2) + float2(-1, 1), 0, 1);
}
"#;

/// One count step per draw; drawn with additive blending, so the red
/// channel counts the draws that ran.
pub const COUNT_PS: &str = r#"
float4 main() : SV_Target { return float4(1.0 / 255.0, 0, 0, 0); }
"#;

/// Exact texel fetch, no filtering.
pub const LOAD_PS: &str = r#"
Texture2D<float4> t0 : register(t0);
float4 main(float4 pos : SV_Position) : SV_Target { return t0.Load(int3(pos.xy, 0)); }
"#;

/// The producer of an indirect draw count: writes `value` into u0[0].
pub const COUNT_CS: &str = r#"
RWByteAddressBuffer u0 : register(u0);
cbuffer c : register(b0) { uint value; };
[numthreads(1, 1, 1)] void main() { u0.Store(0, value); }
"#;

/// The count guard: u1[0] = min(u0[0], max).
pub const CLAMP_CS: &str = r#"
RWByteAddressBuffer u0 : register(u0);
RWByteAddressBuffer u1 : register(u1);
cbuffer c : register(b0) { uint maxCount; };
[numthreads(1, 1, 1)] void main() { u1.Store(0, min(u0.Load(0), maxCount)); }
"#;

pub fn dxc_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("HYBRIDGT_DXC") {
        return Some(PathBuf::from(p));
    }
    let ok = Command::new("dxc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false);
    ok.then(|| PathBuf::from("dxc"))
}

/// Compiles `source` for `profile` (e.g. `ps_6_0`), returning DXIL and the time taken.
pub fn compile(source: &str, profile: &str) -> Result<(Vec<u8>, Duration), Error> {
    let dxc = dxc_path().ok_or_else(|| Error::Skip("no DXC (set HYBRIDGT_DXC or put dxc on PATH)".into()))?;
    let dir = std::env::temp_dir().join(format!("hgt-gfxdiag-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| Error::Fail(e.to_string()))?;
    // Unique per call: tests compile the same source on several threads at
    // once, and a shared name let one read another's half-written output.
    static CALLS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = CALLS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let stem = format!("{profile}-{:016x}-{n}", fnv1a(source.as_bytes()));
    let src = dir.join(format!("{stem}.hlsl"));
    let out = dir.join(format!("{stem}.dxil"));
    std::fs::write(&src, source).map_err(|e| Error::Fail(e.to_string()))?;
    let started = Instant::now();
    let res = Command::new(&dxc)
        .args(["-nologo", "-E", "main", "-T", profile, "-Fo"])
        .arg(&out)
        .arg(&src)
        .output()
        .map_err(|e| Error::Fail(format!("running {}: {e}", dxc.display())))?;
    let took = started.elapsed();
    if !res.status.success() {
        return Err(Error::Fail(format!("dxc {profile}: {}", String::from_utf8_lossy(&res.stderr))));
    }
    let bytes = std::fs::read(&out).map_err(|e| Error::Fail(e.to_string()))?;
    Ok((bytes, took))
}

fn fnv1a(b: &[u8]) -> u64 {
    b.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &x| (h ^ u64::from(x)).wrapping_mul(0x100_0000_01b3))
}

/// Wave64 semantics probe. One 64-thread group; lane predicate `t % 3 == 0`.
/// Per thread: host lane count, host lane index, native ballot .x/.y.
/// Thread 0 also stores the group-wide 64-bit ballot built from two 32-bit
/// halves in group-shared memory: the console's wave64 answer, whatever the
/// host's wave size.
pub const WAVE64_CS: &str = r#"
RWByteAddressBuffer o : register(u0);
groupshared uint g[2];
[numthreads(64, 1, 1)] void main(uint t : SV_GroupIndex) {
    bool p = (t % 3) == 0;
    uint4 b = WaveActiveBallot(p);
    if (t < 2) g[t] = 0;
    GroupMemoryBarrierWithGroupSync();
    if (p) InterlockedOr(g[t / 32], 1u << (t % 32));
    GroupMemoryBarrierWithGroupSync();
    o.Store4(t * 16, uint4(WaveGetLaneCount(), WaveGetLaneIndex(), b.x, b.y));
    if (t == 0) o.Store2(64 * 16, uint2(g[0], g[1]));
}
"#;
