//! An independent D3D12 diagnostic harness.
//!
//! Every check is a small, known GPU workload whose result is read back and
//! compared with an expected value. The checks run on WARP (the software
//! adapter) by default, so a broken check can never reset a real GPU driver;
//! `HYBRIDGT_ADAPTER=hw` runs them on the first hardware adapter instead.
//!
//! The checks target the paths the console title's world needs and its menus
//! do not (see docs/COMPATIBILITY_MATRIX.md): indirect draws whose count is
//! produced on the GPU, and block-compressed textures uploaded from CPU
//! memory.

pub mod shaders;

use std::ffi::c_void;
use std::fmt;
use std::mem::ManuallyDrop;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::core::Interface;

/// A check's outcome. `Skip` means it could not run here (no DXC, no adapter
/// feature); it is never counted as a pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Skip(String),
    Fail(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Skip(s) => write!(f, "skipped: {s}"),
            Error::Fail(s) => write!(f, "failed: {s}"),
        }
    }
}

impl From<windows::core::Error> for Error {
    fn from(e: windows::core::Error) -> Error {
        Error::Fail(format!("{e}"))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Measurements a check took along the way.
#[derive(Debug, Default, Clone)]
pub struct Timings(pub Vec<(&'static str, Duration)>);

impl Timings {
    pub fn add(&mut self, what: &'static str, d: Duration) {
        self.0.push((what, d));
    }
}

/// What the adapter offers, as far as the checks care.
#[derive(Debug, Clone)]
pub struct AdapterInfo {
    pub name: String,
    pub software: bool,
    pub dedicated_video_memory: u64,
    pub highest_shader_model: u32,
    pub mesh_shader_tier: i32,
}

pub struct Gpu {
    pub device: ID3D12Device,
    pub queue: ID3D12CommandQueue,
    fence: ID3D12Fence,
    fence_value: std::cell::Cell<u64>,
    pub info: AdapterInfo,
}

/// How long [`Gpu::run`] waits for the GPU before calling it hung. Bounded,
/// so a broken workload fails the check instead of hanging the harness.
pub const GPU_TIMEOUT: Duration = Duration::from_secs(20);

impl Gpu {
    /// The adapter named by `HYBRIDGT_ADAPTER`: `warp` (default) or `hw`.
    pub fn from_env() -> Result<Gpu> {
        let hw = std::env::var("HYBRIDGT_ADAPTER").map(|v| v.eq_ignore_ascii_case("hw")).unwrap_or(false);
        Gpu::new(hw)
    }

    pub fn new(hardware: bool) -> Result<Gpu> {
        // SAFETY: plain D3D12/DXGI object creation; every out pointer is a local.
        unsafe {
            if debug_layer_requested() {
                let mut debug: Option<ID3D12Debug> = None;
                D3D12GetDebugInterface(&mut debug)
                    .map_err(|e| Error::Skip(format!("HYBRIDGT_D3D12_DEBUG=1 but no debug layer (Graphics Tools): {e}")))?;
                debug.ok_or_else(|| Error::Fail("no debug interface".into()))?.EnableDebugLayer();
            }
            let factory: IDXGIFactory4 = CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0))?;
            let adapter: IDXGIAdapter1 = if hardware {
                let mut found = None;
                for i in 0.. {
                    let Ok(a) = factory.EnumAdapters1(i) else { break };
                    let d = a.GetDesc1()?;
                    if d.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 == 0 {
                        found = Some(a);
                        break;
                    }
                }
                found.ok_or_else(|| Error::Skip("no hardware adapter".into()))?
            } else {
                factory.EnumWarpAdapter()?
            };
            let desc = adapter.GetDesc1()?;
            let name = String::from_utf16_lossy(&desc.Description).trim_end_matches('\0').to_string();
            let mut device: Option<ID3D12Device> = None;
            D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_12_0, &mut device)
                .map_err(|e| Error::Skip(format!("no D3D12 device on {name}: {e}")))?;
            let device = device.ok_or_else(|| Error::Fail("D3D12CreateDevice gave no device".into()))?;

            let mut highest = 0u32;
            for sm in (0x60..=0x69).rev() {
                let mut d = D3D12_FEATURE_DATA_SHADER_MODEL { HighestShaderModel: D3D_SHADER_MODEL(sm) };
                if device
                    .CheckFeatureSupport(
                        D3D12_FEATURE_SHADER_MODEL,
                        &mut d as *mut _ as *mut c_void,
                        size_of_val(&d) as u32,
                    )
                    .is_ok()
                {
                    highest = d.HighestShaderModel.0 as u32;
                    break;
                }
            }
            let mut o7 = D3D12_FEATURE_DATA_D3D12_OPTIONS7::default();
            let mesh = if device
                .CheckFeatureSupport(D3D12_FEATURE_D3D12_OPTIONS7, &mut o7 as *mut _ as *mut c_void, size_of_val(&o7) as u32)
                .is_ok()
            {
                o7.MeshShaderTier.0
            } else {
                0
            };

            let queue: ID3D12CommandQueue = device.CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
                Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                ..Default::default()
            })?;
            let fence: ID3D12Fence = device.CreateFence(0, D3D12_FENCE_FLAG_NONE)?;
            Ok(Gpu {
                device,
                queue,
                fence,
                fence_value: std::cell::Cell::new(0),
                info: AdapterInfo {
                    name,
                    software: desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0,
                    dedicated_video_memory: desc.DedicatedVideoMemory as u64,
                    highest_shader_model: highest,
                    mesh_shader_tier: mesh,
                },
            })
        }
    }

    /// Records one command list, runs it and waits (bounded) for it.
    pub fn run(&self, record: impl FnOnce(&ID3D12GraphicsCommandList) -> Result<()>) -> Result<Duration> {
        // SAFETY: the allocator and list live until the fence says the GPU is done.
        unsafe {
            let alloc: ID3D12CommandAllocator = self.device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)?;
            let list: ID3D12GraphicsCommandList =
                self.device.CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, &alloc, None)?;
            record(&list)?;
            list.Close()?;
            let started = Instant::now();
            self.queue.ExecuteCommandLists(&[Some(list.cast::<ID3D12CommandList>()?)]);
            let v = self.fence_value.get() + 1;
            self.fence_value.set(v);
            self.queue.Signal(&self.fence, v)?;
            while self.fence.GetCompletedValue() < v {
                if let Err(e) = self.device.GetDeviceRemovedReason() {
                    return Err(Error::Fail(format!("device removed: {e}")));
                }
                if started.elapsed() > GPU_TIMEOUT {
                    return Err(Error::Fail(format!("GPU did not finish in {GPU_TIMEOUT:?}")));
                }
                std::thread::yield_now();
            }
            if let Err(e) = self.device.GetDeviceRemovedReason() {
                return Err(Error::Fail(format!("device removed: {e}")));
            }
            let took = started.elapsed();
            let errors = self.debug_errors();
            if !errors.is_empty() {
                return Err(Error::Fail(format!("debug layer: {}", errors.join(" | "))));
            }
            Ok(took)
        }
    }

    /// Errors and corruption messages the debug layer stored since the last
    /// call (none without `HYBRIDGT_D3D12_DEBUG=1`). Warnings are printed.
    pub fn debug_errors(&self) -> Vec<String> {
        let Ok(q) = self.device.cast::<ID3D12InfoQueue>() else { return Vec::new() };
        let mut out = Vec::new();
        // SAFETY: each message is read into a buffer of the size the queue reports.
        unsafe {
            for i in 0..q.GetNumStoredMessages() {
                let mut len = 0usize;
                if q.GetMessage(i, None, &mut len).is_err() || len == 0 {
                    continue;
                }
                let mut buf = vec![0u64; len.div_ceil(8)];
                let msg = buf.as_mut_ptr().cast::<D3D12_MESSAGE>();
                if q.GetMessage(i, Some(msg), &mut len).is_err() {
                    continue;
                }
                let m = &*msg;
                let text = std::slice::from_raw_parts(m.pDescription, m.DescriptionByteLength.saturating_sub(1));
                let text = String::from_utf8_lossy(text).into_owned();
                if m.Severity.0 <= D3D12_MESSAGE_SEVERITY_ERROR.0 {
                    out.push(text);
                } else if m.Severity == D3D12_MESSAGE_SEVERITY_WARNING {
                    eprintln!("debug layer warning: {text}");
                }
            }
            q.ClearStoredMessages();
        }
        out
    }

    pub fn buffer(&self, heap: D3D12_HEAP_TYPE, size: u64, flags: D3D12_RESOURCE_FLAGS, state: D3D12_RESOURCE_STATES) -> Result<ID3D12Resource> {
        let desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
            Width: size,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            Flags: flags,
            ..Default::default()
        };
        self.committed(heap, &desc, state, None)
    }

    pub fn committed(
        &self,
        heap: D3D12_HEAP_TYPE,
        desc: &D3D12_RESOURCE_DESC,
        state: D3D12_RESOURCE_STATES,
        clear: Option<&D3D12_CLEAR_VALUE>,
    ) -> Result<ID3D12Resource> {
        let props = D3D12_HEAP_PROPERTIES { Type: heap, ..Default::default() };
        let mut res: Option<ID3D12Resource> = None;
        // SAFETY: descriptors are locals; the out pointer is a local Option.
        unsafe {
            self.device.CreateCommittedResource(
                &props,
                D3D12_HEAP_FLAG_NONE,
                desc,
                state,
                clear.map(|c| c as *const _),
                &mut res,
            )?;
        }
        res.ok_or_else(|| Error::Fail("CreateCommittedResource gave no resource".into()))
    }

    /// An upload buffer holding `bytes`.
    pub fn upload(&self, bytes: &[u8]) -> Result<ID3D12Resource> {
        let buf = self.buffer(D3D12_HEAP_TYPE_UPLOAD, bytes.len().max(1) as u64, D3D12_RESOURCE_FLAG_NONE, D3D12_RESOURCE_STATE_GENERIC_READ)?;
        // SAFETY: the mapping covers the whole buffer, which is at least `bytes.len()` long.
        unsafe {
            let mut p: *mut c_void = std::ptr::null_mut();
            buf.Map(0, None, Some(&mut p))?;
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), p.cast::<u8>(), bytes.len());
            buf.Unmap(0, None);
        }
        Ok(buf)
    }

    pub fn root_signature(&self, desc: &D3D12_ROOT_SIGNATURE_DESC) -> Result<ID3D12RootSignature> {
        // SAFETY: the blob outlives the slice made from it.
        unsafe {
            let mut blob: Option<ID3DBlob> = None;
            let mut err: Option<ID3DBlob> = None;
            if let Err(e) = D3D12SerializeRootSignature(desc, D3D_ROOT_SIGNATURE_VERSION_1, &mut blob, Some(&mut err)) {
                let msg = err
                    .map(|b| String::from_utf8_lossy(std::slice::from_raw_parts(b.GetBufferPointer().cast::<u8>(), b.GetBufferSize())).into_owned())
                    .unwrap_or_default();
                return Err(Error::Fail(format!("root signature: {e} {msg}")));
            }
            let blob = blob.ok_or_else(|| Error::Fail("no root signature blob".into()))?;
            let bytes = std::slice::from_raw_parts(blob.GetBufferPointer().cast::<u8>(), blob.GetBufferSize());
            Ok(self.device.CreateRootSignature(0, bytes)?)
        }
    }
}

