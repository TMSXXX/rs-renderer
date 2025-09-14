pub mod fragment_shader;
pub mod vertex_shader;
pub mod clip;

use crate::BLACK;
use crate::renderer::fragment_shader::InkShader;
use crate::texture::Texture;
use crate::vertex::{ClipSpaceVertex, ColoredVertex, Material, RasterPoint, RasterTriangle, Triangle};
use crate::{camera, framebuffer, rasterizer};
use camera::Camera;
use cgmath::{ElementWise, InnerSpace, Matrix, Matrix3, Matrix4 as Mat4, SquareMatrix, Zero, dot};
use cgmath::{Matrix3 as Mat3, Vector2 as Vec2, Vector3 as Vec3, Vector4 as Vec4, prelude::*};
use fragment_shader::{FragmentData, FragmentShader, NormalDebugShader, PhongShader, ToonShader};
use framebuffer::FrameBuffer;

use self::vertex_shader::{DefaultVertexShader, VertexShader, VertexShaderUniforms};
use self::clip::{Clipper, SimpleClipper};

use crate::renderer_debug::RendererDebugUtils; // 已经被迁移出去的旧函数 

pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Clone, Copy)] // <--- 添加这一行
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
    pub(crate) framebuffer: FrameBuffer,
    pub(crate) viewport: Viewport,
    pub(crate) light: Light,
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

        let mut i = 0;
        let count = triangles.len() == 200;
        for triangle in triangles {
            if count {
                println!("{i}");
                i += 1;
            }
            //管线阶段 1: 背面剔除
            let world_pos = (uniforms.model_matrix * triangle.vertices[0].pos.extend(1.0)).truncate();
            let view_dir = (self.camera.eye - world_pos).normalize();
            let tri_normal = (uniforms.normal_matrix * triangle.normal.extend(0.0)).truncate();
            if view_dir.dot(tri_normal) <= 0.0 {
                continue;
            }

            //管线阶段 2: 顶点着色
            let clip_space_triangle = vertex_shader.shade_triangle(triangle, &uniforms);

            //管线阶段 3: 裁剪
            let clipped_triangles = clipper.clip_triangle(&clip_space_triangle);


            for clipped_triangle_verts in clipped_triangles {
                // 阶段 4: 屏幕映射
                let raster_triangle = self.viewport_transform(&clipped_triangle_verts, triangle.material);
                
                // 阶段 5: 光栅化和像素着色
                self.rasterize_triangle(&raster_triangle, texture, &*fragment_shader);

            }
        }
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
            let screen_x = (ndc_pos.x + 1.0) * 0.5 * self.viewport.w as f32 + self.viewport.x as f32;
            let screen_y = self.viewport.h as f32 - (ndc_pos.y + 1.0) * 0.5 * self.viewport.h as f32
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

        RasterTriangle { vertices: raster_vertices, material }
    }


    // 进行光栅化
    pub fn rasterize_triangle(
        &mut self,
        triangle: &RasterTriangle,
        texture: Option<&Texture>,
        shader: &dyn FragmentShader, // 接收一个Shader
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
                    let interpolated_color = rasterizer::interpolate_color(points, bary);
                    let interpolated_depth = rasterizer::interpolate_depth(points, bary);
                    let interpolated_normal = rasterizer::interpolate_normal(points, bary);
                    let interpolated_uv = rasterizer::interpolate_uv(points, bary);
                    let interpolated_world_pos = {
                        let p0 = points[0].world_pos * bary.2;
                        let p1 = points[1].world_pos * bary.1;
                        let p2 = points[2].world_pos * bary.0;
                        p0 + p1 + p2
                    };

                    // 打包成 FragmentData
                    let fragment_data = FragmentData {
                        world_pos: interpolated_world_pos,
                        normal: interpolated_normal,
                        uv: interpolated_uv,
                        color: interpolated_color,
                        texture,
                        material: &triangle.material,
                        camera_pos: self.camera.eye,
                    };

                    // 调用 shader 来获取颜色！
                    let final_color_vec = shader.shade(fragment_data);

                    // 转换为 u32 颜色格式
                    let color = ((final_color_vec.x * 255.0) as u32) << 16
                        | ((final_color_vec.y * 255.0) as u32) << 8
                        | ((final_color_vec.z * 255.0) as u32);
                    let color = 0xFF000000 | color;

                    self.framebuffer
                        .put_pixel(x as usize, y as usize, color, interpolated_depth);
                }
            }
        }
    }

    pub fn draw_depth_outline_sobel(&mut self, threshold: f32, line_width: usize) {
        // 定义Prewitt算子的水平和垂直卷积核（3x3二维数组）
        let sobel_x = [[-1.0, 0.0, 1.0], [-2.0, 0.0, 2.0], [-1.0, 0.0, 1.0]];  // x方向Sobel核
        let sobel_y = [[-1.0, -2.0, -1.0], [0.0, 0.0, 0.0], [1.0, 2.0, 1.0]];  // y方向Sobel核

        let width = self.framebuffer.width;
        let height = self.framebuffer.height;
        let depth_buffer = self.framebuffer.depth.clone();

        // 将一维深度缓冲转换为二维数组（y行x列）
        let mut depth_matrix = vec![vec![0.0; width]; height];
        for y in 0..height {
            for x in 0..width {
                depth_matrix[y][x] = depth_buffer[y * width + x];
            }
        }

        let mut outline_pixels = Vec::new();

        // 遍历深度矩阵计算梯度（避开边界像素）
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let current_depth = depth_matrix[y][x];
                const BACKGROUND_DEPTH: f32 = 1.0; // 关键：替换为你渲染管线中的背景深度值
                if (current_depth - BACKGROUND_DEPTH).abs() < 1e-6 {
                    // 允许微小误差（浮点精度问题）
                    continue; // 背景像素不参与边缘检测
                }
                // 初始化梯度值
                let mut gx = 0.0;
                let mut gy = 0.0;

                // 3x3邻域卷积运算
                for ky in 0..3 {
                    for kx in 0..3 {
                        // 计算邻域像素坐标（相对于当前像素的偏移）
                        let ny = (y as i32 + (ky as i32 - 1)) as usize;
                        let nx = (x as i32 + (kx as i32 - 1)) as usize;

                        // 累加梯度值：卷积核权重 × 深度值
                        gx += sobel_x[ky][kx] * depth_matrix[ny][nx];
                        gy += sobel_y[ky][kx] * depth_matrix[ny][nx];
                    }
                }

                // 计算梯度幅值（使用简化版 |Gx| + |Gy|）
                let gradient_mag = gx.abs() + gy.abs();

                // 判断是否为边缘像素
                if gradient_mag > threshold {
                    outline_pixels.push((x, y));
                }
            }
        }

        // 绘制轮廓线
        for &(x, y) in &outline_pixels {
            // 绘制指定线宽的轮廓
            for dy in 0..line_width {
                for dx in 0..line_width {
                    let draw_x = x + dx;
                    let draw_y = y + dy;
                    if draw_x < width && draw_y < height {
                        let index = draw_y * width + draw_x;
                        // 设置轮廓颜色为黑色（RGBA）
                        self.framebuffer.data[index] = BLACK;
                    }
                }
            }
        }
    }

    pub fn draw_color_outline_sobel(&mut self, threshold: f32, line_width: usize) {
        // Sobel
    let sobel_x = [[-1.0, 0.0, 1.0], [-2.0, 0.0, 2.0], [-1.0, 0.0, 1.0]];  // x方向Sobel核
    let sobel_y = [[-1.0, -2.0, -1.0], [0.0, 0.0, 0.0], [1.0, 2.0, 1.0]];  // y方向Sobel核

        let width = self.framebuffer.width;
        let height = self.framebuffer.height;
        let color_buffer = self.framebuffer.data.clone(); // 此时 data 是 Vec<u32>

        // 解析 u32 颜色为 RGB 通道（0.0~1.0 范围），存储为二维矩阵
        let mut color_matrix = vec![vec![[0.0; 3]; width]; height]; // [y][x] -> [r, g, b]
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let argb = color_buffer[idx]; // 假设为 ARGB8888 格式

                // 提取 RGB 通道（8位转 0.0~1.0）
                let r = ((argb >> 16) & 0xFF) as f32 / 255.0;
                let g = ((argb >> 8) & 0xFF) as f32 / 255.0;
                let b = (argb & 0xFF) as f32 / 255.0;

                color_matrix[y][x] = [r, g, b];
            }
        }

        let mut outline_pixels = Vec::new();

        // 遍历计算颜色梯度（避开边界像素）
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                // 计算 RGB 三个通道的梯度
                let (mut gx_r, mut gy_r) = (0.0, 0.0);
                let (mut gx_g, mut gy_g) = (0.0, 0.0);
                let (mut gx_b, mut gy_b) = (0.0, 0.0);

                // 3x3 邻域卷积
                for ky in 0..3 {
                    for kx in 0..3 {
                        let ny = (y as i32 + (ky as i32 - 1)) as usize;
                        let nx = (x as i32 + (kx as i32 - 1)) as usize;

                        // 对每个颜色通道应用卷积核
                        gx_r += sobel_x[ky][kx] * color_matrix[ny][nx][0];
                        gy_r += sobel_y[ky][kx] * color_matrix[ny][nx][0];

                        gx_g += sobel_x[ky][kx] * color_matrix[ny][nx][1];
                        gy_g += sobel_y[ky][kx] * color_matrix[ny][nx][1];

                        gx_b += sobel_x[ky][kx] * color_matrix[ny][nx][2];
                        gy_b += sobel_y[ky][kx] * color_matrix[ny][nx][2];
                    }
                }

                // 计算梯度幅值（取三通道最大值）
                let mag_r = gx_r.abs() + gy_r.abs();
                let mag_g = gx_g.abs() + gy_g.abs();
                let mag_b = gx_b.abs() + gy_b.abs();
                let gradient_mag = mag_r.max(mag_g).max(mag_b);

                // 超过阈值则视为边缘
                if gradient_mag > threshold {
                    outline_pixels.push((x, y));
                }
            }
        }

        // 绘制描边（直接操作 u32 颜色）
        for &(x, y) in &outline_pixels {
            for dy in 0..line_width {
                for dx in 0..line_width {
                    let draw_x = x + dx;
                    let draw_y = y + dy;
                    if draw_x < width && draw_y < height {
                        let index = draw_y * width + draw_x;
                        self.framebuffer.data[index] = BLACK; // 直接设置 u32 颜色
                    }
                }
            }
        }
    }
}
