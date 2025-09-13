use cgmath::{Vector3 as Vec3};
use serde::Deserialize;
use crate::vertex::{Material, Triangle};

#[derive(Debug, Deserialize)]
pub struct JsonConfig {
    pub models: Vec<ModelConfig>,
    pub camera: CameraConfig,
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
}

