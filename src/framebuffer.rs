use cgmath::{Vector3 as Vec3, Vector4 as Vec4};

use crate::{BLUE, FAR_PLANE, NEAR_PLANE};

#[derive(Clone)]
pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Vec4<f32>>,
    pub depth: Vec<f32>,
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        FrameBuffer {
            width,
            height,
            data: vec![Vec4::new(0., 0., 0., 0.); width * height],
            depth: vec![1.0; width * height],
        }
    }

    pub fn clear(&mut self, color: Vec4<f32>) {
        self.data.fill(color);
        self.depth.fill(1.0);
    }

    pub fn put_pixel(&mut self, x: usize, y: usize, color: Vec4<f32>, depth: f32) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            // 确保深度值在 [0, 1] 范围内，值越小越近
            if depth >= 0.0 && depth <= 1.0 && depth < self.depth[idx] {
                self.data[idx] = color;
                self.depth[idx] = depth;
            }
        }
    }

    pub fn ssaa(&self, factor: usize) -> Self {
        if factor == 1 {
            return self.clone();
        }
        let new_width = self.width / factor;
        let new_height = self.height / factor;
        let mut new_framebuffer = FrameBuffer::new(new_width, new_height);

        for y in 0..new_height {
            for x in 0..new_width {
                let mut r_sum = 0.;
                let mut g_sum = 0.;
                let mut b_sum = 0.;
                let mut a_sum = 0.;
                let mut count = 0;

                for fy in 0..factor {
                    for fx in 0..factor {
                        let src_x = x * factor + fx;
                        let src_y = y * factor + fy;
                        if src_x < self.width && src_y < self.height {
                            let idx = src_y * self.width + src_x;
                            let color = self.data[idx];
                            a_sum += color.w;
                            r_sum += color.x;
                            g_sum += color.y;
                            b_sum += color.z;
                            count += 1;
                        }
                    }
                }

                if count > 0 {
                    let a_avg = a_sum / count as f32;
                    let r_avg = r_sum / count as f32;
                    let g_avg = g_sum / count as f32;
                    let b_avg = b_sum / count as f32;

                    let avg_color = Vec4::new(r_avg, g_avg, b_avg, a_avg);
                    new_framebuffer.data[y * new_width + x] = avg_color;
                }
            }
        }
        new_framebuffer
    }

    fn float_to_u8(f: f32) -> u8 {
        (f.clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8
    }

    pub fn save_as_image(&self, filepath: &str) -> Result<(), image::ImageError> {
        use image::{ImageBuffer, Rgba};
        let mut img = ImageBuffer::new(self.width as u32, self.height as u32);

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let color = self.data[idx];

                let a = Self::float_to_u8(color.w);
                let r = Self::float_to_u8(color.x);
                let g = Self::float_to_u8(color.y);
                let b = Self::float_to_u8(color.z);

                img.put_pixel(x as u32, y as u32, Rgba([r, g, b, a]));
            }
        }

        img.save(filepath)
    }

    // 将深度缓冲可视化为图片（近→亮，远→暗）
    pub fn save_depth_as_image(&self, filepath: &str) -> Result<(), image::ImageError> {
        use image::{ImageBuffer, Rgba};
        let mut img = ImageBuffer::new(self.width as u32, self.height as u32);

        let near_plane = NEAR_PLANE; // 对应相机的near参数
        let far_plane = FAR_PLANE; // 对应相机的far参数

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let depth = self.depth[idx];

                // 排除无穷大的背景深度（单独处理）
                let normalized = if depth >= far_plane {
                    1.0 // 远处背景归一化为1.0（黑色）
                } else if depth <= near_plane {
                    0.0 // 近处物体归一化为0.0（白色）
                } else {
                    (depth - near_plane) / (far_plane - near_plane)
                };

                let color = (255.0 * (1.0 - normalized)) as u8; // 翻转颜色，近处亮，远处暗
                img.put_pixel(x as u32, y as u32, Rgba([color, color, color, 255]));
            }
        }

        img.save(filepath)
    }
}
