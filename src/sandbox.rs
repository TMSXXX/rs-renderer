use cgmath::{
    Array, Deg, Matrix4 as Mat4, Rad, SquareMatrix, Vector2 as Vec2, Vector3 as Vec3, Zero,
};
use serde_json::from_reader;
use std::{error::Error, f32::consts::PI, fs::File, path::Path};

use crate::{
    BLUE, FAR_PLANE, NEAR_PLANE, WINDOW_HEIGHT, WINDOW_WIDTH,
    camera::{self, Camera},
    json_struct::{CameraConfig, JsonConfig, LightConfig, ModelConfig},
    model::load_obj,
    renderer::Renderer,
    texture,
    vertex::{ColoredVertex, Material, Triangle},
};

fn match_material(string: &str) -> Material {
    match string {
        "plastic" => Material::plastic(),
        "metal" => Material::metal(),
        "wood" => Material::wood(),
        _ => {
            println!("无此种材质预设，将默认使用塑料材质");
            Material::plastic()
        }
    }
}

pub fn parse_json(
    path: &Path,
) -> Result<(CameraConfig, Vec<ModelConfig>, LightConfig), Box<dyn std::error::Error>> {
    let file = File::open(Path::new(path))?;
    let config: JsonConfig = from_reader(file)?;
    println!("成功获取json");
    Ok((config.camera, config.models, config.light))
}

pub fn run_json() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        return Err("参数不足！使用方式: program <json路径> <着色器方法> <ssaa倍数>".into());
    }

    // 从命令行读取SSAA值（第4个参数，索引为3）
    let ssaa_scale: usize = match args[3].parse() {
        Ok(val) => val,
        Err(_) => return Err("SSAA值必须是正整数（如2、4）".into()),
    };
    // 验证SSAA值有效性（通常为2、4等倍数）
    if ssaa_scale < 1 {
        return Err("SSAA值必须大于等于1".into());
    }
    let width = 1920 * ssaa_scale;
    let height = 1080 * ssaa_scale;
    let shader_method = args[2].clone();
    let path = args[1].clone();
    let (camera_config, models_config, light_config) = parse_json(Path::new(&path)).unwrap();
    let c_position: Vec3<f32> = camera_config.position.into();
    let c_rotation = camera_config.angle.map(|v| Deg(v)).into();
    println!("相机角度：{:?}", c_rotation);

    let mut camera = set_camera(c_position, c_rotation);

    let mut renderer = Renderer::new(camera, width, height);
    renderer.light.set_light(light_config.color, light_config.direction);
    renderer.framebuffer.clear(BLUE);
    println!("初始化完成");
    for model_config in models_config {
        let mut model = load_obj(
            std::path::Path::new(&model_config.path),
            &match_material(&model_config.material),
        )?;

        println!("成功读取模型");
        let texture_owner: Option<texture::Texture> = if model_config.tex_path.is_empty() {
            None
        } else {
            Some(texture::Texture::from_file(std::path::Path::new(
                &model_config.tex_path,
            ))?)
        };
        println!("成功读取材质");
        let [rx, ry, rz] = model_config.angle;
        let rotation_mat =
            Mat4::from_angle_x(Deg(rx)) * Mat4::from_angle_y(Deg(ry)) * Mat4::from_angle_z(Deg(rz));
        let model_mat = rotation_mat
            * Mat4::from_translation(model_config.position.into())
            * Mat4::from_scale(model_config.scale);
        println!("开始渲染");
        renderer.render_colored_triangles(
            &mut model,
            &model_mat,
            texture_owner.as_ref(),
            &shader_method,
        );
        println!("成功渲染一模型");
    }
    // let mut floor = create_floor();
    // renderer.render_colored_triangles(&mut floor, &Mat4::from_translation(Vec3::new(0., -10., -30.)), None);
    // println!("已绘制地板");
    if shader_method == "ink" {
        renderer.draw_color_outline_sobel(0.6, 1);
        renderer.draw_depth_outline_sobel(0.1, 2);
    }
    if shader_method == "toon" {
        renderer.draw_color_outline_sobel(0.6, 1);
        renderer.draw_depth_outline_sobel(0.1, 2);
    }
    let _ = renderer.framebuffer.ssaa(ssaa_scale).save_as_image("output1.png")?;
    println!("已渲染完成");
    Ok(())
}

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
    let size = 40.0;
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

            let v0 = ColoredVertex {
                pos: Vec3::new(x0, -3., z0),
                color: if (x_idx + z_idx) % 2 == 0 {
                    color1
                } else {
                    color2
                },
                normal: Vec3::new(0.0, 1.0, 0.0),
                uv: Vec2::new(0.0, 0.0),
            };
            let v1 = ColoredVertex {
                pos: Vec3::new(x1, -3., z0),
                color: if (x_idx + z_idx) % 2 == 0 {
                    color1
                } else {
                    color2
                },
                normal: Vec3::new(0.0, 1.0, 0.0),
                uv: Vec2::new(1.0, 0.0),
            };
            let v2 = ColoredVertex {
                pos: Vec3::new(x1, -3., z1),
                color: if (x_idx + z_idx) % 2 == 0 {
                    color1
                } else {
                    color2
                },
                normal: Vec3::new(0.0, 1.0, 0.0),
                uv: Vec2::new(1.0, 1.0),
            };
            let v3 = ColoredVertex {
                pos: Vec3::new(x0, -3., z1),
                color: if (x_idx + z_idx) % 2 == 0 {
                    color1
                } else {
                    color2
                },
                normal: Vec3::new(0.0, 1.0, 0.0),
                uv: Vec2::new(0.0, 1.0),
            };

            triangles.push(Triangle {
                vertices: [v0, v1, v2],
                normal: Vec3::new(0.0, 1.0, 0.0),
                material: Material::metal(),
            });
            triangles.push(Triangle {
                vertices: [v2, v3, v0],
                normal: Vec3::new(0.0, 1.0, 0.0),
                material: Material::metal(),
            });
        }
    }
    triangles
}

pub fn set_camera(position: Vec3<f32>, rotation: Vec3<Deg<f32>>) -> Camera {
    let mut camera = Camera::new(
        Vec3::zero(), //初始值保持为0
        NEAR_PLANE,
        FAR_PLANE,
        WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        45.0, //现在可以直接传入镜头角度
    );
    camera.set_position(position);
    camera.set_rotation(rotation.x, rotation.y, rotation.z);
    camera
}
