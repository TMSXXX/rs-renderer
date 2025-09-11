use crate::vertex::{ColoredVertex, Triangle};
use cgmath::{InnerSpace, Matrix4 as Mat4, SquareMatrix, Vector3 as Vec3, Zero};
use obj::Obj;
use std::path::Path;

pub fn load_obj(path: &Path) -> Result<Vec<Triangle>, Box<dyn std::error::Error>> {
    let obj = Obj::load(Path::new(path)).expect("无法加载OBJ文件");
    let mut triangles = Vec::new();

    // 先收集所有顶点位置
    let positions: Vec<Vec3<f32>> = obj.data.position.iter()
        .map(|pos| Vec3::new(pos[0] as f32, pos[1] as f32, pos[2] as f32))
        .collect();

    // 计算每个顶点的法线（平均相邻面的法线）
    let mut normals = vec![Vec3::zero(); positions.len()];
    let mut face_count = vec![0; positions.len()];

    // 使用引用迭代，不获取所有权
    for object in &obj.data.objects {
        for group in &object.groups {
            for poly in &group.polys {
                if poly.0.len() == 3 {
                    let indices: Vec<usize> = poly.0.iter().map(|idx| idx.0).collect();
                    
                    // 计算面法线
                    let v0 = positions[indices[0]];
                    let v1 = positions[indices[1]];
                    let v2 = positions[indices[2]];
                    let face_normal = (v1 - v0).cross(v2 - v0).normalize();
                    
                    // 累加到顶点法线
                    for &idx in &indices {
                        normals[idx] += face_normal;
                        face_count[idx] += 1;
                    }
                }
            }
        }
    }

    // 归一化顶点法线
    for i in 0..normals.len() {
        if face_count[i] > 0 {
            normals[i] = normals[i].normalize();
        }
    }

    // 创建三角形（同样使用引用迭代）
    for object in &obj.data.objects {
        for group in &object.groups {
            for poly in &group.polys {
                if poly.0.len() == 3 {
                    let mut vertices = [ColoredVertex::default(); 3];
                    for (i, idx) in poly.0.iter().enumerate() {
                        let mut pos = positions[idx.0];
                        if idx.0 == 1 {
                            pos = positions[idx.0 - 1];
                        }
                        
                        vertices[i] = ColoredVertex {
                            pos,
                            color: Vec3::new(0.8, 0.8, 0.8), // 默认灰色
                            normal: normals[idx.0], // 使用计算的法线
                        };
                    }
                    triangles.push(Triangle::new(vertices[0], vertices[1], vertices[2]));
                }
            }
        }
    }
    Ok(triangles)
}