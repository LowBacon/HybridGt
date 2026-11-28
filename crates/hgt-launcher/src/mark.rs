// Shared with build.rs (through `include!`), which draws the executable's icon from it.

/// The brand mark as RGBA pixels: a sunset disc with dark bands in its lower half.
pub fn mark_pixels(n: u32) -> (Vec<u8>, u32) {
    let mut rgba = Vec::with_capacity((n * n * 4) as usize);
    let c = n as f32 / 2.0;
    let lerp = |a: [f32; 3], b: [f32; 3], t: f32| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
    for y in 0..n {
        for x in 0..n {
            let (fx, fy) = (x as f32 + 0.5 - c, y as f32 + 0.5 - c);
            let r = (fx * fx + fy * fy).sqrt();
            let t = y as f32 / n as f32;
            let col = if t < 0.5 { lerp([255.0, 170.0, 72.0], [255.0, 79.0, 139.0], t * 2.0) } else { lerp([255.0, 79.0, 139.0], [124.0, 58.0, 237.0], (t - 0.5) * 2.0) };
            let band = t > 0.55 && ((t - 0.55) * n as f32 / 4.0) as u32 % 2 == 1;
            let alpha = if band { 0.0 } else { (c - 0.5 - r).clamp(0.0, 1.0) };
            rgba.extend([col[0] as u8, col[1] as u8, col[2] as u8, (alpha * 255.0) as u8]);
        }
    }
    (rgba, n)
}
