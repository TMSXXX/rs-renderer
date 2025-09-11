use cgmath::{InnerSpace, Vector2 as Vec2, Vector3 as Vec3, Matrix4 as Mat4, Zero};

/// 带颜色信息的顶点（用于插值计算）
#[derive(Debug, Clone, Copy)]
pub struct ColoredVertex {
    pub pos: Vec3<f32>,
    pub color: Vec3<f32>,
    pub normal: Vec3<f32>,
}

/// 光栅化阶段的 2D 点（带颜色和深度）
#[derive(Debug, Clone, Copy)]
pub struct RasterPoint {
    pub pos: Vec2<f32>,
    pub color: Vec3<f32>,
    pub normal: Vec3<f32>,
    pub z: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub vertices: [ColoredVertex; 3],
    pub normal: Vec3<f32>,
}

impl Triangle {
    fn compute_normal(v0: &ColoredVertex, v1: &ColoredVertex, v2: &ColoredVertex) -> Vec3<f32> {
        let edge1 = v1.pos - v0.pos;
        let edge2 = v2.pos - v0.pos;
        edge1.cross(edge2).normalize()
    }
    pub fn new(v0: ColoredVertex, v1: ColoredVertex, v2: ColoredVertex) -> Self {
        let normal = Self::compute_normal(&v0, &v1, &v2);
        Self {
            vertices: [v0, v1, v2],
            normal: normal,
        }
    }
    pub fn get_center(&self) -> Vec3<f32> {
        (self.vertices[0].pos + self.vertices[1].pos + self.vertices[2].pos) / 3.0
    }

    pub fn get_normal(&self) -> Vec3<f32> {
        Self::compute_normal(&self.vertices[0], &self.vertices[1], &self.vertices[2])
    }
    pub fn is_backface_world_space(&self, camera_pos: Vec3<f32>, model_matrix: &Mat4<f32>) -> bool {
        // 将三角形变换到世界空间
        let world_vertices = [
            (*model_matrix * self.vertices[0].pos.extend(1.0)).truncate(),
            (*model_matrix * self.vertices[1].pos.extend(1.0)).truncate(),
            (*model_matrix * self.vertices[2].pos.extend(1.0)).truncate(),
        ];

        // 计算世界空间法线
        let edge1 = world_vertices[1] - world_vertices[0];
        let edge2 = world_vertices[2] - world_vertices[0];
        let world_normal = edge1.cross(edge2).normalize();

        // 计算视图方向（从三角形中心指向相机）
        let center = (world_vertices[0] + world_vertices[1] + world_vertices[2]) / 3.0;
        let view_dir = (camera_pos - center).normalize();

        // 背面检测
        world_normal.dot(view_dir) <= 0.0
    }
}

impl Default for ColoredVertex {
    fn default() -> Self {
        ColoredVertex {
            pos: Vec3::new(0.0, 0.0, 0.0),
            color: Vec3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(0.0, 1.0, 0.0),
        }
    }
}
impl ColoredVertex {
    pub fn zero() -> Self {
        ColoredVertex {
            pos: Vec3::zero(),
            color: Vec3::zero(),
            normal: Vec3::zero(),
        }
    }
}
