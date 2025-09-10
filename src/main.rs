mod camera;
mod framebuffer;
mod model;
mod rasterizer;
mod renderer;
mod vertex;
use std::f32::consts::PI;

use cgmath::{Matrix4 as Mat4, SquareMatrix, Vector3 as Vec3};

use crate::vertex::{ColoredVertex, Triangle};

const WINDOW_HEIGHT: usize = 720;
const WINDOW_WIDTH: usize = 1024;
const BLACK: u32 = 0xFF000000; // A=FF, R=00, G=00, B=00
const WHITE: u32 = 0xFFFFFFFF;
const RED: u32 = 0xFFFF0000;
const GREEN: u32 = 0xFF00FF00;
const BLUE: u32 = 0xFF0000FF;
const PURPLE: u32 = 0xFFFF00FF;

#[rustfmt::skip]
fn rotate_around_self(angle: f32, center: Vec3<f32>) -> Mat4<f32> {
    // 1. 平移到原点（以自身中心为参考）
    let translate_to_origin = Mat4::from_translation(-center);
    // 2. 绕Y轴旋转
    let c = angle.cos();
    let s = angle.sin();
    let rotate = Mat4::new(
        c, 0.0, s, 0.0,
        0.0, 1.0, 0.0, 0.0, 
        -s, 0.0, c, 0.0, 
        0.0, 0.0, 0.0, 1.0,
    );
    let rotate2 = Mat4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1., 0., 0.0, 
        0., 0., 1., 0.0, 
        0.0, 0.0, 0.0, 1.0,
    );
    let rotate = rotate * rotate2;
    // 3. 平移回原位置
    let translate_back = Mat4::from_translation(center);

    // 复合矩阵（顺序：先平移到原点→旋转→平移回）
    translate_back * rotate * translate_to_origin
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model_triangles = model::load_obj(std::path::Path::new("../models/bunny_10k.obj"))?;
    println!("模型三角形数量：{}", model_triangles.len());
    if !model_triangles.is_empty() {
        println!(
            "第一个三角形顶点1坐标：{:?}",
            model_triangles[0].vertices[0].pos
        );
    }

    let colored_vertices = Triangle::new(
        ColoredVertex {
            pos: Vec3::new(-1.3, -0.3, -3.0),
            color: Vec3::new(1.0, 0.0, 0.0), // 红
        },
        ColoredVertex {
            pos: Vec3::new(-0.7, -0.3, -3.0),
            color: Vec3::new(0.0, 1.0, 0.0), // 绿
        },
        ColoredVertex {
            pos: Vec3::new(-1.0, 0.3, -3.0),
            color: Vec3::new(0.0, 0.0, 1.0), // 蓝
        },
    );

    let colored_vertices_2 = Triangle::new(
        ColoredVertex {
            pos: Vec3::new(-0.3, -0.4, -4.0),
            color: Vec3::new(1.0, 1.0, 1.0), // 红
        },
        ColoredVertex {
            pos: Vec3::new(0., -0.5, -4.0),
            color: Vec3::new(0.0, 0.5, 0.2), // 绿
        },
        ColoredVertex {
            pos: Vec3::new(-0.3, 0.2, -2.0),
            color: Vec3::new(0.0, 0.0, 0.5), // 蓝
        },
    );
    let width = 3200;
    let height = 2400;
    let camera = camera::Camera::new(
        1.0,
        5.0,
        WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        (45. as f32).to_radians(),
    );

    let mut renderer = renderer::Renderer::new(camera, width, height);
    renderer.framebuffer.clear(BLACK);

    let center = colored_vertices.get_center();

    // 简单的动画循环，生成一系列旋转的帧
    for i in 0..60 {
        // 生成60帧动画
        let angle = i as f32 * 2. * PI / 30.;

        // 清空帧缓冲
        renderer.framebuffer.clear(BLACK);

        // 创建旋转矩阵
        let model = rotate_around_self(angle, center);

        // 绘制旋转后的三角形
        let raster_points = renderer.transform_colored_vertices(&colored_vertices.vertices, &model);
        // 光栅化带颜色插值的三角形
        renderer.rasterize_colored_triangle(&raster_points);

        let raster_points2 =
            renderer.transform_colored_vertices(&colored_vertices_2.vertices, &Mat4::identity());
        renderer.rasterize_colored_triangle(&raster_points2);
        let model_transform = Mat4::from_translation(Vec3::new(0.0, 0.0, -3.0))  // 平移到相机前方
            * Mat4::from_angle_y(cgmath::Rad(angle))  // 绕Y轴旋转
            * Mat4::from_scale(0.6);  // 缩放模型
        renderer.render_colored_triangles(&model_triangles, &model_transform);

        // 保存每一帧为图片
        renderer
            .framebuffer
            .ssaa(4)
            .save_to_image(&format!("output_{:03}.png", i))?;
    }

    Ok(())
}
