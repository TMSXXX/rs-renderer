#[derive(Clone)]
pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u32>,
    pub depth: Vec<f32>,
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let far_plane = 5.0;
        FrameBuffer {
            width,
            height,
            data: vec![0; width * height],
            depth: vec![far_plane; width * height],
        }
    }

    pub fn clear(&mut self, color: u32) {
        self.data.fill(color);
        self.depth.fill(5.0);
    }

    pub fn put_pixel(&mut self, x: usize, y: usize, color: u32, depth: f32) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            if depth < self.depth[idx] {
                self.data[idx] = color;
                self.depth[idx] = depth;
            }
        }
    }

    pub fn ssaa(&self, factor: usize) -> Self {
        let new_width = self.width / factor;
        let new_height = self.height / factor;
        let mut new_data = vec![0; new_width * new_height];

        // 遍历缩小后的每个像素
        for y in 0..new_height {
            for x in 0..new_width {
                // 计算高分辨率中对应区域的像素
                let mut r = 0u32;
                let mut g = 0u32;
                let mut b = 0u32;
                let mut a = 0u32;
                let count = (factor * factor) as u32;

                // 采样高分辨率区域内的所有像素
                for dy in 0..factor {
                    for dx in 0..factor {
                        let src_x = x * factor + dx;
                        let src_y = y * factor + dy;
                        let src_idx = src_y * self.width + src_x;
                        let color = self.data[src_idx];

                        // 提取RGBA分量
                        a += (color >> 24) & 0xFF;
                        r += (color >> 16) & 0xFF;
                        g += (color >> 8) & 0xFF;
                        b += color & 0xFF;
                    }
                }

                // 计算平均值（关键：包括背景色像素）
                let avg_a = (a / count) as u8;
                let avg_r = (r / count) as u8;
                let avg_g = (g / count) as u8;
                let avg_b = (b / count) as u8;

                // 合并为新颜色
                let new_color = (avg_a as u32) << 24
                    | (avg_r as u32) << 16
                    | (avg_g as u32) << 8
                    | avg_b as u32;
                new_data[y * new_width + x] = new_color;
            }
        }

        Self {
            width: new_width,
            height: new_height,
            data: new_data,
            depth: vec![0.0; new_width * new_height], // 深度缓冲可简化处理
        }
    }

    pub fn save_to_image(&self, filepath: &str) -> Result<(), image::ImageError> {
        use image::{ImageBuffer, Rgba};

        let mut img = ImageBuffer::new(self.width as u32, self.height as u32);

        for y in 0..self.height {
            for x in 0..self.width {
                let color = self.data[y * self.width + x];
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

        // 方法1：使用相机的近远平面范围（更准确）
        let near_plane = 1.0; // 对应相机的near参数
        let far_plane = 5.0; // 对应相机的far参数

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
                    // 线性映射到[0,1]：近→0（白），远→1（黑）
                    (depth - near_plane) / (far_plane - near_plane)
                };

                // 反转映射：让近处更亮，远处更暗
                let brightness = (1.0 - normalized) * 255.0;
                let val = brightness as u8;

                img.put_pixel(x as u32, y as u32, Rgba([val, val, val, 255]));
            }
        }
        img.save(filepath)
    }
}
