use crate::vertex::{ClipSpaceVertex, Triangle};
use cgmath::{InnerSpace, Matrix4 as Mat4, Vector3 as Vec3, Vector4 as Vec4};


pub struct VertexShaderUniforms<'a> {
    pub model_matrix: &'a Mat4<f32>,
    pub mvp_matrix: &'a Mat4<f32>,
    pub normal_matrix: &'a Mat4<f32>,
}

pub trait VertexShader {
    // 接收一个模型空间的三角形和uniforms
    // 返回一个裁剪空间的三角形
    fn shade_triangle(
        &self,
        triangle: &Triangle,
        uniforms: &VertexShaderUniforms,
    ) -> [ClipSpaceVertex; 3];
}


pub struct DefaultVertexShader;

impl VertexShader for DefaultVertexShader {
    fn shade_triangle(
        &self,
        triangle: &Triangle,
        uniforms: &VertexShaderUniforms,
    ) -> [ClipSpaceVertex; 3] {
        triangle.vertices.map(|v| {
            ClipSpaceVertex {
                position: *uniforms.mvp_matrix * v.pos.extend(1.0),
                world_pos: (*uniforms.model_matrix * v.pos.extend(1.0)).truncate(),
                normal: (*uniforms.normal_matrix * v.normal.extend(0.0))
                    .truncate()
                    .normalize(),
                uv: v.uv,
                color: v.color,
            }
        })
    }
}