//! Texture data paths for HybridGt.
//!
//! - [`bcpack`]: BCPack unpacking behind an adapter, with strict failure
//!   semantics: a destination is written only when the whole stream decoded,
//!   and every request is accounted as filled or not filled.
//! - [`bc`]: CPU reference decoding of BCn blocks (public DirectX block
//!   formats), used to check GPU texture paths.

pub mod bc;
pub mod bcpack;
