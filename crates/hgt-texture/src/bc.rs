//! BC1 reference decoding, as documented for Direct3D block compression
//! ("Texture Block Compression in Direct3D 11", BC1). Used as the expected
//! result when the GPU samples a BC1 texture.

/// Expands a 5:6:5 colour to 8:8:8 by bit replication.
fn rgb565(c: u16) -> [u8; 3] {
    let r = ((c >> 11) & 0x1F) as u8;
    let g = ((c >> 5) & 0x3F) as u8;
    let b = (c & 0x1F) as u8;
    [(r << 3) | (r >> 2), (g << 2) | (g >> 4), (b << 3) | (b >> 2)]
}

/// Decodes one 8-byte BC1 block to 16 RGBA8 texels in row-major order.
///
/// The interpolated colours use the exact (2a+b)/3 and (a+b)/2 forms with
/// rounding to nearest; hardware is allowed a small error on them, so
/// comparisons against a GPU must use a tolerance for indices 2 and 3.
pub fn decode_bc1_block(block: &[u8; 8]) -> [[u8; 4]; 16] {
    let c0 = u16::from_le_bytes([block[0], block[1]]);
    let c1 = u16::from_le_bytes([block[2], block[3]]);
    let a = rgb565(c0);
    let b = rgb565(c1);
    let mix = |num_a: u32, num_b: u32, den: u32| -> [u8; 4] {
        let f = |x: u8, y: u8| ((num_a * u32::from(x) + num_b * u32::from(y) + den / 2) / den) as u8;
        [f(a[0], b[0]), f(a[1], b[1]), f(a[2], b[2]), 255]
    };
    let palette: [[u8; 4]; 4] = if c0 > c1 {
        [[a[0], a[1], a[2], 255], [b[0], b[1], b[2], 255], mix(2, 1, 3), mix(1, 2, 3)]
    } else {
        [[a[0], a[1], a[2], 255], [b[0], b[1], b[2], 255], mix(1, 1, 2), [0, 0, 0, 0]]
    };
    let bits = u32::from_le_bytes([block[4], block[5], block[6], block[7]]);
    std::array::from_fn(|i| palette[((bits >> (2 * i)) & 3) as usize])
}

/// Encodes a block whose 16 texels all use one palette entry of the two
/// endpoints (`index` 0 or 1). Fixtures only.
pub fn solid_bc1_block(c0: u16, c1: u16, index: u8) -> [u8; 8] {
    assert!(index < 2);
    let bits: u32 = if index == 0 { 0 } else { 0x5555_5555 };
    let mut out = [0u8; 8];
    out[0..2].copy_from_slice(&c0.to_le_bytes());
    out[2..4].copy_from_slice(&c1.to_le_bytes());
    out[4..8].copy_from_slice(&bits.to_le_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints_expand_by_bit_replication() {
        assert_eq!(rgb565(0xFFFF), [255, 255, 255]);
        assert_eq!(rgb565(0x0000), [0, 0, 0]);
        assert_eq!(rgb565(0xF800), [255, 0, 0]);
        assert_eq!(rgb565(0x07E0), [0, 255, 0]);
        assert_eq!(rgb565(0x001F), [0, 0, 255]);
    }

    #[test]
    fn four_colour_mode_interpolates_thirds() {
        // c0 = white > c1 = black: index 2 = 2/3 white, index 3 = 1/3 white
        let mut block = solid_bc1_block(0xFFFF, 0x0000, 0);
        block[4..8].copy_from_slice(&0b11_10_01_00u32.to_le_bytes());
        let t = decode_bc1_block(&block);
        assert_eq!(t[0], [255, 255, 255, 255]);
        assert_eq!(t[1], [0, 0, 0, 255]);
        assert_eq!(t[2], [170, 170, 170, 255]);
        assert_eq!(t[3], [85, 85, 85, 255]);
    }

    #[test]
    fn three_colour_mode_has_transparent_black() {
        // c0 <= c1 selects the 3-colour + transparent palette
        let mut block = solid_bc1_block(0x0000, 0xFFFF, 0);
        block[4..8].copy_from_slice(&0b11_10u32.to_le_bytes());
        let t = decode_bc1_block(&block);
        assert_eq!(t[0], [128, 128, 128, 255]);
        assert_eq!(t[1], [0, 0, 0, 0]);
    }

    #[test]
    fn solid_blocks_decode_to_one_colour() {
        let t = decode_bc1_block(&solid_bc1_block(0xF800, 0x001F, 1));
        assert!(t.iter().all(|p| *p == [0, 0, 255, 255]));
    }
}
