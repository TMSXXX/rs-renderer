use cgmath::{Matrix4 as Mat4, Rad, SquareMatrix, Vector2 as Vec2, Vector3 as Vec3};
use std::{error::Error, f32::consts::PI};

use crate::{
    BLUE, FAR_PLANE, NEAR_PLANE, WINDOW_HEIGHT, WINDOW_WIDTH, camera::{self, Camera}, model::load_obj, renderer::Renderer, texture, vertex::{ColoredVertex, Material, Triangle}
};

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
    // 3. 移回原位
    let translate_back = Mat4::from_translation(center);

    translate_back * rotate * translate_to_origin
}

pub fn create_floor() -> Vec<Triangle> {
    let mut triangles = Vec::new();
    let size = 30.0;
    let cell_count = 10;
    let half_size = size / 2.0;
    let cell_size = size / cell_count as f32;

    let color1 = Vec3::new(0.5, 0.5, 0.5);
    let color2 = Vec3::new(0.3, 0.3, 0.3);

    for z_idx in 0..cell_count {
        for x_idx in 0..cell_count {
            let x0 = -half_size + x_idx as f32 * cell_size;
            let x1 = x0 + cell_size;
            let z0 = -half_size + z_idx as f32 * cell_size;
            let z1 = z0 + cell_size;

            let v0 = ColoredVertex { pos: Vec3::new(x0, -3., z0), color: if (x_idx + z_idx) % 2 == 0 { color1 } else { color2 }, normal: Vec3::new(0.0, 1.0, 0.0), uv: Vec2::new(0.0, 0.0) };
            let v1 = ColoredVertex { pos: Vec3::new(x1, -3., z0), color: if (x_idx + z_idx) % 2 == 0 { color1 } else { color2 }, normal: Vec3::new(0.0, 1.0, 0.0), uv: Vec2::new(1.0, 0.0) };
            let v2 = ColoredVertex { pos: Vec3::new(x1, -3., z1), color: if (x_idx + z_idx) % 2 == 0 { color1 } else { color2 }, normal: Vec3::new(0.0, 1.0, 0.0), uv: Vec2::new(1.0, 1.0) };
            let v3 = ColoredVertex { pos: Vec3::new(x0, -3., z1), color: if (x_idx + z_idx) % 2 == 0 { color1 } else { color2 }, normal: Vec3::new(0.0, 1.0, 0.0), uv: Vec2::new(0.0, 1.0) };

            triangles.push(Triangle { vertices: [v0, v1, v2], normal: Vec3::new(0.0, 1.0, 0.0), material: Material::plastic() });
            triangles.push(Triangle { vertices: [v2, v3, v0], normal: Vec3::new(0.0, 1.0, 0.0), material: Material::plastic() });
        }
    }
    triangles
}

pub fn set_camera() -> Camera {
    let mut camera = Camera::new(
        NEAR_PLANE,
        FAR_PLANE,
        WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        (45. as f32).to_radians(),
    );
    camera.set_position(Vec3::new(0., 10., 0.));
    camera
}

pub fn run_app() -> Result<(), Box<dyn Error>> {
    // 初始设置
    let width = 3200;
    let height = 2400;
    let mut camera = set_camera();
    let mut renderer = Renderer::new(camera, width, height);
    renderer.framebuffer.clear(BLUE);

    // 加载模型
    let mut floor = create_floor();
    let mut model1 = load_obj(
        std::path::Path::new("./models/miku_race.obj"),
        &Material::metal(),
    )?;
    let mut model2 = load_obj(std::path::Path::new("./models/bunny_10k.obj"), &Material::metal())?;
    let tex_idx = texture::Texture::from_file(std::path::Path::new("./models/miku_race.jpg"))?;

    for i in 0..120 {
        println!("渲染第{}帧, 相机角度:{:?}",i, renderer.camera.yaw);
        let model_mat = rotate_around_self(PI / 60. * (i) as f32, Vec3::new(-0.2, 0., -5.0));
        let model_mat2 = rotate_around_self(PI / 60. * (i) as f32, Vec3::new(-0.2, 0., -5.0));
        renderer.framebuffer.clear(BLUE);
        renderer.render_colored_triangles(&mut model1, &(model_mat*Mat4::from_scale(0.6)*Mat4::from_translation(Vec3::new(-0.2, 0., -5.0))), Some(&tex_idx));
        renderer.render_colored_triangles(&mut model2, &(&model_mat2*Mat4::from_translation(Vec3::new(-5., 2.0, -6.0))), None);
        //renderer.render_colored_triangles(&mut floor, &Mat4::identity(), None);
        renderer.draw_depth_outline_prewitt(0.1, 2);
        let path = format!("./src/output/test_{}.png", i);
        let _ = renderer.framebuffer.ssaa(2).save_as_image(&path);
        //let _ = renderer.framebuffer.save_depth_as_image(&format!("./src/output/test_depth_{}.png", i));
    }

    Ok(())
}