/// `HYBRIDGT_D3D12_DEBUG=1` turns on the D3D12 debug layer.
pub fn debug_layer_requested() -> bool {
    std::env::var("HYBRIDGT_D3D12_DEBUG").is_ok_and(|v| v == "1")
}

/// A transition barrier that borrows `res` (no reference taken or dropped).
pub fn transition(res: &ID3D12Resource, before: D3D12_RESOURCE_STATES, after: D3D12_RESOURCE_STATES) -> D3D12_RESOURCE_BARRIER {
    D3D12_RESOURCE_BARRIER {
        Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
        Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
        Anonymous: D3D12_RESOURCE_BARRIER_0 {
            Transition: ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                // SAFETY: a bitwise copy of the interface pointer, never released.
                pResource: unsafe { std::mem::transmute_copy(res) },
                Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                StateBefore: before,
                StateAfter: after,
            }),
        },
    }
}

fn borrowed(res: &ID3D12Resource) -> ManuallyDrop<Option<ID3D12Resource>> {
    // SAFETY: a bitwise copy of the interface pointer, never released.
    unsafe { std::mem::transmute_copy(res) }
}

fn bytecode(b: &[u8]) -> D3D12_SHADER_BYTECODE {
    D3D12_SHADER_BYTECODE { pShaderBytecode: b.as_ptr().cast(), BytecodeLength: b.len() }
}

