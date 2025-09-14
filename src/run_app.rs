// 并不使用的旧运行函数
pub fn run_app() -> Result<(), Box<dyn Error>> {
    // 初始设置
    let width = 1600;
    let height = 1200;
    let mut camera = set_camera(
        Vec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        },
        Vec3 {
            x: Deg(0.),
            y: Deg(0.),
            z: Deg(0.),
        },
    );
    let mut renderer = Renderer::new(camera, width, height);
    renderer.framebuffer.clear(BLUE);

    // 加载模型
    let mut floor = create_floor();
    let mut model1 = load_obj(
        std::path::Path::new("./models/miku_race.obj"),
        &Material::metal(),
    )?;
    let mut model2 = load_obj(
        std::path::Path::new("./models/bunny_10k.obj"),
        &Material::metal(),
    )?;
    let tex_idx = texture::Texture::from_file(std::path::Path::new("./models/miku_race.jpg"))?;

    for i in 0..120 {
        println!(
            "渲染第{}帧,\n相机坐标(X: {:?} Y: {:?} Z: {:?})\n相机角度(偏航: {:?} 俯仰: {:?} 翻滚: {:?})",
            i,
            renderer.camera.eye.x,
            renderer.camera.eye.y,
            renderer.camera.eye.z,
            renderer.camera.yaw,
            renderer.camera.pitch,
            renderer.camera.roll
        );

        renderer
            .camera
            .process_rotation(Deg(0.0), Deg(0.0), Deg(3.0));

        let model_mat = rotate_around_self(PI / 60. * (i) as f32, Vec3::new(-0.2, 0., -5.0));
        let model_mat2: Mat4<f32> =
            rotate_around_self(PI / 60. * (i) as f32, Vec3::new(-0.2, 0., -5.0));
        renderer.framebuffer.clear(BLUE);
        renderer.render_colored_triangles(
            &mut model1,
            &(model_mat
                * Mat4::from_scale(0.6)
                * Mat4::from_translation(Vec3::new(-0.2, 0., -5.0))),
            Some(&tex_idx),
        );
        renderer.render_colored_triangles(
            &mut model2,
            &(&model_mat2 * Mat4::from_translation(Vec3::new(-5., 2.0, -6.0))),
            None,
        );
        //renderer.render_colored_triangles(&mut floor, &Mat4::identity(), None);
        renderer.draw_depth_outline_prewitt(0.1, 2);
        let path = format!("./output/test_{}.png", i);
        let _ = renderer.framebuffer.ssaa(2).save_as_image(&path);
        //let _ = renderer.framebuffer.save_depth_as_image(&format!("./src/output/test_depth_{}.png", i));
    }

    Ok(())
}
