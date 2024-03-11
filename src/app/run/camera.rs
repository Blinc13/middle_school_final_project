//! Soon this piece of code will be one year old!
//! It consist of things what i wrote myseft and pieces from some papers

use super::gpu_shared_data::CameraData;

pub type Mat3 = nalgebra::Matrix3<f32>;
pub type Mat4 = nalgebra::Matrix4<f32>;
pub type Vec3 = nalgebra::Vector3<f32>;
pub type Vec2 = nalgebra::Vector2<f32>;

pub struct CamBasis {
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3,
    
    pub pos: Vec3
}

#[inline(always)]
fn rotate_by_axis(axis: Vec3, angle: f32) -> Mat3 {
    let cos = angle.cos();
    let sin = angle.sin();
    let t = 1.0 - cos;

    Mat3::new(
        t * axis.x * axis.x + cos         , t * axis.x * axis.y - sin * axis.z, t * axis.x * axis.z + sin * axis.y,
        t * axis.x * axis.y + sin * axis.z, t * axis.y * axis.y + cos,          t * axis.y * axis.z - sin * axis.x,
        t * axis.x * axis.z - sin * axis.y, t * axis.y * axis.z + sin * axis.x,       t * axis.z * axis.z + cos
    )
}

impl Default for CamBasis {
    fn default() -> Self {
        Self {
            x: Vec3::new(1.0, 0.0, 0.0),
            y: Vec3::new(0.0, 1.0, 0.0),
            z: Vec3::new(0.0, 0.0, 1.0),
            
            pos: Vec3::zeros()
        }
    }
}

impl CamBasis {
    pub fn rotate(&mut self, axis: Vec3, angle: f32) {
        let mat = rotate_by_axis(axis, angle);

        self.x = mat * self.x;
        self.y = mat * self.y;
        self.z = mat * self.z;
    }

    pub fn translate(&mut self, by: Vec3) {
        self.pos += by;
    }

    pub fn as_basis_mat(&self) -> Mat3 {
        Mat3::new(
            self.x.x, self.x.y, self.x.z,
            self.y.x, self.y.y, self.y.z,
            self.z.x, self.z.y, self.z.z,
        ).transpose()
    }

    pub fn as_4d_basis_mat(&self) -> Mat4 {
        Mat4::new(
            self.x.x, self.x.y, self.x.z, 0.0,
            self.y.x, self.y.y, self.y.z, 0.0,
            self.z.x, self.z.y, self.z.z, 0.0,
            0.0, 0.0, 0.0, 1.0
        ).transpose()
    }

    pub fn build_look_at_matrix(&self) -> Mat4 {
        Mat4::new(
            self.x.x, self.x.y, self.x.z, 0.0,
            self.y.x, self.y.y, self.y.z, 0.0,
            self.z.x, self.z.y, self.z.z, 0.0,
            0.0, 0.0, 0.0, 1.0
        ) *
            Mat4::new_translation(&-self.pos)
    }

    // the only new addition
    pub fn build_camera_data(&self) -> CameraData {
        CameraData {
            pos: self.pos.into(),
            basis: self.as_4d_basis_mat()
        }
    }
}