/// A render target, its RTV heap and a readback buffer sized for it.
pub struct Target {
    pub texture: ID3D12Resource,
    pub rtv_heap: ID3D12DescriptorHeap,
    pub rtv: D3D12_CPU_DESCRIPTOR_HANDLE,
    pub readback: ID3D12Resource,
    pub footprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
    pub width: u32,
    pub height: u32,
}

pub const TARGET_FORMAT: DXGI_FORMAT = DXGI_FORMAT_R8G8B8A8_UNORM;

impl Target {
    pub fn new(gpu: &Gpu, width: u32, height: u32) -> Result<Target> {
        let desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
            Width: u64::from(width),
            Height: height,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: TARGET_FORMAT,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Flags: D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET,
            ..Default::default()
        };
        let clear = D3D12_CLEAR_VALUE { Format: TARGET_FORMAT, Anonymous: D3D12_CLEAR_VALUE_0 { Color: [0.0; 4] } };
        let texture = gpu.committed(D3D12_HEAP_TYPE_DEFAULT, &desc, D3D12_RESOURCE_STATE_RENDER_TARGET, Some(&clear))?;
        // SAFETY: descriptor creation into a heap we own; out pointers are locals.
        unsafe {
            let rtv_heap: ID3D12DescriptorHeap = gpu.device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                NumDescriptors: 1,
                ..Default::default()
            })?;
            let rtv = rtv_heap.GetCPUDescriptorHandleForHeapStart();
            gpu.device.CreateRenderTargetView(&texture, None, rtv);
            let mut footprint = D3D12_PLACED_SUBRESOURCE_FOOTPRINT::default();
            let mut total = 0u64;
            gpu.device.GetCopyableFootprints(&desc, 0, 1, 0, Some(&mut footprint), None, None, Some(&mut total));
            let readback = gpu.buffer(D3D12_HEAP_TYPE_READBACK, total, D3D12_RESOURCE_FLAG_NONE, D3D12_RESOURCE_STATE_COPY_DEST)?;
            Ok(Target { texture, rtv_heap, rtv, readback, footprint, width, height })
        }
    }

    /// Records: viewport, scissor, the target bound and cleared to `color`.
    pub fn begin(&self, list: &ID3D12GraphicsCommandList, color: [f32; 4]) {
        // SAFETY: recording into an open list with handles we own.
        unsafe {
            list.RSSetViewports(&[D3D12_VIEWPORT {
                Width: self.width as f32,
                Height: self.height as f32,
                MaxDepth: 1.0,
                ..Default::default()
            }]);
            list.RSSetScissorRects(&[RECT { left: 0, top: 0, right: self.width as i32, bottom: self.height as i32 }]);
            list.OMSetRenderTargets(1, Some(&self.rtv), false, None);
            list.ClearRenderTargetView(self.rtv, &color, None);
            list.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
        }
    }

    /// Records the copy of the target into its readback buffer (the target
    /// ends in COPY_SOURCE).
    pub fn copy_out(&self, list: &ID3D12GraphicsCommandList) {
        // SAFETY: recording into an open list; the copy locations borrow live resources.
        unsafe {
            list.ResourceBarrier(&[transition(&self.texture, D3D12_RESOURCE_STATE_RENDER_TARGET, D3D12_RESOURCE_STATE_COPY_SOURCE)]);
            let dst = D3D12_TEXTURE_COPY_LOCATION {
                pResource: borrowed(&self.readback),
                Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { PlacedFootprint: self.footprint },
            };
            let src = D3D12_TEXTURE_COPY_LOCATION {
                pResource: borrowed(&self.texture),
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { SubresourceIndex: 0 },
            };
            list.CopyTextureRegion(&dst, 0, 0, 0, &src, None);
        }
    }

    /// The read-back texels, RGBA8, row-major.
    pub fn pixels(&self) -> Result<Vec<[u8; 4]>> {
        let pitch = self.footprint.Footprint.RowPitch as usize;
        let mut out = Vec::with_capacity((self.width * self.height) as usize);
        // SAFETY: the readback buffer holds `height` rows of `pitch` bytes.
        unsafe {
            let mut p: *mut c_void = std::ptr::null_mut();
            self.readback.Map(0, None, Some(&mut p))?;
            let base = p.cast::<u8>();
            for y in 0..self.height as usize {
                for x in 0..self.width as usize {
                    let q = base.add(y * pitch + x * 4);
                    out.push([*q, *q.add(1), *q.add(2), *q.add(3)]);
                }
            }
            self.readback.Unmap(0, None);
        }
        Ok(out)
    }
}

fn graphics_pso(gpu: &Gpu, root: &ID3D12RootSignature, vs: &[u8], ps: &[u8], additive: bool) -> Result<ID3D12PipelineState> {
    let mut blend = D3D12_BLEND_DESC::default();
    blend.RenderTarget[0] = D3D12_RENDER_TARGET_BLEND_DESC {
        BlendEnable: additive.into(),
        SrcBlend: D3D12_BLEND_ONE,
        DestBlend: if additive { D3D12_BLEND_ONE } else { D3D12_BLEND_ZERO },
        BlendOp: D3D12_BLEND_OP_ADD,
        SrcBlendAlpha: D3D12_BLEND_ONE,
        DestBlendAlpha: if additive { D3D12_BLEND_ONE } else { D3D12_BLEND_ZERO },
        BlendOpAlpha: D3D12_BLEND_OP_ADD,
        LogicOp: D3D12_LOGIC_OP_NOOP,
        RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as u8,
        ..Default::default()
    };
    let mut rtv_formats = [DXGI_FORMAT_UNKNOWN; 8];
    rtv_formats[0] = TARGET_FORMAT;
    let desc = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
        pRootSignature: borrowed_root(root),
        VS: bytecode(vs),
        PS: bytecode(ps),
        BlendState: blend,
        SampleMask: u32::MAX,
        RasterizerState: D3D12_RASTERIZER_DESC {
            FillMode: D3D12_FILL_MODE_SOLID,
            CullMode: D3D12_CULL_MODE_NONE,
            DepthClipEnable: true.into(),
            ..Default::default()
        },
        PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
        NumRenderTargets: 1,
        RTVFormats: rtv_formats,
        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
        ..Default::default()
    };
    // SAFETY: the descriptor's pointers (bytecode, root signature) outlive the call.
    Ok(unsafe { gpu.device.CreateGraphicsPipelineState(&desc)? })
}

