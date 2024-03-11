//use qubicon_vulkan::memory::resources::mapped_resource::MappableType;

use super::camera::{Mat3, Mat4, Vec3};

#[repr(align(16))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlignedVec(Vec3);
#[repr(align(16))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlignedMat(Mat3);

impl From<Vec3> for AlignedVec {
    fn from(value: Vec3) -> Self {
        Self (value)
    }
}
impl From<Mat3> for AlignedMat {
    fn from(value: Mat3) -> Self {
        Self (value)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraData {
    pub pos: AlignedVec,
    pub basis: Mat4
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoxelData {
    pub child_indicies: [u32; 8],
    pub pallete_idx: u32
}

//unsafe impl MappableType for VoxelData {}