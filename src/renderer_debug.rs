use crate::framebuffer::FrameBuffer;
use crate::renderer::Renderer;
use crate::rasterizer;
use cgmath::{Vector2 as Vec2, Vector3 as Vec3, Matrix4 as Mat4};

pub trait RendererDebugUtils {
    // 把遗留下来的简化函数和调试函数都放在这里
    fn draw_line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: u32);
    fn draw_line_clipped(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: u32) -> bool;
    fn draw_rectangle(&mut self, vertices: &[Vec2<f32>; 3], color: u32);
    fn rasterize_triangle(&mut self, vertices: &[Vec2<f32>; 3], color: u32);
    fn transform_and_project(
        &self,
        vertices: &[Vec3<f32>; 3],
        model: &Mat4<f32>,
    ) -> [Vec2<f32>; 3];
    fn draw_depth_outline(&mut self, line_width: usize, threshold: f32);
}

impl RendererDebugUtils for Renderer {
    //布雷森汉姆算法 画线段
    fn draw_line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: u32) {
        // Bresenham
        let mut x0 = x0 as i32;
        let mut y0 = y0 as i32;
        let x1 = x1 as i32;
        let y1 = y1 as i32;

        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        loop {
            if x0 >= 0 && y0 >= 0 {
                self.framebuffer
                    .put_pixel(x0 as usize, y0 as usize, color, 0.0);
            }

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x0 += sx;
            }
            if e2 < dx {
                err += dx;
                y0 += sy;
            }
        }
    }
    //科恩-萨瑟兰裁剪算法 先判断线段是否在区域内，再画线段
    fn draw_line_clipped(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: u32) -> bool {
        let mut x0: i32 = x0 as i32;
        let mut y0: i32 = y0 as i32;
        let mut x1: i32 = x1 as i32;
        let mut y1: i32 = y1 as i32;

        // 定义区域码
        const INSIDE: u8 = 0;
        const LEFT: u8 = 1;
        const RIGHT: u8 = 2;
        const BOTTOM: u8 = 4;
        const TOP: u8 = 8;

        let width_i = self.framebuffer.width as i32;
        let height_i = self.framebuffer.height as i32;

        // 计算点的区域码
        fn compute_code(x: i32, y: i32, width: i32, height: i32) -> u8 {
            let mut code = INSIDE;
            if x < 0 {
                code |= LEFT;
            } else if x >= width {
                code |= RIGHT;
            }
            if y < 0 {
                code |= BOTTOM;
            } else if y >= height {
                code |= TOP;
            }
            code
        }

        // 计算与边界的交点
        fn compute_intersection(
            x0: f32,
            y0: f32,
            x1: f32,
            y1: f32,
            boundary: u8,
            width: f32,
            height: f32,
        ) -> (f32, f32) {
            let t = match boundary {
                LEFT => (0.0 - x0) / (x1 - x0),
                RIGHT => ((width - 1.0) - x0) / (x1 - x0),
                BOTTOM => (0.0 - y0) / (y1 - y0),
                TOP => ((height - 1.0) - y0) / (y1 - y0),
                _ => 0.0,
            };

            let x = x0 + t * (x1 - x0);
            let y = y0 + t * (y1 - y0);

            (x, y)
        }

        let mut code0 = compute_code(x0, y0, width_i, height_i);
        let mut code1 = compute_code(x1, y1, width_i, height_i);
        let mut accept = false;

        loop {
            if code0 == INSIDE && code1 == INSIDE {
                accept = true;
                break;
            } else if (code0 & code1) != 0 {
                break;
            } else {
                let code_out = if code0 != INSIDE { code0 } else { code1 };
                let (x0_f, y0_f) = (x0 as f32, y0 as f32);
                let (x1_f, y1_f) = (x1 as f32, y1 as f32);
                let (width_f, height_f) = (width_i as f32, height_i as f32);

                let (x, y) = if (code_out & TOP) != 0 {
                    compute_intersection(x0_f, y0_f, x1_f, y1_f, TOP, width_f, height_f)
                } else if (code_out & BOTTOM) != 0 {
                    compute_intersection(x0_f, y0_f, x1_f, y1_f, BOTTOM, width_f, height_f)
                } else if (code_out & RIGHT) != 0 {
                    compute_intersection(x0_f, y0_f, x1_f, y1_f, RIGHT, width_f, height_f)
                } else {
                    compute_intersection(x0_f, y0_f, x1_f, y1_f, LEFT, width_f, height_f)
                };

                if code_out == code0 {
                    x0 = x.round() as i32;
                    y0 = y.round() as i32;
                    code0 = compute_code(x0, y0, width_i, height_i);
                } else {
                    x1 = x.round() as i32;
                    y1 = y.round() as i32;
                    code1 = compute_code(x1, y1, width_i, height_i);
                }
            }
        }

        if accept {
            let x0 = x0.max(0) as usize;
            let y0 = y0.max(0) as usize;
            let x1 = x1.max(0) as usize;
            let y1 = y1.max(0) as usize;

            if x0 < self.framebuffer.width
                && y0 < self.framebuffer.height
                && x1 < self.framebuffer.width
                && y1 < self.framebuffer.height
            {
                self.draw_line(x0, y0, x1, y1, color);
                return true;
            }
        }

        false
    }
    //调用draw_line_clipped画三角形（话说为什么是rectangle）
    fn draw_rectangle(&mut self, vertices: &[Vec2<f32>; 3], color: u32) {

        for i in 0..vertices.len() {
            let p1 = &vertices[i];
            let p2 = &vertices[(i + 1) % vertices.len()];

            self.draw_line_clipped(p1.x, p1.y, p2.x, p2.y, color);
        }
    }
    // 一个不涉及颜色插值和光线的简单光栅化函数
    fn rasterize_triangle(&mut self, vertices: &[Vec2<f32>; 3], color: u32) {
        let (min_x, min_y, max_x, max_y) = rasterizer::get_box(vertices);
        let (min_x, min_y) = (min_x.max(0), min_y.max(0));
        let (max_x, max_y) = (
            max_x.min(self.framebuffer.width as i32 - 1),
            max_y.min(self.framebuffer.height as i32 - 1),
        );

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                if rasterizer::is_inside_triangle(vertices, &p) {
                    self.framebuffer
                        .put_pixel(x as usize, y as usize, color, 0.0);
                }
            }
        }
    }

    // 普通变换，无插值
    // 一个不涉及深度和颜色插值的Vertex Shader
    fn transform_and_project(
        &self,
        vertices: &[Vec3<f32>; 3],
        model: &Mat4<f32>,
    ) -> [Vec2<f32>; 3] {
        let mut vertices = vertices.map(|v| v.extend(1.0));
        for v in &mut vertices {
            *v = *self.camera.get_frustum().get_mat() * *model * *v;
            *v /= v.w;
        }

        let vertices = vertices.map(|v| {
            Vec2::new(
                (v.x + 1.0) * 0.5 * self.viewport.w as f32 - 1.0 + self.viewport.x as f32,
                self.viewport.h as f32 - (v.y + 1.0) * 0.5 * self.viewport.h as f32 - 1.0
                    + self.viewport.y as f32,
            )
        });
        vertices
    }

    //基本的无卷积核描边
    fn draw_depth_outline(&mut self, line_width: usize, threshold: f32) {
        let width = self.framebuffer.width;
        let height = self.framebuffer.height;

        // 复制一份深度缓冲，避免在遍历时修改
        let depth_buffer = self.framebuffer.depth.clone();

        // 遍历所有像素，注意边界
        for y in line_width..height - line_width {
            for x in line_width..width - line_width {
                let idx = y * width + x;
                let current_depth = depth_buffer[idx];

                // 检查当前像素的深度是否有效（非背景）
                if current_depth >= f32::MAX {
                    continue;
                }

                let mut is_outline = false;

                // 扩大采样范围，检查周围line_width个像素
                for i in 1..=line_width {
                    // 检查右侧和左侧
                    let right_idx = idx + i;
                    let left_idx = idx - i;
                    let diff_right = (current_depth - depth_buffer[right_idx]).abs();
                    let diff_left = (current_depth - depth_buffer[left_idx]).abs();

                    // 检查下方和上方
                    let bottom_idx = (y + i) * width + x;
                    let top_idx = (y - i) * width + x;
                    let diff_bottom = (current_depth - depth_buffer[bottom_idx]).abs();
                    let diff_top = (current_depth - depth_buffer[top_idx]).abs();

                    if diff_right > threshold
                        || diff_left > threshold
                        || diff_bottom > threshold
                        || diff_top > threshold
                    {
                        is_outline = true;
                        break; // 只要检测到是描边，就退出内层循环
                    }
                }

                if is_outline {
                    self.framebuffer.data[idx] = 0xFF000000; // 黑色
                }
            }
        }
    }

}