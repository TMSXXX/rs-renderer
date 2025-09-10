use cgmath::{Vector2 as Vec2, Vector3 as Vec3, Vector4 as Vec4};

/// 带颜色信息的顶点（用于插值计算）
#[derive(Debug, Clone, Copy)]
pub struct ColoredVertex {
    pub pos: Vec3<f32>,
    pub color: Vec3<f32>,
}

/// 光栅化阶段的 2D 点（带颜色和深度）
#[derive(Debug, Clone, Copy)]
pub struct RasterPoint {
    pub pos: Vec2<f32>, 
    pub color: Vec3<f32>,
    pub z: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub vertices: [ColoredVertex; 3],
}

impl Triangle {
    pub fn new(v0: ColoredVertex, v1: ColoredVertex, v2: ColoredVertex) -> Self {
        Self {
            vertices: [v0, v1, v2],
        }
    }

    pub fn get_center(&self) -> Vec3<f32> {
        (self.vertices[0].pos + self.vertices[1].pos + self.vertices[2].pos) / 3.0
    }
}

impl Default for ColoredVertex {
    fn default() -> Self {
        ColoredVertex {
            pos: Vec3::new(0.0, 0.0, 0.0),
            color: Vec3::new(0.0, 0.0, 0.0),
        }
    }
}