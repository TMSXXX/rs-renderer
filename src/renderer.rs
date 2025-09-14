pub mod clip;
pub mod fragment_shader;
pub mod vertex_shader;

use crate::BLACK;
use crate::renderer::fragment_shader::InkShader;
use crate::texture::Texture;
use crate::vertex::{ClipSpaceVertex, Material, RasterPoint, RasterTriangle, Triangle};
use crate::{camera, framebuffer, rasterizer};
use camera::Camera;
use cgmath::{InnerSpace, Matrix, Matrix4 as Mat4, SquareMatrix};
use cgmath::{Vector2 as Vec2, Vector3 as Vec3};
use fragment_shader::{FragmentData, FragmentShader, NormalDebugShader, PhongShader, ToonShader};
use framebuffer::FrameBuffer;
use parking_lot::Mutex;
use rayon::prelude::*;
use std::sync::Arc;

use self::clip::{Clipper, SimpleClipper};
use self::vertex_shader::{DefaultVertexShader, VertexShader, VertexShaderUniforms};

//use crate::renderer_debug::RendererDebugUtils; // 已经被迁移出去的旧函数

pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Clone, Copy)]
pub struct Light {
    pub direction: Vec3<f32>,
    pub color: Vec3<f32>,
    pub intensity: f32,
    pub ambient_strength: f32,
    pub ambient_color: Vec3<f32>,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            direction: Vec3::new(1., -0.2, -0.1).normalize(),
            color: Vec3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
            ambient_strength: 0.5,                   // 默认环境光强度
            ambient_color: Vec3::new(1.0, 1.0, 1.0), // 白色环境光
        }
    }
}

impl Light {
    pub fn set_light(&mut self, color: [f32; 3], direction: [f32; 3]) {
        self.color = Vec3::new(color[0], color[1], color[2]);
        self.direction = Vec3::new(direction[0], direction[1], direction[2]).normalize();
    }
}

pub struct Renderer {
    pub(crate) camera: Camera,
    pub(crate) framebuffer: Arc<Mutex<FrameBuffer>>,
    pub(crate) viewport: Viewport,
    pub(crate) light: Light,
}

impl Renderer {
    pub fn new(camera: Camera, w: usize, h: usize) -> Self {
        let framebuffer = Arc::new(Mutex::new(FrameBuffer::new(w, h)));
        Self {
            camera,
            framebuffer,
            viewport: Viewport {
                x: 0,
                y: 0,
                w: w as i32,
                h: h as i32,
            },
            light: Light::default(),
        }
    }
    //一统江山后的完整渲染管线
    pub fn render_colored_triangles(
        &mut self,
        triangles: &mut Vec<Triangle>,
        model: &Mat4<f32>,
        texture: Option<&Texture>,
        shader_name: &str,
    ) {
        println!("三角形数量: {}", triangles.len());
        //统一运算矩阵
        let normal_matrix = model.invert().unwrap().transpose();
        let view_matrix = self.camera.get_view_mat();
        let proj_matrix = self.camera.get_frustum().get_mat();
        let mvp_matrix = proj_matrix * view_matrix * model;

        // 初始化本次渲染所使用的模块
        let vertex_shader = DefaultVertexShader;
        let clipper = SimpleClipper;
        let fragment_shader: Box<dyn FragmentShader> = match shader_name {
            "toon" => Box::new(ToonShader { light: self.light }),
            "ink" => Box::new(InkShader { light: self.light }),
            "phong" => Box::new(PhongShader { light: self.light }),
            "normal" => Box::new(NormalDebugShader),
            _ => Box::new(ToonShader { light: self.light }),
        };

        let uniforms = VertexShaderUniforms {
            model_matrix: model,
            mvp_matrix: &mvp_matrix,
            normal_matrix: &normal_matrix,
        };

        triangles.par_iter().for_each(|triangle| {
            //管线阶段 1: 背面剔除
            let world_pos =
                (uniforms.model_matrix * triangle.vertices[0].pos.extend(1.0)).truncate();
            let view_dir = (self.camera.eye - world_pos).normalize();
            let tri_normal = (uniforms.normal_matrix * triangle.normal.extend(0.0)).truncate();
            if view_dir.dot(tri_normal) <= 0.0 {
                return; // 剔除该三角形
            }

            //管线阶段 2: 顶点着色
            let clip_space_triangle = vertex_shader.shade_triangle(triangle, &uniforms);

            //管线阶段 3: 裁剪
            let clipped_triangles = clipper.clip_triangle(&clip_space_triangle);

            for clipped_triangle_verts in clipped_triangles {
                // 阶段 4: 屏幕映射
                let raster_triangle =
                    self.viewport_transform(&clipped_triangle_verts, triangle.material);

                let mut fb = self.framebuffer.lock();
                // 阶段 5: 光栅化和像素着色
                Self::rasterize_triangle(
                    &mut fb,
                    &raster_triangle,
                    texture,
                    fragment_shader.as_ref(),
                    self.camera.eye,
                );
            }
        });
    }

