use cgmath::{InnerSpace, Matrix, Matrix4 as Mat4, SquareMatrix, Vector2 as Vec2, Vector3 as Vec3, Zero};
use crate::renderer::Renderer;

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub ambient: Vec3<f32>,    // 环境光反射率（通常与漫反射相同）
    pub diffuse: Vec3<f32>,    // 漫反射率（影响物体基础颜色）
    pub specular: Vec3<f32>,   // 高光颜色（金属常用光源色，塑料常用白色）
    pub specular_strength: f32, // 高光强度（0~1）
    pub shininess: f32,        // 反光度（值越大高光越集中）
}

impl Material {
    // 金属材质（高高光强度，高反光度）
    pub fn metal() -> Self {
        Self {
            ambient: Vec3::new(0.2, 0.2, 0.2),
            diffuse: Vec3::new(0.8, 0.8, 0.8),
            specular: Vec3::new(1.0, 1.0, 1.0), // 金属高光接近光源色
            specular_strength: 0.9,
            shininess: 128.0,
        }
    }

    // 塑料材质（中等高光强度，低反光度）
    pub fn plastic() -> Self {
        Self {
            ambient: Vec3::new(0.1, 0.1, 0.1),
            diffuse: Vec3::new(0.5, 0.5, 0.5),
            specular: Vec3::new(0.8, 0.8, 0.8), // 塑料高光偏白
            specular_strength: 0.5,
            shininess: 32.0,
        }
    }

    // 木材材质（低高光强度）
    pub fn wood() -> Self {
        Self {
            ambient: Vec3::new(0.3, 0.2, 0.1),
            diffuse: Vec3::new(0.6, 0.4, 0.2),
            specular: Vec3::new(0.2, 0.2, 0.2), // 木材高光很弱
            specular_strength: 0.1,
            shininess: 8.0,
        }
    }
}


/// 带颜色信息的顶点（用于插值计算）
#[derive(Debug, Clone, Copy)]
pub struct ColoredVertex {
    pub pos: Vec3<f32>,
    pub color: Vec3<f32>,
    pub normal: Vec3<f32>,
    pub uv: Vec2<f32>,
}
impl Default for ColoredVertex {
    fn default() -> Self {
        ColoredVertex {
            pos: Vec3::new(0.0, 0.0, 0.0),
            color: Vec3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(0.0, 1.0, 0.0),
            uv: Vec2::new(0., 0.),
        }
    }
}
impl ColoredVertex {
    pub fn zero() -> Self {
        ColoredVertex {
            pos: Vec3::zero(),
            color: Vec3::zero(),
            normal: Vec3::zero(),
            uv: Vec2::zero(),
        }
    }
}
/// 光栅化阶段的 2D 点（带颜色和深度）
#[derive(Debug, Clone, Copy)]
pub struct RasterPoint {
    pub pos: Vec2<f32>,
    pub world_pos: Vec3<f32>,
    pub color: Vec3<f32>,
    pub normal: Vec3<f32>,
    pub z: f32,
    pub uv: Vec2<f32>,
}

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub vertices: [ColoredVertex; 3],
    pub normal: Vec3<f32>,
    pub material: Material,
}

pub struct RasterTriangle {
    pub vertices: [RasterPoint; 3],
    pub material: Material,
}



impl Triangle {
    fn compute_normal(v0: &ColoredVertex, v1: &ColoredVertex, v2: &ColoredVertex) -> Vec3<f32> {
        let edge1 = v1.pos - v0.pos;
        let edge2 = v2.pos - v0.pos;
        edge1.cross(edge2).normalize()
    }
    pub fn new(v0: ColoredVertex, v1: ColoredVertex, v2: ColoredVertex, material: &Material) -> Self {
        let normal = Self::compute_normal(&v0, &v1, &v2);
        let material = material.clone();
        Self {
            vertices: [v0, v1, v2],
            normal: normal,
            material,
        }
    }
    pub fn get_center(&self) -> Vec3<f32> {
        (self.vertices[0].pos + self.vertices[1].pos + self.vertices[2].pos) / 3.0
    }

    pub fn get_normal(&self) -> Vec3<f32> {
        Self::compute_normal(&self.vertices[0], &self.vertices[1], &self.vertices[2])
    }
    pub fn is_backface_world_space(&self, camera_pos: Vec3<f32>, model_matrix: &Mat4<f32>) -> bool {
        // 1. 变换法线到世界空间（使用法线矩阵：模型矩阵的逆转置）
        let normal_matrix = model_matrix.invert().unwrap().transpose();
        let world_normal = (normal_matrix * self.normal.extend(0.0)).truncate().normalize();
        
        // 2. 计算三角形中心（更稳定，避免单个顶点偏差）
        let center_world = *model_matrix * self.get_center().extend(1.0);
        let center_world = Vec3::new(center_world.x, center_world.y, center_world.z);
        
        // 3. 视线方向：从三角形中心指向相机
        let view_dir = (camera_pos - center_world).normalize();
        
        // 4. 点积 < 0 说明法线背离相机，视为背面
        world_normal.dot(view_dir) < 0.0
    }
}


