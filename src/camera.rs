use cgmath::{Matrix, Matrix4 as Mat4, Rad, Vector3 as Vec3};

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
    eye: Vec3<f32>,       // 摄像机位置
    yaw: Rad<f32>,        // 偏航角（绕Y轴，单位：弧度）
    pitch: Rad<f32>,      // 俯仰角（绕X轴，单位：弧度）
    roll: Rad<f32>,       // 翻滚角（绕Z轴，单位：弧度）
}

impl Camera {
    pub fn new(near: f32, far: f32, aspect: f32, fovy: f32) -> Self {
        Self {
            frustum: Frustum::new(near, aspect, far, fovy),
            eye: Vec3::new(0.0, 0.0, 0.0),  // 默认位置
            yaw: Rad(0.0),                  // 初始无偏航
            pitch: Rad(0.0),                // 初始无俯仰
            roll: Rad(0.0),                 // 初始无翻滚
        }
    }

    pub fn get_frustum(&self) -> &Frustum {
        &self.frustum
    }

    pub fn set_position(&mut self, pos: Vec3<f32>) {
        self.eye = pos;
    }

    // 新增：设置欧拉角（弧度）
    pub fn set_rotation(&mut self, yaw: Rad<f32>, pitch: Rad<f32>, roll: Rad<f32>) {
        self.yaw = yaw;
        self.pitch = pitch;
        self.roll = roll;
    }

    pub fn get_view_matrix(&self) -> Mat4<f32> {
        let rotation = Mat4::from_angle_x(self.pitch)  // 俯仰（X轴）
            * Mat4::from_angle_y(self.yaw)             // 偏航（Y轴）
            * Mat4::from_angle_z(self.roll);           // 翻滚（Z轴）

        let translation = Mat4::from_translation(-self.eye);
        translation * rotation.transpose()
    }
}