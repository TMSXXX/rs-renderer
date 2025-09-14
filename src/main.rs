use cgmath::Vector4;

mod camera;
mod framebuffer;
mod model;
mod rasterizer;
mod renderer;
mod texture;
mod vertex;
mod sandbox;
mod json_struct;
//mod renderer_debug; // 已经被迁移出去的旧函数 

const WINDOW_HEIGHT: usize = 1080;
const WINDOW_WIDTH: usize = 1920;
const BLUE: Vector4<f32> = Vector4::new(0.5, 0.55, 0.7, 1.0);
const BLACK: Vector4<f32> = Vector4::new(0., 0., 0., 1.0);
const FAR_PLANE: f32 = 100.;
const NEAR_PLANE: f32 = 5.;



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = sandbox::run_json();
    Ok(())
}
