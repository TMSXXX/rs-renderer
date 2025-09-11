use crate::vertex::{ColoredVertex, Triangle};
use cgmath::{InnerSpace, Matrix4 as Mat4, SquareMatrix, Vector3 as Vec3, Vector2 as Vec2, Zero};
use obj::Obj;
use std::path::Path;

pub fn load_obj(path: &Path) -> Result<Vec<Triangle>, Box<dyn std::error::Error>> {
    let obj = Obj::load(Path::new(path)).expect("无法加载OBJ文件");
    let mut triangles = Vec::new();
    for object in obj.data.objects {
        for group in object.groups {
            for poly in group.polys {
                if poly.0.len() == 3 {
                    let mut vertices = [ColoredVertex::default(); 3];
                    for (i, idx) in poly.0.iter().enumerate() {
                        // 获取位置
                        let pos = obj.data.position[idx.0];

                        // 获取法线
                        let normal = if let Some(normal_idx) = idx.2 {
                            let n = obj.data.normal[normal_idx];
                            Vec3::new(n[0] as f32, n[1] as f32, n[2] as f32).normalize()
                        } else {
                            Vec3::zero() // 如果没有法线索引，使用零向量
                        };
                        // 获取UV坐标
                        let uv = if let Some(uv_idx) = idx.1 {
                            let tex_coord = obj.data.texture[uv_idx];
                            Vec2::new(tex_coord[0] as f32, tex_coord[1] as f32)
                        } else {
                            Vec2::new(0.0, 0.0) // 无UV时默认(0,0)
                        };
                        vertices[i] = ColoredVertex {
                            pos: Vec3::new(pos[0] as f32, pos[1] as f32, pos[2] as f32),
                            color: Vec3::new(0.8, 0.8, 0.8), // 默认灰色
                            normal,
                            uv,
                        };
                    }
                    triangles.push(Triangle::new(vertices[0], vertices[1], vertices[2]));
                }
            }
        }
    }
    Ok(triangles)
}