fn borrowed_root(root: &ID3D12RootSignature) -> ManuallyDrop<Option<ID3D12RootSignature>> {
    // SAFETY: a bitwise copy of the interface pointer, never released.
    unsafe { std::mem::transmute_copy(root) }
}

fn empty_root(gpu: &Gpu) -> Result<ID3D12RootSignature> {
    gpu.root_signature(&D3D12_ROOT_SIGNATURE_DESC::default())
}

// ---------------------------------------------------------------------------
// Probe: descriptor heap capacity.

/// The largest shader-visible CBV/SRV/UAV heap the device creates, found by
/// binary search between `lo` (known to work) and `hi`. VIctory needs the
/// title's heap plus its own dynamic slots in one heap; GTA VI's world asks
/// for 1,245,184 title slots.
pub fn max_view_heap(gpu: &Gpu, lo: u32, hi: u32) -> Result<u32> {
    let make = |n: u32| -> bool {
        // SAFETY: plain heap creation; the heap is dropped at once.
        unsafe {
            gpu.device
                .CreateDescriptorHeap::<ID3D12DescriptorHeap>(&D3D12_DESCRIPTOR_HEAP_DESC {
                    Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
                    NumDescriptors: n,
                    Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
                    NodeMask: 0,
                })
                .is_ok()
        }
    };
    if !make(lo) {
        return Err(Error::Fail(format!("a shader-visible view heap of {lo} slots was refused")));
    }
    if make(hi) {
        return Ok(hi);
    }
    let (mut ok, mut bad) = (lo, hi);
    while bad - ok > 1 {
        let mid = ok + (bad - ok) / 2;
        if make(mid) {
            ok = mid;
        } else {
            bad = mid;
        }
    }
    Ok(ok)
}

// ---------------------------------------------------------------------------
// Probe: where buffers shaders read should live.

/// Where a benchmarked buffer lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// CPU-visible system memory (`CUSTOM`, `WRITE_BACK`, `L0`): what VIctory
    /// uses for its copies of title buffers. On a discrete GPU every read
    /// crosses PCIe.
    SystemL0,
    /// Video memory, not CPU-visible (`DEFAULT`).
    Video,
    /// CPU-writable video memory (`GPU_UPLOAD`, needs Resizable BAR).
    GpuUpload,
}

/// Whether the device offers `D3D12_HEAP_TYPE_GPU_UPLOAD`.
pub fn gpu_upload_heap_supported(gpu: &Gpu) -> bool {
    let mut o16 = D3D12_FEATURE_DATA_D3D12_OPTIONS16::default();
    // SAFETY: a sized out structure.
    unsafe {
        gpu.device
            .CheckFeatureSupport(D3D12_FEATURE_D3D12_OPTIONS16, &mut o16 as *mut _ as *mut c_void, size_of_val(&o16) as u32)
            .is_ok()
            && o16.GPUUploadHeapSupported.as_bool()
    }
}

pub const READ_CS: &str = r#"
RWByteAddressBuffer src : register(u0);
RWByteAddressBuffer dst : register(u1);
[numthreads(64, 1, 1)] void main(uint3 id : SV_DispatchThreadID) {
    uint4 acc = 0;
    [unroll] for (uint k = 0; k < 16; k++) { acc += src.Load4((id.x * 16 + k) * 16); }
    if (acc.x == 0x12345678) dst.Store(0, acc.y);
}
"#;

/// Time for one compute pass reading `mib` MiB of a buffer in `placement`
/// (best of 3 submissions, wall time of submit + wait).
pub fn bench_buffer_read(gpu: &Gpu, placement: Placement, mib: u64) -> Result<Duration> {
    let (cs, _) = shaders::compile(READ_CS, "cs_6_0")?;
    let bytes = mib << 20;
    let flags = D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS;
    let desc = D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
        Width: bytes,
        Height: 1,
        DepthOrArraySize: 1,
        MipLevels: 1,
        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
        Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
        Flags: flags,
        ..Default::default()
    };
    let props = match placement {
        Placement::SystemL0 => D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_CUSTOM,
            CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_WRITE_BACK,
            MemoryPoolPreference: D3D12_MEMORY_POOL_L0,
            ..Default::default()
        },
        Placement::Video => D3D12_HEAP_PROPERTIES { Type: D3D12_HEAP_TYPE_DEFAULT, ..Default::default() },
        Placement::GpuUpload => {
            if !gpu_upload_heap_supported(gpu) {
                return Err(Error::Skip("no GPU_UPLOAD heap (Resizable BAR off or unsupported)".into()));
            }
            D3D12_HEAP_PROPERTIES { Type: D3D12_HEAP_TYPE_GPU_UPLOAD, ..Default::default() }
        }
    };
    let mut src: Option<ID3D12Resource> = None;
    // SAFETY: descriptors are locals.
    unsafe {
        gpu.device.CreateCommittedResource(&props, D3D12_HEAP_FLAG_NONE, &desc, D3D12_RESOURCE_STATE_COMMON, None, &mut src)?;
    }
    let src = src.ok_or_else(|| Error::Fail("no buffer".into()))?;
    let dst = gpu.buffer(D3D12_HEAP_TYPE_DEFAULT, 256, flags, D3D12_RESOURCE_STATE_COMMON)?;
    let uav = |reg| D3D12_ROOT_PARAMETER {
        ParameterType: D3D12_ROOT_PARAMETER_TYPE_UAV,
        Anonymous: D3D12_ROOT_PARAMETER_0 { Descriptor: D3D12_ROOT_DESCRIPTOR { ShaderRegister: reg, RegisterSpace: 0 } },
        ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
    };
    let params = [uav(0), uav(1)];
    let root = gpu.root_signature(&D3D12_ROOT_SIGNATURE_DESC { NumParameters: 2, pParameters: params.as_ptr(), ..Default::default() })?;
    // SAFETY: the descriptor's pointers outlive the call.
    let pso: ID3D12PipelineState = unsafe {
        gpu.device.CreateComputePipelineState(&D3D12_COMPUTE_PIPELINE_STATE_DESC {
            pRootSignature: borrowed_root(&root),
            CS: bytecode(&cs),
            ..Default::default()
        })?
    };
    let groups = (bytes / (16 * 16 * 64)) as u32;
    let mut best = Duration::MAX;
    for _ in 0..3 {
        let took = gpu.run(|list| {
            // SAFETY: recording into an open list with live resources.
            unsafe {
                list.SetComputeRootSignature(&root);
                list.SetPipelineState(&pso);
                list.SetComputeRootUnorderedAccessView(0, src.GetGPUVirtualAddress());
                list.SetComputeRootUnorderedAccessView(1, dst.GetGPUVirtualAddress());
                list.Dispatch(groups, 1, 1);
            }
            Ok(())
        })?;
        best = best.min(took);
    }
    Ok(best)
}

