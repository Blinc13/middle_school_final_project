//use qubicon_vulkan::memory::resources::mapped_resource::MappableType;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraData {
    pub pos: (f32, f32, f32),
    pub mat: (
        (f32, f32, f32),
        (f32, f32, f32),
        (f32, f32, f32)
    )
}

//unsafe impl MappableType for CameraData {}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoxelData {
    pub child_indicies: [u32; 8],
    pub pallete_idx: u32
}

//unsafe impl MappableType for VoxelData {}