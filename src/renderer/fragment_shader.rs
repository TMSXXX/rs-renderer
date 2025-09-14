use cgmath::{ElementWise, InnerSpace, Vector2 as Vec2, Vector3 as Vec3};
use rand::Rng;

use crate::renderer::Light; // 从 renderer 模块导入 Light
use crate::texture::Texture;
use crate::vertex::Material;

#[derive(Debug)]
pub struct FragmentData<'a> {
    pub world_pos: Vec3<f32>,
    pub normal: Vec3<f32>,
    pub uv: Vec2<f32>,
    pub color: Vec3<f32>, // 顶点颜色插值结果
    pub texture: Option<&'a Texture>,
    pub material: &'a Material,
    pub camera_pos: Vec3<f32>,
}

// 定义 Shader 的通用行为
pub trait FragmentShader: Sync {
    // 输入插值后的片元数据，输出最终的颜色 (0.0 ~ 1.0 范围的 Vec3)
    fn shade(&self, data: FragmentData) -> Vec3<f32>;
}

//非线性漫反射：卡通风格渲染
pub struct ToonShader {
    pub light: Light,
}

impl FragmentShader for ToonShader {
    fn shade(&self, data: FragmentData) -> Vec3<f32> {
        // 优先使用纹理颜色作为基础色
        let mut base_color = data.color;
        if let Some(tex) = data.texture {
            base_color = tex.sample(data.uv);
        }

        // 1. 环境光分量 (保持不变)
        let ambient = self.light.ambient_color * self.light.ambient_strength;

        // 2. 卡通风格的漫反射分量 (核心部分)
        let light_dir = self.light.direction.normalize();
        let diff = data.normal.dot(-light_dir).max(0.0);
        let diffuse = if diff > 0.6 {
            self.light.color * self.light.intensity * 1.1
        } else if diff > 0.2 {
            self.light.color * self.light.intensity * 0.8
        } else {
            self.light.color * self.light.intensity * 0.5
        };

        // 3. 高光分量 (保持不变，卡通渲染也可以有高光)
        let specular = {
            // 视线方向（从像素到相机）
            let view_dir = (data.camera_pos - data.world_pos).normalize();
            // 半程向量
            let half_dir = (-light_dir + view_dir).normalize();
            // 高光强度（结合材质的反光度）
            let spec = data.normal.dot(half_dir).max(0.0);
            let spec = spec.powf(data.material.shininess);
            // 高光颜色 = 光源色 * 材质高光色 * 材质高光强度 * 计算值
            self.light.color.mul_element_wise(data.material.specular)
                * data.material.specular_strength
                * spec
        };

        // 合并光照
        let final_lighting = ambient + diffuse + specular;
        let mut final_color = base_color.mul_element_wise(final_lighting);

        // Clamping
        final_color.x = final_color.x.clamp(0.0, 1.0);
        final_color.y = final_color.y.clamp(0.0, 1.0);
        final_color.z = final_color.z.clamp(0.0, 1.0);

        final_color
    }
}

//经典冯模型
pub struct PhongShader {
    pub light: Light,
}

impl<'a> FragmentShader for PhongShader {
    fn shade(&self, data: FragmentData) -> Vec3<f32> {
        // 优先使用纹理颜色
        let mut base_color = data.color;
        if let Some(tex) = data.texture {
            base_color = tex.sample(data.uv);
        }

        // 环境光分量 (Ambient)
        let ambient = self.light.ambient_color * self.light.ambient_strength;

        // 漫反射分量 (Diffuse)
        let light_dir = self.light.direction.normalize();
        let diff = data.normal.dot(-light_dir).max(0.0);
        let diffuse = self.light.color * self.light.intensity * diff;

        // 高光分量 (Specular)
        let mut specular = {
            let view_dir = (data.camera_pos - data.world_pos).normalize();
            let half_dir = (-light_dir + view_dir).normalize();
            let spec = data.normal.dot(half_dir).max(0.0);
            let spec = spec.powf(data.material.shininess);
            self.light.color.mul_element_wise(data.material.specular)
                * data.material.specular_strength
                * spec
        };

        let split_level = 6.0;
        specular = Vec3::new(
            (specular.x * split_level).floor() / split_level,
            (specular.y * split_level).floor() / split_level,
            (specular.z * split_level).floor() / split_level,
        );

        // 合并光照
        let final_lighting = ambient + diffuse + specular;
        let mut final_color = base_color.mul_element_wise(final_lighting);

        // 最后进行Clamping，确保颜色值在有效范围内
        final_color.x = final_color.x.clamp(0.0, 1.0);
        final_color.y = final_color.y.clamp(0.0, 1.0);
        final_color.z = final_color.z.clamp(0.0, 1.0);

        final_color
    }
}

pub struct NormalDebugShader;

impl FragmentShader for NormalDebugShader {
    fn shade(&self, data: FragmentData) -> Vec3<f32> {
        let color = (data.normal + Vec3::new(1.0, 1.0, 1.0)) * 0.5;

        color
    }
}

pub struct InkShader {
    pub light: Light,
}

impl FragmentShader for InkShader {
    fn shade(&self, data: FragmentData) -> Vec3<f32> {
        let mut base_color = data.color;
        if let Some(tex) = data.texture {
            base_color = tex.sample(data.uv);
        }
        let gray = base_color.x * 0.299 + base_color.y * 0.587 + base_color.z * 0.114;
        let gray_color = Vec3::new(gray, gray, gray);

        let ambient = self.light.ambient_color * self.light.ambient_strength;

        let light_dir = self.light.direction.normalize();
        let diff = data.normal.dot(-light_dir).max(0.0);
        let diffuse = if diff > 0.8 {
            self.light.color * self.light.intensity * 1.1
        } else if diff > 0.3 {
            self.light.color * self.light.intensity * 0.6
        } else {
            self.light.color * self.light.intensity * 0.05
        };
        let mut specular = {
            let view_dir = (data.camera_pos - data.world_pos).normalize();
            let half_dir = (-light_dir + view_dir).normalize();
            let spec = data.normal.dot(half_dir).max(0.0);
            let spec = spec.powf(data.material.shininess);
            self.light.color.mul_element_wise(data.material.specular)
                * data.material.specular_strength
                * spec
        };

        let split_level = 4.0;
        specular = Vec3::new(
            (specular.x * split_level).floor() / split_level,
            (specular.y * split_level).floor() / split_level,
            (specular.z * split_level).floor() / split_level,
        );
        let mut final_color = gray_color.mul_element_wise(ambient + diffuse + specular);

        let rnumber = rand::random_range(0..=100);
        match rnumber {
            0..2 => {
                if final_color.x < 0.2 {
                    final_color *= 0.1;
                }
            }
            10 => {
                final_color *= 2.;
            }
            _ => {}
        }

        final_color.x = final_color.x.clamp(0.0, 1.0);
        final_color.y = final_color.y.clamp(0.0, 1.0);
        final_color.z = final_color.z.clamp(0.0, 1.0);

        final_color
    }
}