// ---------------------------------------------------------------------------
// Check 1: clear and read back.

/// The baseline: a cleared target reads back as its clear colour. If this
/// fails nothing else here means anything.
pub fn check_clear(gpu: &Gpu) -> Result<Timings> {
    let target = Target::new(gpu, 16, 16)?;
    let color = [0.25f32, 0.5, 0.75, 1.0];
    let took = gpu.run(|list| {
        target.begin(list, color);
        target.copy_out(list);
        Ok(())
    })?;
    // D3D allows float -> UNORM conversion within 0.6 ULP of the exact
    // value: 0.5 * 255 = 127.5 may store 127 or 128 (RTX 4070 Ti: 127).
    let exact = color.map(|c| c * 255.0);
    let px = target.pixels()?;
    let off = |p: &[u8; 4]| p.iter().zip(exact).any(|(&g, e)| (f32::from(g) - e).abs() > 0.6);
    if let Some((i, p)) = px.iter().enumerate().find(|(_, p)| off(p)) {
        let want = exact;
        return Err(Error::Fail(format!("texel {i} is {p:?}, expected {want:?}")));
    }
    let mut t = Timings::default();
    t.add("submit+wait", took);
    Ok(t)
}

// ---------------------------------------------------------------------------
// Check 2: indirect draw counts produced on the GPU.

/// Where an indirect draw count comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountSource {
    /// Written by the CPU through an upload copy.
    Uploaded(u32),
    /// Written by a compute dispatch (the producer). The buffer holds
    /// `stale` before the producer runs.
    Produced { value: u32, stale: u32, producer: Producer },
}

/// What a runtime does with the producer dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Producer {
    Runs,
    /// Skipped (e.g. its pipeline has no translation); the buffer keeps
    /// whatever it held.
    SkippedLeave,
    /// Skipped, and the runtime zeroes the outputs the dispatch would have written.
    SkippedZeroed,
}

/// What a runtime does with an indirect count before the GPU consumes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountGuard {
    /// The count buffer goes to `ExecuteIndirect` as it is.
    None,
    /// A compute pass writes `min(count, MaxCommandCount)` to a shadow
    /// buffer, and `ExecuteIndirect` reads the shadow. D3D12 already
    /// requires the GPU to clamp. This guard exists because an unclamped
    /// 0xFFFFFFFF count hung an NVIDIA device once (see
    /// docs/findings/2026-11-24-indirect-count-hang.md).
    Clamp,
}

/// Whether running this combination could hang a real GPU: a count above
/// `max` reaching `ExecuteIndirect` without a guard. On a hardware adapter
/// such cases are refused unless `HYBRIDGT_ALLOW_HANG_REPRO=1`.
pub fn may_hang(source: CountSource, max: u32, guard: CountGuard) -> bool {
    let reaching = match source {
        CountSource::Uploaded(v) => v,
        CountSource::Produced { value, producer: Producer::Runs, .. } => value,
        CountSource::Produced { stale, producer: Producer::SkippedLeave, .. } => stale,
        CountSource::Produced { producer: Producer::SkippedZeroed, .. } => 0,
    };
    guard == CountGuard::None && reaching > max
}

