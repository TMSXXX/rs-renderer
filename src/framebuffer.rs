use cgmath::Vector3 as Vec3;

use crate::{BLUE, FAR_PLANE, NEAR_PLANE};

#[derive(Clone)]
pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u32>,
    pub depth: Vec<f32>,
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        FrameBuffer {
            width,
            height,
            data: vec![0; width * height],
            depth: vec![1.0; width * height],
        }
    }

    pub fn clear(&mut self, color: u32) {
        self.data.fill(color);
        self.depth.fill(1.0);
    }

    pub fn put_pixel(&mut self, x: usize, y: usize, color: u32, depth: f32) {
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
        let new_width = self.width / factor;
        let new_height = self.height / factor;
        let mut new_framebuffer = FrameBuffer::new(new_width, new_height);

        for y in 0..new_height {
            for x in 0..new_width {
                let mut r_sum = 0u32;
                let mut g_sum = 0u32;
                let mut b_sum = 0u32;
                let mut a_sum = 0u32;
                let mut count = 0;

                for fy in 0..factor {
                    for fx in 0..factor {
                        let src_x = x * factor + fx;
                        let src_y = y * factor + fy;
                        if src_x < self.width && src_y < self.height {
                            let idx = src_y * self.width + src_x;
                            let color = self.data[idx];
                            a_sum += (color >> 24) & 0xFF;
                            r_sum += (color >> 16) & 0xFF;
                            g_sum += (color >> 8) & 0xFF;
                            b_sum += color & 0xFF;
                            count += 1;
                        }
                    }
                }

                if count > 0 {
                    let a_avg = (a_sum / count) as u32;
                    let r_avg = (r_sum / count) as u32;
                    let g_avg = (g_sum / count) as u32;
                    let b_avg = (b_sum / count) as u32;

                    let avg_color = (a_avg << 24) | (r_avg << 16) | (g_avg << 8) | b_avg;
                    new_framebuffer.data[y * new_width + x] = avg_color;
                }
            }
        }
        new_framebuffer
    }


    pub fn save_as_image(&self, filepath: &str) -> Result<(), image::ImageError> {
        use image::{ImageBuffer, Rgba};
        let mut img = ImageBuffer::new(self.width as u32, self.height as u32);

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let color = self.data[idx];

                let a = ((color >> 24) & 0xFF) as u8;
                let r = ((color >> 16) & 0xFF) as u8;
                let g = ((color >> 8) & 0xFF) as u8;
                let b = (color & 0xFF) as u8;

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
