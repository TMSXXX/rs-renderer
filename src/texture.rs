use cgmath::{Vector2 as Vec2, Vector3 as Vec3};
use image::{ImageBuffer, Rgba};
use std::path::Path;

pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u32>,
}

impl Texture {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![0xFFFFFFFF; width * height],
        }
    }

    pub fn from_file(path: &Path) -> Result<Self, image::ImageError> {
        let img = image::open(path)?.to_rgba8();
        let (width, height) = img.dimensions();
        let mut data = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y);
                let color = ((pixel[0] as u32) << 24)
                    | ((pixel[1] as u32) << 16)
                    | ((pixel[2] as u32) << 8)
                    | (pixel[3] as u32);
                data.push(color);
            }
        }
        Ok(Texture {
            width: width as usize,
            height: height as usize,
            data,
        })
    }

    pub fn sample(&self, uv: Vec2<f32>) -> Vec3<f32> {
        let u = uv.x.fract();
        let v = uv.y.fract();

        let x = (u * self.width as f32) as usize;
        let y = ((1.0 - v) * self.height as f32) as usize; // 翻转V轴，使UV(0,0)对应纹理左下角

        // 3. 防止坐标越界（超出纹理尺寸）
        let x = x.min(self.width - 1);
        let y = y.min(self.height - 1);

        // 4. 获取对应像素的颜色
        self.get_pixel_color(x, y)
    }

    fn get_pixel_color(&self, x: usize, y: usize) -> Vec3<f32> {
        let color = self.data[y * self.width + x];
        Vec3::new(
            ((color >> 24) & 0xFF) as f32 / 255.0,
            ((color >> 16) & 0xFF) as f32 / 255.0,
            ((color >> 8) & 0xFF) as f32 / 255.0,
        )
    }
}
