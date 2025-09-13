use cgmath::{Deg, Matrix, Matrix4 as Mat4, Point3, Rad, Transform, Vector3 as Vec3};
use cgmath::Angle;
use cgmath::prelude::*;

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
        let tan_half_fovy =  Rad::from(Deg(fovy / 2.0)).tan();  //时刻注意角度值和弧度制的转换
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
    pub eye: Vec3<f32>,      // 位置

    //前+上+右方向量构建以相机为中心的右手坐标系
    pub front: Vec3<f32>, 
    pub up: Vec3<f32>,
    pub right: Vec3<f32>, 

    world_up: Vec3<f32>, // 世界坐标的上方向，用来辅助计算

    // 将欧拉角从弧度制改为角度制，便于输入
    pub yaw: Deg<f32>,
    pub pitch: Deg<f32>,
    pub roll: Deg<f32>,
}

impl Camera {
    
    pub fn new(position: Vec3<f32>, near: f32, far: f32, aspect: f32, fovy: f32) -> Self {
        let mut camera = Self {
            frustum: Frustum::new(near, aspect, far, fovy),
            eye: position,
            front: Vec3::new(0.0, 0.0, -1.0),
            up: Vec3::zero(),
            right: Vec3::zero(),
            world_up: Vec3::unit_y(),

            yaw: Deg(-90.0), //偏航 绕Y轴
            pitch: Deg(0.0), //俯仰 绕X轴
            roll: Deg(0.0), //翻滚 绕Z轴
        };
        camera.update_camera_vectors();
        camera
    }

    //根据 yaw 和 pitch 初始化方向向量
    pub fn update_camera_vectors(&mut self) {
        let yaw_rad = Rad::from(self.yaw);
        let pitch_rad = Rad::from(self.pitch);

        let front_x = yaw_rad.cos() * pitch_rad.cos();
        let front_y = pitch_rad.sin();
        let front_z = yaw_rad.sin() * pitch_rad.cos();
        
        let front_no_roll = Vec3::new(front_x, front_y, front_z).normalize();
        self.front = front_no_roll;

        self.right = self.front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();

        //最后应用roll
        let roll_rad = Rad::from(self.roll);
        if roll_rad.0.abs() > 0.001 {
            let roll_mat = Mat4::from_axis_angle(self.front, roll_rad);
            self.up = (roll_mat * self.up.extend(0.0)).truncate();
            self.right = (roll_mat * self.right.extend(0.0)).truncate();
        }
    }

    pub fn get_frustum(&self) -> &Frustum {
        &self.frustum
    }

    pub fn set_position(&mut self, position: Vec3<f32>) {
        self.eye = position; 
    }
    
    //用于设置初始的相机朝向
    pub fn set_rotation(&mut self, yaw: Deg<f32>, pitch: Deg<f32>, roll: Deg<f32>) {
        self.yaw = yaw;
        self.pitch = pitch;
        self.roll = roll;
        //进行范围限制
        let pitch_limit = Deg(89.0);
        if self.pitch > pitch_limit { self.pitch = pitch_limit; }
        if self.pitch < -pitch_limit { self.pitch = -pitch_limit; }

        self.update_camera_vectors();
    }

    //用于调整相机朝向
    pub fn process_rotation(&mut self, yaw_offset: Deg<f32>, pitch_offset: Deg<f32>, roll_offset: Deg<f32>) {
        self.yaw += yaw_offset;
        self.pitch += pitch_offset;
        self.roll += roll_offset;

        //进行范围限制
        let pitch_limit = Deg(89.0);
        if self.pitch > pitch_limit { self.pitch = pitch_limit; }
        if self.pitch < -pitch_limit { self.pitch = -pitch_limit; }

        self.update_camera_vectors();
    }
    
    pub fn get_view_mat(&self) -> Mat4<f32> {
        // 目标点 = 当前位置 + 前方向量
        Mat4::look_at_rh(
            Point3::from_vec(self.eye), 
            Point3::from_vec(self.eye + self.front), 
            self.up
        )
    }

    pub fn get_view_proj_mat(&self) -> Mat4<f32> {
        self.frustum.get_mat() * self.get_view_mat()
    }
}