/// Runs an `ExecuteIndirect` of up to `max` full-screen draws whose count
/// comes from `source`, and returns how many draws reached the target.
///
/// Each draw adds 1/255 to red, so red counts draws up to 255.
pub fn indirect_draws(gpu: &Gpu, source: CountSource, max: u32, guard: CountGuard) -> Result<(u32, Timings)> {
    assert!(max <= 255, "red channel counts at most 255 draws");
    if !gpu.info.software
        && may_hang(source, max, guard)
        && !std::env::var("HYBRIDGT_ALLOW_HANG_REPRO").is_ok_and(|v| v == "1")
    {
        return Err(Error::Skip(
            "an unguarded count above MaxCommandCount hung an NVIDIA device once; not run on hardware \
             (HYBRIDGT_ALLOW_HANG_REPRO=1 runs it)"
                .into(),
        ));
    }
    let mut t = Timings::default();
    let (vs, d) = shaders::compile(shaders::FULLSCREEN_VS, "vs_6_0")?;
    t.add("dxc vs", d);
    let (ps, d) = shaders::compile(shaders::COUNT_PS, "ps_6_0")?;
    t.add("dxc ps", d);
    let (cs, d) = shaders::compile(shaders::COUNT_CS, "cs_6_0")?;
    t.add("dxc cs", d);
    let (clamp_cs, d) = shaders::compile(shaders::CLAMP_CS, "cs_6_0")?;
    t.add("dxc clamp cs", d);

    let started = Instant::now();
    let gfx_root = empty_root(gpu)?;
    let pso = graphics_pso(gpu, &gfx_root, &vs, &ps, true)?;
    let uav = |reg| D3D12_ROOT_PARAMETER {
        ParameterType: D3D12_ROOT_PARAMETER_TYPE_UAV,
        Anonymous: D3D12_ROOT_PARAMETER_0 { Descriptor: D3D12_ROOT_DESCRIPTOR { ShaderRegister: reg, RegisterSpace: 0 } },
        ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
    };
    let params = [
        uav(0),
        uav(1),
        D3D12_ROOT_PARAMETER {
            ParameterType: D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS,
            Anonymous: D3D12_ROOT_PARAMETER_0 {
                Constants: D3D12_ROOT_CONSTANTS { ShaderRegister: 0, RegisterSpace: 0, Num32BitValues: 1 },
            },
            ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
        },
    ];
    let cs_root = gpu.root_signature(&D3D12_ROOT_SIGNATURE_DESC {
        NumParameters: params.len() as u32,
        pParameters: params.as_ptr(),
        ..Default::default()
    })?;
    let compute = |code: &[u8]| -> Result<ID3D12PipelineState> {
        // SAFETY: the descriptor's pointers outlive the call.
        Ok(unsafe {
            gpu.device.CreateComputePipelineState(&D3D12_COMPUTE_PIPELINE_STATE_DESC {
                pRootSignature: borrowed_root(&cs_root),
                CS: bytecode(code),
                ..Default::default()
            })?
        })
    };
    let producer_pso = compute(&cs)?;
    let clamp_pso = compute(&clamp_cs)?;
    let arg = D3D12_INDIRECT_ARGUMENT_DESC { Type: D3D12_INDIRECT_ARGUMENT_TYPE_DRAW, ..Default::default() };
    let mut signature: Option<ID3D12CommandSignature> = None;
    // SAFETY: the descriptor's pointers outlive the call.
    unsafe {
        gpu.device.CreateCommandSignature(
            &D3D12_COMMAND_SIGNATURE_DESC {
                ByteStride: size_of::<D3D12_DRAW_ARGUMENTS>() as u32,
                NumArgumentDescs: 1,
                pArgumentDescs: &arg,
                NodeMask: 0,
            },
            None,
            &mut signature,
        )?;
    }
    let signature = signature.ok_or_else(|| Error::Fail("no command signature".into()))?;
    t.add("pipelines", started.elapsed());

    // `max` draws of one full-screen triangle each.
    let mut args = Vec::new();
    for _ in 0..max.max(1) {
        for v in [3u32, 1, 0, 0] {
            args.extend_from_slice(&v.to_le_bytes());
        }
    }
    let arg_buf = gpu.upload(&args)?;
    let (initial, produced) = match source {
        CountSource::Uploaded(v) => (v, None),
        CountSource::Produced { value, stale, producer } => (stale, Some((value, producer))),
    };
    let init_buf = gpu.upload(&initial.to_le_bytes())?;
    let zero_buf = gpu.upload(&[0u8; 4])?;
    // Buffers are created in COMMON whatever the initial state says; the
    // first copy promotes them to COPY_DEST.
    let flags = D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS;
    let count = gpu.buffer(D3D12_HEAP_TYPE_DEFAULT, 4, flags, D3D12_RESOURCE_STATE_COMMON)?;
    let shadow = gpu.buffer(D3D12_HEAP_TYPE_DEFAULT, 4, flags, D3D12_RESOURCE_STATE_COMMON)?;
    let target = Target::new(gpu, 4, 4)?;

    let took = gpu.run(|list| {
        // SAFETY: recording into an open list; every resource outlives the run.
        unsafe {
            list.CopyBufferRegion(&count, 0, &init_buf, 0, 4);
            let mut state = D3D12_RESOURCE_STATE_COPY_DEST;
            let mut to = |list: &ID3D12GraphicsCommandList, after: D3D12_RESOURCE_STATES| {
                if state != after {
                    list.ResourceBarrier(&[transition(&count, state, after)]);
                    state = after;
                }
            };
            list.SetComputeRootSignature(&cs_root);
            match produced {
                Some((value, Producer::Runs)) => {
                    to(list, D3D12_RESOURCE_STATE_UNORDERED_ACCESS);
                    list.SetPipelineState(&producer_pso);
                    list.SetComputeRootUnorderedAccessView(0, count.GetGPUVirtualAddress());
                    list.SetComputeRootUnorderedAccessView(1, count.GetGPUVirtualAddress());
                    list.SetComputeRoot32BitConstant(2, value, 0);
                    list.Dispatch(1, 1, 1);
                }
                Some((_, Producer::SkippedZeroed)) => list.CopyBufferRegion(&count, 0, &zero_buf, 0, 4),
                Some((_, Producer::SkippedLeave)) | None => {}
            }
            let consumed = match guard {
                CountGuard::None => {
                    to(list, D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT);
                    &count
                }
                CountGuard::Clamp => {
                    to(list, D3D12_RESOURCE_STATE_UNORDERED_ACCESS);
                    list.ResourceBarrier(&[
                        transition(&shadow, D3D12_RESOURCE_STATE_COMMON, D3D12_RESOURCE_STATE_UNORDERED_ACCESS),
                        // the producer's write must land before the clamp reads it
                        D3D12_RESOURCE_BARRIER {
                            Type: D3D12_RESOURCE_BARRIER_TYPE_UAV,
                            Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                            Anonymous: D3D12_RESOURCE_BARRIER_0 {
                                UAV: ManuallyDrop::new(D3D12_RESOURCE_UAV_BARRIER { pResource: borrowed(&count) }),
                            },
                        },
                    ]);
                    list.SetPipelineState(&clamp_pso);
                    list.SetComputeRootUnorderedAccessView(0, count.GetGPUVirtualAddress());
                    list.SetComputeRootUnorderedAccessView(1, shadow.GetGPUVirtualAddress());
                    list.SetComputeRoot32BitConstant(2, max, 0);
                    list.Dispatch(1, 1, 1);
                    list.ResourceBarrier(&[transition(
                        &shadow,
                        D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                        D3D12_RESOURCE_STATE_INDIRECT_ARGUMENT,
                    )]);
                    &shadow
                }
            };
            target.begin(list, [0.0; 4]);
            list.SetGraphicsRootSignature(&gfx_root);
            list.SetPipelineState(&pso);
            list.ExecuteIndirect(&signature, max, &arg_buf, 0, consumed, 0);
        }
        target.copy_out(list);
        Ok(())
    })?;
    t.add("submit+wait", took);
    let px = target.pixels()?;
    let red = px[0][0];
    if px.iter().any(|p| p[0] != red) {
        return Err(Error::Fail(format!("texels disagree: {:?}", px)));
    }
    Ok((u32::from(red), t))
}

