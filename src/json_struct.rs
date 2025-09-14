use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct JsonConfig {
    pub models: Vec<ModelConfig>,
    pub camera: CameraConfig,
    pub light: LightConfig,
}

#[derive(Debug, Deserialize)]
pub struct CameraConfig {
    pub position: [f32; 3],
    pub angle: [f32; 3],
}
#[derive(Debug, Deserialize)]
pub struct ModelConfig {
    pub path: String,
    pub tex_path: String,
    pub material: String,
    pub position: [f32; 3],
    pub angle: [f32; 3],
    pub scale: f32,
}

#[derive(Debug, Deserialize)]
pub struct LightConfig {
    pub direction: [f32; 3],
    pub color: [f32; 3],
}
