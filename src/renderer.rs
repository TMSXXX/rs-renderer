use crate::vertex::{ColoredVertex, RasterPoint, Triangle};
use crate::{camera, framebuffer, rasterizer};
use camera::Camera;
use cgmath::Matrix4 as Mat4;
use cgmath::{Vector2 as Vec2, Vector3 as Vec3, Vector4 as Vec4};
use framebuffer::FrameBuffer;

struct Viewport {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

pub struct Renderer {
    camera: Camera,
    pub(crate) framebuffer: FrameBuffer,
    viewport: Viewport,
}

impl Renderer {
    pub fn new(camera: Camera, w: usize, h: usize) -> Self {
        let framebuffer = FrameBuffer::new(w, h);
        Self {
            camera,
            framebuffer,
            viewport: Viewport {
                x: 0,
                y: 0,
                w: w as i32,
                h: h as i32,
            },
        }
    }
    // Unclipped lines, unsafe
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
                self.framebuffer.put_pixel(x0 as usize, y0 as usize, color, 0.0);
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

    pub fn draw_line_clipped(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: u32) -> bool {
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

    // 普通变换，无插值
    pub fn transform_and_project(
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

    // 带颜色插值的变换
    pub fn transform_colored_vertices(
        &self,
        vertices: &[ColoredVertex; 3],
        model: &Mat4<f32>,
    ) -> [RasterPoint; 3] {
        vertices.map(|v| {
            // 变换 3D 位置到裁剪空间
            let mut pos = v.pos.extend(1.0);
            pos = *self.camera.get_frustum().get_mat() * *model * pos;
            pos /= pos.w; // 透视除法

            // 转换到屏幕空间
            let screen_x = (pos.x + 1.0) * 0.5 * self.viewport.w as f32 + self.viewport.x as f32;
            let screen_y = self.viewport.h as f32 - (pos.y + 1.0) * 0.5 * self.viewport.h as f32
                + self.viewport.y as f32;

            RasterPoint {
                pos: Vec2::new(screen_x, screen_y),
                color: v.color, // 颜色保持不变，后续插值使用
                z: pos.z,       // 深度值（用于深度缓冲）
            }
        })
    }

    pub fn draw_rectangle(&mut self, vertices: &[Vec2<f32>; 3], color: u32) {
        for i in 0..vertices.len() {
            let p1 = &vertices[i];
            let p2 = &vertices[(i + 1) % vertices.len()];

            self.draw_line_clipped(p1.x, p1.y, p2.x, p2.y, color);
        }
    }

    pub fn rasterize_triangle(&mut self, vertices: &[Vec2<f32>; 3], color: u32) {
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

    pub fn rasterize_colored_triangle(&mut self, points: &[RasterPoint; 3]) {
        let (min_x, min_y, max_x, max_y) =
            rasterizer::get_box(&[points[0].pos, points[1].pos, points[2].pos]);
        let (min_x, min_y) = (min_x.max(0), min_y.max(0));
        let (max_x, max_y) = (
            max_x.min(self.framebuffer.width as i32 - 1),
            max_y.min(self.framebuffer.height as i32 - 1),
        );

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5); // 像素中心
                if rasterizer::is_inside_triangle(
                    &[points[0].pos, points[1].pos, points[2].pos],
                    &p,
                ) {
                    // 计算重心坐标
                    let bary = rasterizer::get_barycentric_coords(
                        &[points[0].pos, points[1].pos, points[2].pos],
                        &p,
                    );
                    // 插值颜色
                    let interpolated_color = rasterizer::interpolate_color(points, bary.unwrap());
                    let interpolated_depth = rasterizer::interpolate_depth(points, bary.unwrap());
                    // 转换为 u32 颜色格式（0~255 范围）
                    let color = ((interpolated_color.x * 255.0) as u32) << 16
                        | ((interpolated_color.y * 255.0) as u32) << 8
                        | ((interpolated_color.z * 255.0) as u32);
                    let color = 0xFF000000 | color; // 不透明 alpha 通道
                    // 绘制像素（如果后续有深度缓冲，这里需要加深度测试）
                    self.framebuffer.put_pixel(
                        x as usize,
                        y as usize,
                        color,
                        interpolated_depth,
                    );
                }
            }
        }
    }

    // 绘制多个带颜色插值的三角形
    pub fn render_colored_triangles(&mut self, triangles: &Vec<Triangle>, model: &Mat4<f32>) {
        for triangle in triangles {
            let raster_points = self.transform_colored_vertices(&triangle.vertices, model);
            self.rasterize_colored_triangle(&raster_points);
        }
    }
}