    //视口变换
    fn viewport_transform(
        &self,
        clip_triangle: &[ClipSpaceVertex; 3], // 接收一个数组的引用
        material: Material,
    ) -> RasterTriangle {
        let raster_vertices = clip_triangle.map(|clip_v| {
            let mut ndc_pos = clip_v.position;

            // 1. 透视除法
            ndc_pos /= clip_v.position.w;

            // 转换到屏幕空间
            let screen_x =
                (ndc_pos.x + 1.0) * 0.5 * self.viewport.w as f32 + self.viewport.x as f32;
            let screen_y = self.viewport.h as f32
                - (ndc_pos.y + 1.0) * 0.5 * self.viewport.h as f32
                + self.viewport.y as f32;

            RasterPoint {
                pos: Vec2::new(screen_x, screen_y),
                z: (ndc_pos.z + 1.0) * 0.5,
                // 继承其他属性
                world_pos: clip_v.world_pos,
                normal: clip_v.normal,
                uv: clip_v.uv,
                color: clip_v.color,
            }
        });

        RasterTriangle {
            vertices: raster_vertices,
            material,
        }
    }

    // 进行光栅化
    pub fn rasterize_triangle(
        framebuffer: &mut FrameBuffer,
        triangle: &RasterTriangle,
        texture: Option<&Texture>,
        shader: &dyn FragmentShader, // 接收一个Shader
        camera_pos: Vec3<f32>,
    ) {
        let points = &triangle.vertices;
        let (min_x, min_y, max_x, max_y) =
            rasterizer::get_box(&[points[0].pos, points[1].pos, points[2].pos]);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                if rasterizer::is_inside_triangle(
                    &[points[0].pos, points[1].pos, points[2].pos],
                    &p,
                ) {
                    let bary = rasterizer::get_barycentric_coords(
                        &[points[0].pos, points[1].pos, points[2].pos],
                        &p,
                    )
                    .unwrap_or((0.0, 0.0, 0.0));

                    // 插值所有属性
                    let interpolated = {
                        let z = rasterizer::interpolate_depth(points, bary);
                        let normal = rasterizer::interpolate_normal(points, bary);
                        let uv = rasterizer::interpolate_uv(points, bary);
                        let color = rasterizer::interpolate_color(points, bary);
                        let world_pos = points[0].world_pos * bary.2
                            + points[1].world_pos * bary.1
                            + points[2].world_pos * bary.0;
                        (z, normal, uv, color, world_pos)
                    };

                    // 打包成 FragmentData
                    let fragment_data = FragmentData {
                        world_pos: interpolated.4,
                        normal: interpolated.1,
                        uv: interpolated.2,
                        color: interpolated.3,
                        texture,
                        material: &triangle.material,
                        camera_pos,
                    };

                    // 调用 shader 来获取颜色！
                    let color = shader.shade(fragment_data);
                    framebuffer.put_pixel(
                        x as usize,
                        y as usize,
                        color.extend(1.0),
                        interpolated.0,
                    );
                }
            }
        }
    }

    pub fn draw_depth_outline_sobel(&mut self, threshold: f32, line_width: usize) {
        let (width, height, depth_buffer) = {
            let fb = self.framebuffer.lock(); // 获取锁，生成守卫 fb
            (fb.width, fb.height, fb.depth.clone()) // 克隆深度缓冲（关键：避免持有原锁）
            // 代码块结束，fb 离开作用域，锁自动释放
        };

        // 并行计算边缘像素
        let outline_pixels: Vec<(usize, usize)> = (line_width..height - line_width)
            .into_par_iter()
            .flat_map(|y| {
                let mut row_pixels = Vec::new();
                for x in line_width..width - line_width {
                    let idx = y * width + x;
                    let current_depth = depth_buffer[idx];

                    // 跳过背景像素（深度值无效）
                    if current_depth >= f32::MAX {
                        continue;
                    }

                    let mut is_outline = false;

                    // 检查周围 line_width 范围内的深度差异
                    for i in 1..=line_width {
                        // 左右像素检查
                        let right_idx = idx + i;
                        let left_idx = idx - i;
                        let diff_right = (current_depth - depth_buffer[right_idx]).abs();
                        let diff_left = (current_depth - depth_buffer[left_idx]).abs();

                        // 上下像素检查
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
                            break; // 一旦检测到边缘，退出当前循环
                        }
                    }

                    if is_outline {
                        row_pixels.push((x, y));
                    }
                }
                row_pixels
            })
            .collect();
        // 绘制描边（优化锁操作和循环）
        if !outline_pixels.is_empty() {
            let mut fb = self.framebuffer.lock();
            let width = fb.width;
            let height = fb.height;

            for &(x, y) in &outline_pixels {
                let max_x = (x + line_width).min(width);
                let max_y = (y + line_width).min(height);
                // 扩展线宽并绘制
                for draw_y in y..max_y {
                    for draw_x in x..max_x {
                        let index = draw_y * width + draw_x;
                        fb.data[index] = BLACK; // 黑色描边
                    }
                }
            }
        }
    }

    pub fn draw_color_outline_sobel(&mut self, threshold: f32, line_width: usize) {
        // Sobel
        let sobel_x = [[-1.0, 0.0, 1.0], [-2.0, 0.0, 2.0], [-1.0, 0.0, 1.0]]; // x方向Sobel核
        let sobel_y = [[-1.0, -2.0, -1.0], [0.0, 0.0, 0.0], [1.0, 2.0, 1.0]]; // y方向Sobel核

        let (width, height, color_buffer) = {
            let fb = self.framebuffer.lock(); // 获取锁，生成守卫 fb
            (fb.width, fb.height, fb.data.clone()) // 克隆深度缓冲（关键：避免持有原锁）
            // 代码块结束，fb 离开作用域，锁自动释放
        };

        // 解析 u32 颜色为 RGB 通道（0.0~1.0 范围），存储为二维矩阵
        let mut color_matrix = vec![vec![[0.0; 3]; width]; height]; // [y][x] -> [r, g, b]
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let argb = color_buffer[idx]; // 假设为 ARGB8888 格式

                // 提取 RGB 通道（8位转 0.0~1.0）
                let r = argb.x;
                let g = argb.y;
                let b = argb.z;

                color_matrix[y][x] = [r, g, b];
            }
        }

        // 遍历计算颜色梯度（避开边界像素）
        let outline_pixels: Vec<(usize, usize)> = (1..height - 1)
            .into_par_iter()
            .flat_map(|y| {
                let mut row_pixels = Vec::new();
                for x in 1..width - 1 {
                    // 计算 RGB 三个通道的梯度（保留原有逻辑）
                    let (mut gx_r, mut gy_r) = (0.0, 0.0);
                    let (mut gx_g, mut gy_g) = (0.0, 0.0);
                    let (mut gx_b, mut gy_b) = (0.0, 0.0);

                    for ky in 0..3 {
                        for kx in 0..3 {
                            let ny = (y as i32 + (ky as i32 - 1)) as usize;
                            let nx = (x as i32 + (kx as i32 - 1)) as usize;

                            gx_r += sobel_x[ky][kx] * color_matrix[ny][nx][0];
                            gy_r += sobel_y[ky][kx] * color_matrix[ny][nx][0];

                            gx_g += sobel_x[ky][kx] * color_matrix[ny][nx][1];
                            gy_g += sobel_y[ky][kx] * color_matrix[ny][nx][1];

                            gx_b += sobel_x[ky][kx] * color_matrix[ny][nx][2];
                            gy_b += sobel_y[ky][kx] * color_matrix[ny][nx][2];
                        }
                    }

                    // 计算梯度幅值（保留原有逻辑）
                    let mag_r = gx_r.abs() + gy_r.abs();
                    let mag_g = gx_g.abs() + gy_g.abs();
                    let mag_b = gx_b.abs() + gy_b.abs();
                    let gradient_mag = mag_r.max(mag_g).max(mag_b);

                    if gradient_mag > threshold {
                        row_pixels.push((x, y));
                    }
                }
                row_pixels
            })
            .collect();

        // 绘制描边（直接操作 u32 颜色）
        if !outline_pixels.is_empty() {
            let mut fb = self.framebuffer.lock();
            let width = fb.width;
            let height = fb.height;

            for &(x, y) in &outline_pixels {
                let max_x = (x + line_width).min(width);
                let max_y = (y + line_width).min(height);

                // 扩展线宽并绘制
                for draw_y in y..max_y {
                    for draw_x in x..max_x {
                        let index = draw_y * width + draw_x;
                        fb.data[index] = BLACK; // 黑色描边
                    }
                }
            }
        }
    }
}
