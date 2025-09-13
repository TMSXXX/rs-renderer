pub mod fragment_shader;

use crate::BLACK;
use crate::texture::Texture;
use crate::vertex::{ColoredVertex, Material, RasterPoint, RasterTriangle, Triangle};
use crate::{camera, framebuffer, rasterizer};
use camera::Camera;
use cgmath::{ElementWise, InnerSpace, Matrix, Matrix3, Matrix4 as Mat4, SquareMatrix, Zero, dot};
use cgmath::{Matrix3 as Mat3, Vector2 as Vec2, Vector3 as Vec3, Vector4 as Vec4, prelude::*};
use framebuffer::FrameBuffer;
use fragment_shader::{FragmentShader, FragmentData,ToonShader, PhongShader, NormalDebugShader};


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

pub struct Renderer {
    pub(crate) camera: Camera,
    pub(crate) framebuffer: FrameBuffer,
    pub(crate) viewport: Viewport,
    light: Light,
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


    // 提取模型三角形并逐一送入渲染管线内

    pub fn render_colored_triangles(
        &mut self,
        triangles: &mut Vec<Triangle>,
        model: &Mat4<f32>,
        texture: Option<&Texture>,
    ) {
        println!("三角形数量: {}", triangles.len());
        let normal_matrix = model.invert().unwrap().transpose();

        let shader = ToonShader { light: self.light };
        //let shader = PhongShader { light: self.light };
        //let shader = NormalDebugShader;

        let mut i = 0;
        for triangle in triangles {
            //println!("{i}");
            //i += 1;
            let world_pos = (*model * triangle.vertices[0].pos.extend(1.0)).truncate();
            let view_dir = (self.camera.eye - world_pos).normalize();
            let tri_normal = (normal_matrix * triangle.normal.extend(0.0)).truncate();
            // 提前剔除背面
            if view_dir.dot(tri_normal) <= 0.0 {
                continue;
            }
            let raster_triangle = self.transform_colored_vertices(triangle, model);
            self.rasterize_triangle(&raster_triangle, texture, &shader);
        }
    }
    

    // 带颜色插值的变换
    // 其实就是一个完整的顶点着色器 Vertex Shader
    pub fn transform_colored_vertices( 
        &self,
        triangle: &Triangle,
        model: &Mat4<f32>,
    ) -> RasterTriangle {
        let vertices = triangle.vertices;
        let normal_matrix = model.invert().unwrap().transpose();
        let view_matrix = self.camera.get_view_mat();

        let raster_vertices = vertices.map(|v| {
            let world_pos = (*model * v.pos.extend(1.0)).truncate();
            // 变换 3D 位置到裁剪空间
            let mut pos = v.pos.extend(1.0);
            pos = *self.camera.get_frustum().get_mat() * view_matrix * *model * pos;

            pos /= pos.w;

            let depth = (pos.z + 1.0) * 0.5;

            // 变换法线（使用法线矩阵）
            let mut normal = v.normal.extend(0.0);
            normal = normal_matrix * normal;
            let normal = Vec3::new(normal.x, normal.y, normal.z).normalize();

            // 转换到屏幕空间
            let screen_x = (pos.x + 1.0) * 0.5 * self.viewport.w as f32 + self.viewport.x as f32;
            let screen_y = self.viewport.h as f32 - (pos.y + 1.0) * 0.5 * self.viewport.h as f32
                + self.viewport.y as f32;

            RasterPoint {
                pos: Vec2::new(screen_x, screen_y),
                color: v.color, // 颜色保持不变，后续插值使用
                z: depth,       // 深度值（用于深度缓冲）
                normal: normal, // 法线保持不变，后续光照计算使用
                uv: v.uv,
                world_pos,
            }
        });
        //print!(" 转换");
        RasterTriangle {
            vertices: raster_vertices,
            material: triangle.material,
        }
    }

    // 进行光栅化
    pub fn rasterize_triangle<S: FragmentShader>(
        &mut self,
        triangle: &RasterTriangle,
        texture: Option<&Texture>,
        shader: &S, // 接收一个Shader
    ) {
        let points = &triangle.vertices;
        let (min_x, min_y, max_x, max_y) =
            rasterizer::get_box(&[points[0].pos, points[1].pos, points[2].pos]);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                if rasterizer::is_inside_triangle(&[points[0].pos, points[1].pos, points[2].pos], &p) {

                    let bary = rasterizer::get_barycentric_coords(
                        &[points[0].pos, points[1].pos, points[2].pos],
                        &p,
                    ).unwrap_or((0.0, 0.0, 0.0));

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

                    self.framebuffer.put_pixel(x as usize, y as usize, color, interpolated_depth);
                }
            }
        }
    }
    pub fn draw_depth_outline_prewitt(&mut self, threshold: f32, line_width: usize) {
        // 定义Prewitt算子的水平和垂直卷积核（3x3二维数组）
        let prewitt_x = [[-1.0, 0.0, 1.0], [-1.0, 0.0, 1.0], [-1.0, 0.0, 1.0]];
        let prewitt_y = [[-1.0, -1.0, -1.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]];

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
                        gx += prewitt_x[ky][kx] * depth_matrix[ny][nx];
                        gy += prewitt_y[ky][kx] * depth_matrix[ny][nx];
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
}
