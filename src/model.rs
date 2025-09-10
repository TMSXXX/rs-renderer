use crate::vertex::{ColoredVertex, Triangle};
use cgmath::{Matrix4 as Mat4, SquareMatrix, Vector3 as Vec3};
use obj::Obj;
use std::path::Path;

pub fn load_obj(path: &Path) -> Result<Vec<Triangle>, Box<dyn std::error::Error>> {
    let obj = Obj::load(Path::new(path)).expect("无法加载OBJ文件");
    let mut triangles = Vec::new();

    for object in obj.data.objects {
        for group in object.groups {
            for poly in group.polys {
                // 只处理三角形（确保多边形有3个顶点）
                if poly.0.len() == 3 {
                    let mut vertices = [ColoredVertex::default(); 3];
                    for (i, idx) in poly.0.iter().enumerate() {
                        // 关键修正：顶点数据从 obj.data.position 读取（而非 vertices）
                        let pos = obj.data.position[idx.0];
                        vertices[i] = ColoredVertex {
                            pos: Vec3::new(pos[0] as f32, pos[1] as f32, pos[2] as f32),
                            color: Vec3::new(0.8, 0.8, 0.8), // 默认灰色
                        };
                    }
                    triangles.push(Triangle::new(
                        vertices[0],
                        vertices[1],
                        vertices[2],
                    ));
                }
            }
        }
    }
    Ok(triangles)
}