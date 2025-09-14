mod camera;
mod framebuffer;
mod model;
mod rasterizer;
mod renderer;
mod texture;
mod vertex;
mod sandbox;
mod json_struct;
mod renderer_debug; // 已经被迁移出去的旧函数 

const WINDOW_HEIGHT: usize = 1080;
const WINDOW_WIDTH: usize = 1920;
const BLUE: u32 = 0xFFA3A3F0;
const BLACK: u32 = 0xFF000000;
const FAR_PLANE: f32 = 100.;
const NEAR_PLANE: f32 = 5.;



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = sandbox::run_json();
    Ok(())
}
