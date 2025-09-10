use cgmath::Matrix4 as Mat4;

#[derive(Debug)]
pub struct Frustum {
    near: f32,
    aspect: f32,
    fovy: f32,
    far: f32,
    mat: Mat4<f32>,
}

impl Frustum {
    #[rustfmt::skip]
    pub fn new(near: f32, aspect: f32, far: f32, fovy: f32) -> Self {
        let tan_half_fovy = (fovy / 2.0).tan();
        let a = 1.0 / (aspect * tan_half_fovy);
        let b = 1.0 / tan_half_fovy;
        let c = -(far + near) / (far - near);
        let d = -2.0 * far * near / (far - near);
        
        // projection
        let mat = Mat4::new(
            a,    0.0,   0.0,   0.0,
            0.0,  b,     0.0,   0.0,
            0.0,  0.0,   c,    -1.0,
            0.0,  0.0,   d,     0.0,
        );

        Self {
            near,
            aspect,
            fovy,
            far,
            mat,
        }
    }
    pub fn get_mat(&self) -> &Mat4<f32> {
        &self.mat
    }
}

pub struct Camera {
    frustum: Frustum,
}

impl Camera {
    pub fn new(near: f32, far: f32, aspect: f32, fovy: f32) -> Self {
        Self { frustum: Frustum::new(near, aspect, far, fovy) }
    }
    pub fn get_frustum(&self) -> &Frustum {
        &self.frustum
    }
}