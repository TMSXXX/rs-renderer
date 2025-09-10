use crate::vertex::{ColoredVertex, RasterPoint};
use cgmath::{Matrix4 as Mat4, Vector2 as Vec2, Vector3 as Vec3, Vector4 as Vec4, dot};

pub fn get_barycentric_coords(
    vertices: &[Vec2<f32>; 3],
    p: &Vec2<f32>,
) -> Option<(f32, f32, f32)> {
    let v0 = vertices[1] - vertices[0];
    let v1 = vertices[2] - vertices[0];
    let v2 = *p - vertices[0];

    let d00 = dot(v0, v0);
    let d01 = dot(v0, v1);
    let d11 = dot(v1, v1);
    let d20 = dot(v2, v0);
    let d21 = dot(v2, v1);

    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < 1e-6 {
        return None; // 三角形面积为零，无法计算重心坐标
    }

    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;

    Some((u, v, w))
}

pub fn interpolate_depth(
    points: &[RasterPoint; 3],  // 带颜色的三角形三个顶点（屏幕空间）
    bary: (f32, f32, f32)       // 重心坐标 (u, v, w)
) -> f32 {
    let (u, v, w) = bary;
    // 深度 = u*v0_depth + v*v1_depth + w*v2_depth
    points[0].z * w + points[1].z * v + points[2].z * u
}

pub fn interpolate_color(
    points: &[RasterPoint; 3],  // 带颜色的三角形三个顶点（屏幕空间）
    bary: (f32, f32, f32)       // 重心坐标 (u, v, w)
) -> Vec3<f32> {
    let (u, v, w) = bary;
    // 颜色 = u*v0_color + v*v1_color + w*v2_color
    points[0].color * w + points[1].color * v + points[2].color * u
}


pub fn get_box(vertices: &[Vec2<f32>; 3]) -> (i32, i32, i32, i32) {
    let mut min_x = vertices[0].x;
    let mut max_x = vertices[0].x;
    let mut min_y = vertices[0].y;
    let mut max_y = vertices[0].y;

    for v in vertices.iter().skip(1) {
        if v.x < min_x {
            min_x = v.x;
        }
        if v.x > max_x {
            max_x = v.x;
        }
        if v.y < min_y {
            min_y = v.y;
        }
        if v.y > max_y {
            max_y = v.y;
        }
    }

    (
        min_x.floor() as i32,
        min_y.floor() as i32,
        max_x.ceil() as i32,
        max_y.ceil() as i32,
    )
}

pub fn is_inside_triangle(vertices: &[Vec2<f32>; 3], p: &Vec2<f32>) -> bool {
    let v0 = vertices[1] - vertices[0];
    let v1 = vertices[2] - vertices[1];
    let v2 = vertices[0] - vertices[2];

    let p0 = *p - vertices[0];
    let p1 = *p - vertices[1];
    let p2 = *p - vertices[2];

    let cross0 = v0.x * p0.y - v0.y * p0.x;
    let cross1 = v1.x * p1.y - v1.y * p1.x;
    let cross2 = v2.x * p2.y - v2.y * p2.x;

    (cross0 >= 0.0 && cross1 >= 0.0 && cross2 >= 0.0)
        || (cross0 <= 0.0 && cross1 <= 0.0 && cross2 <= 0.0)
}
