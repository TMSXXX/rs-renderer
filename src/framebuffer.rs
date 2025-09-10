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
            depth: vec![f32::INFINITY; width * height],
        }
    }

    pub fn clear(&mut self, color: u32) {
        self.data.fill(color);
        self.depth.fill(f32::INFINITY);
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
        if factor <= 1 {
            return self.clone();
        }
        let mut new_self = FrameBuffer::new(self.width / factor, self.height / factor);
        for y in 0..new_self.height {
            for x in 0..new_self.width {
                let mut r_sum = 0u32;
                let mut g_sum = 0u32;
                let mut b_sum = 0u32;
                let mut a_sum = 0u32;
                for dy in 0..factor {
                    for dx in 0..factor {
                        let color = self.data[(y * factor + dy) * self.width + (x * factor + dx)];
                        a_sum += (color >> 24) & 0xFF;
                        r_sum += (color >> 16) & 0xFF;
                        g_sum += (color >> 8) & 0xFF;
                        b_sum += color & 0xFF;
                    }
                }
                let total_samples = (factor * factor) as u32;
                let a_avg = (a_sum / total_samples) as u32;
                let r_avg = (r_sum / total_samples) as u32;
                let g_avg = (g_sum / total_samples) as u32;
                let b_avg = (b_sum / total_samples) as u32;
                let avg_color = (a_avg << 24) | (r_avg << 16) | (g_avg << 8) | b_avg;
                let avg_depth = self.depth[(y * factor) * self.width + (x * factor)];
                new_self.put_pixel(x, y, avg_color, avg_depth);
            }
        }
        new_self
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
}
