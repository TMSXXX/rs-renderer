use crate::BLACK;
use crate::texture::Texture;
use crate::vertex::{ColoredVertex, Material, RasterPoint, RasterTriangle, Triangle};
use crate::{camera, framebuffer, rasterizer};
use camera::Camera;
use cgmath::{ElementWise, InnerSpace, Matrix, Matrix3, Matrix4 as Mat4, SquareMatrix, Zero, dot};
use cgmath::{Matrix3 as Mat3, Vector2 as Vec2, Vector3 as Vec3, Vector4 as Vec4, prelude::*};
use framebuffer::FrameBuffer;


use crate::renderer_debug::RendererDebugUtils; // 已经被迁移出去的旧函数 

pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

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
        print!(" 转换");
        RasterTriangle {
            vertices: raster_vertices,
            material: triangle.material,
        }
    }

    pub fn rasterize_colored_triangle(
        &mut self,
        triangle: &RasterTriangle,
        texture: Option<&Texture>,
    ) {
        /*
        let points = &triangle.vertices;
        let cross_product_z = (points[1].pos.x - points[0].pos.x)
            * (points[2].pos.y - points[0].pos.y)
            - (points[1].pos.y - points[0].pos.y) * (points[2].pos.x - points[0].pos.x);

        if cross_product_z < 0.0 {
            return;
        }
        */

        let points = &triangle.vertices;
        let material = &triangle.material;
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
                    // 插值
                    let bary = bary.unwrap_or((0.0, 0.0, 0.0));
                    let mut interpolated_color = rasterizer::interpolate_color(points, bary);
                    let interpolated_depth = rasterizer::interpolate_depth(points, bary);
                    let interpolated_normal = rasterizer::interpolate_normal(points, bary);
                    let interpolated_uv = rasterizer::interpolate_uv(points, bary);
                    let world_pos = {
                        let p0 = points[0].world_pos * bary.2; // bary.2 是第一个顶点的权重
                        let p1 = points[1].world_pos * bary.1;
                        let p2 = points[2].world_pos * bary.0;
                        p0 + p1 + p2
                    };

                    // 纹理颜色采样
                    if let Some(tex) = texture {
                        interpolated_color = tex.sample(interpolated_uv);
                    }

                    // 环境光分量
                    let ambient = self.light.ambient_color * self.light.ambient_strength;

                    // 漫反射分量
                    let light_dir = self.light.direction.normalize();
                    let diff = interpolated_normal.dot(-light_dir).max(0.0);
                    let diffuse = self.light.color * self.light.intensity * diff;

                    let diffuse = if diff > 0.6 {
                        self.light.color * self.light.intensity * 1.1
                    } else if diff > 0.2 {
                        self.light.color * self.light.intensity * 0.8
                    } else {
                        self.light.color * self.light.intensity * 0.5
                    };

                    let specular = {
                        // 视线方向（从像素到相机）
                        let view_dir = (self.camera.eye - world_pos).normalize();
                        // 半程向量
                        let half_dir = (-light_dir + view_dir).normalize();
                        // 高光强度（结合材质的反光度）
                        let spec = interpolated_normal.dot(half_dir).max(0.0);
                        let spec = spec.powf(material.shininess);
                        // 高光颜色 = 光源色 * 材质高光色 * 材质高光强度 * 计算值
                        self.light.color.mul_element_wise(material.specular)
                            * material.specular_strength
                            * spec
                    };

                    // 合并光照
                    let mut final_color =
                        interpolated_color.mul_element_wise(ambient + diffuse + specular);
                    final_color.x = final_color.x.clamp(0.0, 1.0);
                    final_color.y = final_color.y.clamp(0.0, 1.0);
                    final_color.z = final_color.z.clamp(0.0, 1.0);
                    //let final_color = (interpolated_color + Vec3::new(1.0, 1.0, 1.0)) * 0.5;

                    // 法线产生
                    //let final_color = interpolated_normal;
                    // 转换为 u32 颜色格式（0~255 范围）
                    let color = ((final_color.x * 255.0) as u32) << 16
                        | ((final_color.y * 255.0) as u32) << 8
                        | ((final_color.z * 255.0) as u32);
                    let color = 0xFF000000 | color; // 不透明 alpha 通道
                    // 绘制像素（如果后续有深度缓冲，这里需要加深度测试）
                    self.framebuffer
                        .put_pixel(x as usize, y as usize, color, interpolated_depth);
                }
            }
        }
        println!("着色");
    }

    // 绘制多个带颜色插值的三角形

    pub fn render_colored_triangles(
        &mut self,
        triangles: &mut Vec<Triangle>,
        model: &Mat4<f32>,
        texture: Option<&Texture>,
    ) {
        println!("三角形数量: {}", triangles.len());
        let normal_matrix = model.invert().unwrap().transpose();
        let mut i = 0;
        for triangle in triangles {
            println!("{i}");
            i += 1;
            let world_pos = (*model * triangle.vertices[0].pos.extend(1.0)).truncate();
            let view_dir = (self.camera.eye - world_pos).normalize();
            let tri_normal = (normal_matrix * triangle.normal.extend(0.0)).truncate();
            // 提前剔除背面
            if view_dir.dot(tri_normal) <= 0.0 {
                continue;
            }
            let raster_triangle = self.transform_colored_vertices(triangle, model);
            self.rasterize_colored_triangle(&raster_triangle, texture);
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
