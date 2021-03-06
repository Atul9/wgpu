use crate::{
    swap_chain::{SwapChainLink, SwapImageEpoch},
    BufferMapReadCallback, BufferMapWriteCallback, DeviceId, Extent3d, LifeGuard, RefCount, Stored,
    TextureId,
};

use bitflags::bitflags;
use hal;
use parking_lot::Mutex;

use std::borrow::Borrow;


bitflags! {
    #[repr(transparent)]
    pub struct BufferUsageFlags: u32 {
        const MAP_READ = 1;
        const MAP_WRITE = 2;
        const TRANSFER_SRC = 4;
        const TRANSFER_DST = 8;
        const INDEX = 16;
        const VERTEX = 32;
        const UNIFORM = 64;
        const STORAGE = 128;
        const NONE = 0;
        const WRITE_ALL = 2 + 8 + 128;
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct BufferDescriptor {
    pub size: u32,
    pub usage: BufferUsageFlags,
}

pub enum BufferMapAsyncStatus {
    Success,
    Error,
    Unknown,
    ContextLost,
}

#[derive(Clone)]
pub(crate) enum BufferMapOperation {
    Read(std::ops::Range<u64>, BufferMapReadCallback, *mut u8),
    Write(std::ops::Range<u64>, BufferMapWriteCallback, *mut u8),
}

unsafe impl Send for BufferMapOperation {}
unsafe impl Sync for BufferMapOperation {}

pub struct Buffer<B: hal::Backend> {
    pub(crate) raw: B::Buffer,
    pub(crate) device_id: Stored<DeviceId>,
    pub(crate) memory_properties: hal::memory::Properties,
    pub(crate) memory: B::Memory,
    pub(crate) mapped_write_ranges: Vec<std::ops::Range<u64>>,
    pub(crate) pending_map_operation: Option<BufferMapOperation>,
    pub(crate) life_guard: LifeGuard,
}

impl<B: hal::Backend> Borrow<RefCount> for Buffer<B> {
    fn borrow(&self) -> &RefCount {
        &self.life_guard.ref_count
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum TextureDimension {
    D1,
    D2,
    D3,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum TextureFormat {
    // Normal 8 bit formats
    R8Unorm = 0,
    R8UnormSrgb = 1,
    R8Snorm = 2,
    R8Uint = 3,
    R8Sint = 4,

    // Normal 16 bit formats
    R16Unorm = 5,
    R16Snorm = 6,
    R16Uint = 7,
    R16Sint = 8,
    R16Float = 9,

    Rg8Unorm = 10,
    Rg8UnormSrgb = 11,
    Rg8Snorm = 12,
    Rg8Uint = 13,
    Rg8Sint = 14,

    // Packed 16 bit formats
    B5g6r5Unorm = 15,

    // Normal 32 bit formats
    R32Uint = 16,
    R32Sint = 17,
    R32Float = 18,
    Rg16Unorm = 19,
    Rg16Snorm = 20,
    Rg16Uint = 21,
    Rg16Sint = 22,
    Rg16Float = 23,
    Rgba8Unorm = 24,
    Rgba8UnormSrgb = 25,
    Rgba8Snorm = 26,
    Rgba8Uint = 27,
    Rgba8Sint = 28,
    Bgra8Unorm = 29,
    Bgra8UnormSrgb = 30,

    // Packed 32 bit formats
    Rgb10a2Unorm = 31,
    Rg11b10Float = 32,

    // Normal 64 bit formats
    Rg32Uint = 33,
    Rg32Sint = 34,
    Rg32Float = 35,
    Rgba16Unorm = 36,
    Rgba16Snorm = 37,
    Rgba16Uint = 38,
    Rgba16Sint = 39,
    Rgba16Float = 40,

    // Normal 128 bit formats
    Rgba32Uint = 41,
    Rgba32Sint = 42,
    Rgba32Float = 43,

    // Depth and stencil formats
    D16Unorm = 44,
    D32Float = 45,
    D24UnormS8Uint = 46,
    D32FloatS8Uint = 47,
}

bitflags! {
    #[repr(transparent)]
    pub struct TextureUsageFlags: u32 {
        const TRANSFER_SRC = 1;
        const TRANSFER_DST = 2;
        const SAMPLED = 4;
        const STORAGE = 8;
        const OUTPUT_ATTACHMENT = 16;
        const NONE = 0;
        const WRITE_ALL = 2 + 8 + 16;
        const UNINITIALIZED = 0xFFFF;
    }
}

#[repr(C)]
pub struct TextureDescriptor {
    pub size: Extent3d,
    pub array_size: u32,
    pub dimension: TextureDimension,
    pub format: TextureFormat,
    pub usage: TextureUsageFlags,
}

pub(crate) enum TexturePlacement<B: hal::Backend> {
    SwapChain(SwapChainLink<Mutex<SwapImageEpoch>>),
    Memory(B::Memory),
    Void,
}

impl<B: hal::Backend> TexturePlacement<B> {
    pub fn as_swap_chain(&self) -> &SwapChainLink<Mutex<SwapImageEpoch>> {
        match *self {
            TexturePlacement::SwapChain(ref link) => link,
            TexturePlacement::Memory(_) | TexturePlacement::Void => {
                panic!("Expected swap chain link!")
            }
        }
    }
}

pub struct Texture<B: hal::Backend> {
    pub(crate) raw: B::Image,
    pub(crate) device_id: Stored<DeviceId>,
    pub(crate) kind: hal::image::Kind,
    pub(crate) format: TextureFormat,
    pub(crate) full_range: hal::image::SubresourceRange,
    pub(crate) placement: TexturePlacement<B>,
    pub(crate) life_guard: LifeGuard,
}

impl<B: hal::Backend> Borrow<RefCount> for Texture<B> {
    fn borrow(&self) -> &RefCount {
        &self.life_guard.ref_count
    }
}

bitflags! {
    #[repr(transparent)]
    pub struct TextureAspectFlags: u32 {
        const COLOR = 1;
        const DEPTH = 2;
        const STENCIL = 4;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum TextureViewDimension {
    D1,
    D2,
    D2Array,
    Cube,
    CubeArray,
    D3,
}

#[repr(C)]
pub struct TextureViewDescriptor {
    pub format: TextureFormat,
    pub dimension: TextureViewDimension,
    pub aspect: TextureAspectFlags,
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub array_count: u32,
}

pub struct TextureView<B: hal::Backend> {
    pub(crate) raw: B::ImageView,
    pub(crate) texture_id: Stored<TextureId>,
    //TODO: store device_id for quick access?
    pub(crate) format: TextureFormat,
    pub(crate) extent: hal::image::Extent,
    pub(crate) samples: hal::image::NumSamples,
    pub(crate) is_owned_by_swap_chain: bool,
    #[cfg_attr(not(feature = "local"), allow(dead_code))]
    pub(crate) life_guard: LifeGuard,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum AddressMode {
    ClampToEdge = 0,
    Repeat = 1,
    MirrorRepeat = 2,
    ClampToBorderColor = 3,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum FilterMode {
    Nearest = 0,
    Linear = 1,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum CompareFunction {
    Never = 0,
    Less = 1,
    Equal = 2,
    LessEqual = 3,
    Greater = 4,
    NotEqual = 5,
    GreaterEqual = 6,
    Always = 7,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum BorderColor {
    TransparentBlack = 0,
    OpaqueBlack = 1,
    OpaqueWhite = 2,
}

#[repr(C)]
pub struct SamplerDescriptor {
    pub r_address_mode: AddressMode,
    pub s_address_mode: AddressMode,
    pub t_address_mode: AddressMode,
    pub mag_filter: FilterMode,
    pub min_filter: FilterMode,
    pub mipmap_filter: FilterMode,
    pub lod_min_clamp: f32,
    pub lod_max_clamp: f32,
    pub max_anisotropy: u32,
    pub compare_function: CompareFunction,
    pub border_color: BorderColor,
}

pub struct Sampler<B: hal::Backend> {
    pub(crate) raw: B::Sampler,
}