// ---------------------------------------------------------------------------
// Check 3: a BC1 texture uploaded from CPU memory, fetched texel by texel.

/// Four BC1 blocks (an 8x8 texture) covering both palette modes, every
/// index, transparent black and equal endpoints.
pub fn bc1_fixture() -> Vec<[u8; 8]> {
    use hgt_texture::bc::solid_bc1_block;
    let mut a = solid_bc1_block(0xFFFF, 0x0000, 0);
    a[4..8].copy_from_slice(&0xE4E4_E4E4u32.to_le_bytes()); // rows 0,1,2,3 indices
    let mut b = solid_bc1_block(0x001F, 0xF800, 0); // c0 < c1: 3-colour mode
    b[4..8].copy_from_slice(&0x1B1B_1B1Bu32.to_le_bytes()); // 3,2,1,0 per row
    let c = solid_bc1_block(0x07E0, 0x0000, 0); // solid green
    let mut d = solid_bc1_block(0x8410, 0x8410, 0); // equal endpoints: 3-colour mode
    d[4..8].copy_from_slice(&0xFFAA_5500u32.to_le_bytes());
    vec![a, b, c, d]
}

/// Uploads [`bc1_fixture`] as an 8x8 BC1 texture, draws it with exact texel
/// fetches and compares every texel with [`hgt_texture::bc::decode_bc1_block`].
/// Endpoints and transparent black must match exactly; interpolated colours
/// within 3/255 (the tolerance Direct3D allows hardware decoders).
pub fn check_bc1_texture(gpu: &Gpu) -> Result<Timings> {
    let mut t = Timings::default();
    let (vs, d) = shaders::compile(shaders::FULLSCREEN_VS, "vs_6_0")?;
    t.add("dxc vs", d);
    let (ps, d) = shaders::compile(shaders::LOAD_PS, "ps_6_0")?;
    t.add("dxc ps", d);
    let blocks = bc1_fixture();

    let range = D3D12_DESCRIPTOR_RANGE {
        RangeType: D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
        NumDescriptors: 1,
        BaseShaderRegister: 0,
        RegisterSpace: 0,
        OffsetInDescriptorsFromTableStart: 0,
    };
    let param = D3D12_ROOT_PARAMETER {
        ParameterType: D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
        Anonymous: D3D12_ROOT_PARAMETER_0 {
            DescriptorTable: D3D12_ROOT_DESCRIPTOR_TABLE { NumDescriptorRanges: 1, pDescriptorRanges: &range },
        },
        ShaderVisibility: D3D12_SHADER_VISIBILITY_PIXEL,
    };
    let started = Instant::now();
    let root = gpu.root_signature(&D3D12_ROOT_SIGNATURE_DESC { NumParameters: 1, pParameters: &param, ..Default::default() })?;
    let pso = graphics_pso(gpu, &root, &vs, &ps, false)?;
    t.add("pipelines", started.elapsed());

    let tex_desc = D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
        Width: 8,
        Height: 8,
        DepthOrArraySize: 1,
        MipLevels: 1,
        Format: DXGI_FORMAT_BC1_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
        ..Default::default()
    };
    let texture = gpu.committed(D3D12_HEAP_TYPE_DEFAULT, &tex_desc, D3D12_RESOURCE_STATE_COPY_DEST, None)?;
    let mut fp = D3D12_PLACED_SUBRESOURCE_FOOTPRINT::default();
    let mut rows = 0u32;
    let mut row_bytes = 0u64;
    let mut total = 0u64;
    // SAFETY: out pointers are locals.
    unsafe {
        gpu.device.GetCopyableFootprints(&tex_desc, 0, 1, 0, Some(&mut fp), Some(&mut rows), Some(&mut row_bytes), Some(&mut total));
    }
    if rows != 2 || row_bytes != 16 {
        return Err(Error::Fail(format!("BC1 8x8 footprint: {rows} rows of {row_bytes} bytes, expected 2 of 16")));
    }
    let mut staging = vec![0u8; total as usize];
    for (i, b) in blocks.iter().enumerate() {
        let (bx, by) = (i % 2, i / 2);
        let at = by * fp.Footprint.RowPitch as usize + bx * 8;
        staging[at..at + 8].copy_from_slice(b);
    }
    let upload = gpu.upload(&staging)?;
    // SAFETY: descriptor creation into a heap we own.
    let heap: ID3D12DescriptorHeap = unsafe {
        let heap: ID3D12DescriptorHeap = gpu.device.CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
            NumDescriptors: 1,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
            NodeMask: 0,
        })?;
        gpu.device.CreateShaderResourceView(
            &texture,
            Some(&D3D12_SHADER_RESOURCE_VIEW_DESC {
                Format: DXGI_FORMAT_BC1_UNORM,
                ViewDimension: D3D12_SRV_DIMENSION_TEXTURE2D,
                Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
                Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                    Texture2D: D3D12_TEX2D_SRV { MipLevels: 1, ..Default::default() },
                },
            }),
            heap.GetCPUDescriptorHandleForHeapStart(),
        );
        heap
    };
    let target = Target::new(gpu, 8, 8)?;
    let took = gpu.run(|list| {
        // SAFETY: recording into an open list; every resource outlives the run.
        unsafe {
            let dst = D3D12_TEXTURE_COPY_LOCATION {
                pResource: borrowed(&texture),
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { SubresourceIndex: 0 },
            };
            let src = D3D12_TEXTURE_COPY_LOCATION {
                pResource: borrowed(&upload),
                Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { PlacedFootprint: fp },
            };
            list.CopyTextureRegion(&dst, 0, 0, 0, &src, None);
            list.ResourceBarrier(&[transition(&texture, D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE)]);
            target.begin(list, [1.0, 0.0, 1.0, 1.0]);
            list.SetGraphicsRootSignature(&root);
            list.SetDescriptorHeaps(&[Some(heap.clone())]);
            list.SetGraphicsRootDescriptorTable(0, heap.GetGPUDescriptorHandleForHeapStart());
            list.SetPipelineState(&pso);
            list.DrawInstanced(3, 1, 0, 0);
        }
        target.copy_out(list);
        Ok(())
    })?;
    t.add("submit+wait", took);

    let px = target.pixels()?;
    let mut bad = Vec::new();
    // A texel passes when it matches a known decoder: the exact formula
    // (interpolated colours within 3/255), or NVIDIA's (within 1/255).
    // Endpoints and transparent black must be exact in both.
    let near = |got: [u8; 4], want: [u8; 4], tol: u8| got.iter().zip(want).all(|(g, w)| g.abs_diff(w) <= tol);
    for (i, b) in blocks.iter().enumerate() {
        let exact_ref = hgt_texture::bc::decode_bc1_block(b);
        let nv_ref = hgt_texture::bc::decode_bc1_block_nvidia(b);
        let (bx, by) = (i % 2, i / 2);
        for j in 0..16 {
            let (x, y) = (bx * 4 + j % 4, by * 4 + j / 4);
            let got = px[y * 8 + x];
            let index = (u32::from_le_bytes([b[4], b[5], b[6], b[7]]) >> (2 * j)) & 3;
            let fixed = index < 2 || exact_ref[j] == [0, 0, 0, 0];
            let ok = if fixed {
                got == exact_ref[j]
            } else {
                near(got, exact_ref[j], 3) || near(got, nv_ref[j], 1)
            };
            if !ok {
                bad.push(format!("({x},{y}) got {got:?} want {:?} or NVIDIA {:?}", exact_ref[j], nv_ref[j]));
            }
        }
    }
    if !bad.is_empty() {
        return Err(Error::Fail(format!("{} texels differ: {}", bad.len(), bad.join("; "))));
    }
    Ok(t)
}

