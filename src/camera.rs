use cgmath::{Deg, Matrix, Matrix4 as Mat4, Point3, Rad, Transform, Vector3 as Vec3};
use cgmath::Angle;

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
    pub(crate) eye: Vec3<f32>,
    pub(crate) at: Vec3<f32>,
    pub(crate) up: Vec3<f32>,
    pub(crate) yaw: Rad<f32>,
    pitch: Rad<f32>,
}

impl Camera {
    pub fn new(near: f32, far: f32, aspect: f32, fovy: f32) -> Self {
        Self {
            frustum: Frustum::new(near, aspect, far, fovy),
            eye: Vec3::new(0.0, 0.0, 5.0),
            at: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            yaw: Rad(0.0),
            pitch: Rad(0.0),
        }
    }

    pub fn get_frustum(&self) -> &Frustum {
        &self.frustum
    }

    pub fn set_position(&mut self, position: Vec3<f32>) {
        self.eye = position;
    }
    
    pub fn set_rotation(&mut self, angle: Rad<f32>) {
        self.yaw = angle;
    }
    
    pub fn get_view_mat(&self) -> Mat4<f32> {
        let rotation = Mat4::from_angle_y(self.yaw);
        let pos = rotation.transform_point(Point3::new(0., 0., 5.));
        
        Mat4::look_at_rh(pos, Point3::new(0., 0., 0.), Vec3::new(0., 1., 0.))
    }

    pub fn get_view_proj_mat(&self) -> Mat4<f32> {
        self.frustum.get_mat() * self.get_view_mat()
    }
}