// ---------------------------------------------------------------------------
// Check: wave64 semantics on the host.

/// What one 64-thread group showed about the host's waves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wave64Report {
    /// `WaveGetLaneCount()` on the host (32 on NVIDIA, 32 or 64 on AMD).
    pub host_lanes: u32,
    /// The native ballot equals the console's 64-lane ballot in every thread.
    pub native_matches_console: bool,
    /// The two-half group-shared emulation equals the console's ballot.
    pub emulated_matches_console: bool,
}

/// The console's 64-bit ballot of `t % 3 == 0` over lanes 0..64, as (low, high).
pub fn console_ballot() -> (u32, u32) {
    let m: u64 = (0..64u32).filter(|t| t % 3 == 0).fold(0, |m, t| m | 1 << t);
    (m as u32, (m >> 32) as u32)
}

/// Runs `WAVE64_CS` and compares native and emulated ballots with the
/// console's answer. Also checks that each native ballot is consistent with
/// the host's own lane count (a host bug would fail here, not a mismatch).
pub fn check_wave64(gpu: &Gpu) -> Result<Wave64Report> {
    let (cs, _) = shaders::compile(shaders::WAVE64_CS, "cs_6_0")?;
    let size = 64 * 16 + 16;
    let flags = D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS;
    let out = gpu.buffer(D3D12_HEAP_TYPE_DEFAULT, size, flags, D3D12_RESOURCE_STATE_COMMON)?;
    let back = gpu.buffer(D3D12_HEAP_TYPE_READBACK, size, D3D12_RESOURCE_FLAG_NONE, D3D12_RESOURCE_STATE_COPY_DEST)?;
    let params = [D3D12_ROOT_PARAMETER {
        ParameterType: D3D12_ROOT_PARAMETER_TYPE_UAV,
        Anonymous: D3D12_ROOT_PARAMETER_0 { Descriptor: D3D12_ROOT_DESCRIPTOR { ShaderRegister: 0, RegisterSpace: 0 } },
        ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
    }];
    let root = gpu.root_signature(&D3D12_ROOT_SIGNATURE_DESC { NumParameters: 1, pParameters: params.as_ptr(), ..Default::default() })?;
    // SAFETY: the descriptor's pointers outlive the call.
    let pso: ID3D12PipelineState = unsafe {
        gpu.device.CreateComputePipelineState(&D3D12_COMPUTE_PIPELINE_STATE_DESC {
            pRootSignature: borrowed_root(&root),
            CS: bytecode(&cs),
            ..Default::default()
        })?
    };
    gpu.run(|list| {
        // SAFETY: recording into an open list with live resources.
        unsafe {
            list.SetComputeRootSignature(&root);
            list.SetPipelineState(&pso);
            list.SetComputeRootUnorderedAccessView(0, out.GetGPUVirtualAddress());
            list.Dispatch(1, 1, 1);
            list.ResourceBarrier(&[transition(&out, D3D12_RESOURCE_STATE_UNORDERED_ACCESS, D3D12_RESOURCE_STATE_COPY_SOURCE)]);
            list.CopyBufferRegion(&back, 0, &out, 0, size);
        }
        Ok(())
    })?;
    let mut words = vec![0u32; (size / 4) as usize];
    // SAFETY: the readback buffer holds `size` bytes and the GPU is done.
    unsafe {
        let mut p = std::ptr::null_mut();
        back.Map(0, None, Some(&mut p))?;
        std::ptr::copy_nonoverlapping(p as *const u32, words.as_mut_ptr(), words.len());
        back.Unmap(0, None);
    }
    let (lo, hi) = console_ballot();
    let full: u64 = u64::from(lo) | u64::from(hi) << 32;
    let host_lanes = words[0];
    if !matches!(host_lanes, 4..=128) || !host_lanes.is_power_of_two() {
        return Err(Error::Fail(format!("host lane count {host_lanes}")));
    }
    let mut native_matches_console = true;
    for t in 0..64u32 {
        let w = &words[(t * 4) as usize..(t * 4 + 4) as usize];
        if w[0] != host_lanes {
            return Err(Error::Fail(format!("thread {t}: lane count {} vs {host_lanes}", w[0])));
        }
        // What this thread's own host wave should report.
        let base = t / host_lanes * host_lanes;
        let lanes = host_lanes.min(64);
        let own = (full >> base) & if lanes == 64 { u64::MAX } else { (1u64 << lanes) - 1 };
        let native = u64::from(w[2]) | u64::from(w[3]) << 32;
        if w[1] != t % host_lanes || native != own {
            return Err(Error::Fail(format!("thread {t}: lane {} ballot {native:#x}, host wave expects {own:#x}", w[1])));
        }
        native_matches_console &= native == full;
    }
    let emulated_matches_console = words[256] == lo && words[257] == hi;
    Ok(Wave64Report { host_lanes, native_matches_console, emulated_matches_console })
}
